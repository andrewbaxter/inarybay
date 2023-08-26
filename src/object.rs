use std::{
    rc::{
        Rc,
        Weak,
    },
    cell::RefCell,
    fmt::Display,
    collections::{
        HashMap,
        BTreeMap,
    },
};
use proc_macro2::TokenStream;
use quote::IdentFragment;
use crate::{
    util::{
        S,
        new_s,
        BVec,
        ToIdent,
    },
    node_serial::{
        NodeSerial,
        NodeSerialSegment,
    },
    node_rust::{
        NodeRustObj,
        NodeRustField,
    },
    node_const::NodeConst,
    schema::{
        Schema_,
    },
    node::{
        Node,
        RedirectRef,
    },
    node_fixed_bytes::NodeFixedBytes,
    node_int::{
        NodeInt,
        NodeIntArgs,
        Endian,
    },
    node_dynamic_bytes::NodeDynamicBytes,
    node_dynamic_array::NodeDynamicArray,
    node_enum::{
        NodeEnum,
        EnumVariant,
    },
};

#[derive(Clone)]
struct NestingParentEnum {
    enum_: S<NodeEnum>,
    variant_name: String,
    parent: WeakObj,
}

#[derive(Clone)]
pub(crate) enum NestingParent {
    None,
    Enum(NestingParentEnum),
}

#[derive(Clone)]
pub(crate) enum SomeNestingParent {
    Enum(NestingParentEnum),
}

pub(crate) struct Object_ {
    pub(crate) schema: Weak<RefCell<Schema_>>,
    pub(crate) nesting_parent: NestingParent,
    pub(crate) id: String,
    pub(crate) serial_root: S<NodeSerial>,
    pub(crate) rust_root: S<NodeRustObj>,
    pub(crate) rust_const_roots: Vec<S<NodeConst>>,
    pub(crate) has_external_deps: bool,
}

#[derive(Clone)]
pub(crate) struct Object(pub(crate) Rc<RefCell<Object_>>);

pub(crate) type WeakObj = Weak<RefCell<Object_>>;

#[derive(Clone, Copy)]
struct RangeAllocSingle {
    /// Relative to root serial
    start: BVec,
    avail: BVec,
}

struct RangeAllocEnum {
    enum_id: String,
    template_alloc: RangeAllocSingle,
    variants: HashMap<String, RangeAlloc>,
}

enum RangeAlloc {
    Unset(RangeAllocSingle),
    Local(RangeAllocSingle),
    Enum(RangeAllocEnum),
}

struct Range_ {
    serial: S<NodeFixedBytes>,
    alloc: RangeAlloc,
}

#[derive(Clone)]
pub struct Range(Rc<RefCell<Range_>>);

pub struct Enum {
    schema: Rc<RefCell<Schema_>>,
    obj: Object,
    enum_: S<NodeEnum>,
}

impl Object {
    pub(crate) fn new(schema: &Rc<RefCell<Schema_>>, name: String) -> Object {
        let mut schema2 = schema.as_ref().borrow_mut();
        let out = Object(Rc::new(RefCell::new(Object_ {
            schema: Rc::downgrade(schema),
            nesting_parent: NestingParent::None,
            id: schema2.take_id(),
            serial_root: new_s(NodeSerial {
                id: "serial".into(),
                children: vec![],
                lifted_serial_deps: BTreeMap::new(),
            }),
            rust_root: new_s(NodeRustObj {
                id: schema2.take_id(),
                type_name: name.clone(),
                fields: vec![],
            }),
            rust_const_roots: vec![],
            has_external_deps: false,
        })));
        schema2.objects.entry(name).or_insert_with(Vec::new).push(out);
        return out;
    }

    fn seg(&self) -> S<NodeSerialSegment> {
        let self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let out = new_s(NodeSerialSegment {
            scope: Rc::downgrade(&self.0),
            id: schema.take_id(),
            serial_root: self2.serial_root.into(),
            serial_before: root.children.last().cloned(),
            rust: None,
        });
        root.children.push(out);
        return out;
    }

    pub fn fixed_bytes(&self, bytes: usize) -> Range {
        let seg = self.seg();
        let self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let serial = new_s(NodeFixedBytes {
            scope: Rc::downgrade(&self.0),
            id: schema.take_id(),
            serial_before: root.children.last().map(|x| Node::from(*x)),
            serial: seg,
            len_bytes: bytes,
            sub_ranges: vec![],
            rust: None,
        });
        seg.borrow_mut().rust = Some(serial.into());
        return Range(Rc::new(RefCell::new(Range_ {
            serial,
            alloc: RangeAlloc::Unset(RangeAllocSingle {
                start: BVec::zero(),
                avail: BVec::bytes(bytes),
            }),
        })));
    }

    fn get_ancestry_to(&self, level: &Rc<RefCell<Object_>>) -> Vec<SomeNestingParent> {
        let mut ancestry = vec![];
        let mut at = self.0.clone();
        loop {
            if at.as_ptr() == level.as_ptr() {
                break;
            }
            match &at.borrow().nesting_parent {
                NestingParent::None => {
                    panic!("TODO different tree");
                },
                NestingParent::Enum(e) => {
                    ancestry.push(SomeNestingParent::Enum(e.clone()));
                    at = e.parent.upgrade().unwrap();
                },
            };
        }
        ancestry.reverse();
        return ancestry;
    }

    fn modify_range<
        T,
    >(
        &self,
        range: &mut Range_,
        ancestry: &Vec<SomeNestingParent>,
        f: impl FnOnce(&mut RangeAllocSingle) -> T,
    ) -> T {
        let at = &mut range.alloc;
        for e in ancestry {
            match e {
                SomeNestingParent::Enum(ancestor_enum) => {
                    match at {
                        RangeAlloc::Unset(alloc) => {
                            let alloc = *alloc;
                            *at = RangeAlloc::Enum(RangeAllocEnum {
                                enum_id: ancestor_enum.enum_.borrow().type_name.clone(),
                                template_alloc: alloc,
                                variants: HashMap::new(),
                            });
                            match &mut at {
                                RangeAlloc::Enum(e) => {
                                    e.variants.insert(ancestor_enum.variant_name.clone(), RangeAlloc::Unset(alloc));
                                    at = e.variants.get_mut(&ancestor_enum.variant_name).unwrap();
                                },
                                _ => unreachable!(),
                            };
                        },
                        RangeAlloc::Local(_) => panic!("TODO already used in object X"),
                        RangeAlloc::Enum(existing) => {
                            if existing.enum_id != ancestor_enum.enum_.borrow().type_name {
                                panic!("TODO already used by other enum X");
                            }
                            let old =
                                existing
                                    .variants
                                    .insert(
                                        ancestor_enum.variant_name.clone(),
                                        RangeAlloc::Unset(existing.template_alloc),
                                    );
                            if old.is_some() {
                                panic!("TODO already used by other variant X");
                            }
                            at = existing.variants.get_mut(&ancestor_enum.variant_name).unwrap();
                        },
                    }
                },
            }
        }
        match at {
            RangeAlloc::Unset(alloc) => {
                let avail = *alloc;
                range.alloc = RangeAlloc::Local(avail);
                match &mut range.alloc {
                    RangeAlloc::Local(avail) => {
                        return f(avail);
                    },
                    _ => unreachable!(),
                };
            },
            RangeAlloc::Local(alloc) => {
                return f(alloc);
            },
            RangeAlloc::Enum(enum_) => {
                panic!("TODO already used by child variant X");
            },
        }
    }

    pub fn sub_fixed_bytes(&self, range: Range, bytes: usize, bits: usize) -> Range {
        let mut range2 = range.0.as_ref().borrow_mut();
        let using = BVec {
            bytes: bytes,
            bits: bits,
        };
        if using.bits > 8 {
            panic!("Bits should be in excess of bytes (i.e. < 8), but got argument of {}b", using.bits);
        }
        let start =
            self.modify_range(
                &mut range2,
                &self.get_ancestry_to(&range2.serial.borrow().scope.upgrade().unwrap()),
                |alloc| {
                    if using > alloc.avail {
                        panic!("Range has {} available, but subrange consumes {}", alloc.avail, using);
                    }
                    let out = alloc.start;
                    alloc.start += using;
                    alloc.avail -= using;
                    return out;
                },
            );
        let out = Range(Rc::new(RefCell::new(Range_ {
            serial: range2.serial,
            alloc: RangeAlloc::Unset(RangeAllocSingle {
                start: start,
                avail: using,
            }),
        })));
        return out;
    }

    fn lift_connect(&mut self, ancestry: &Vec<SomeNestingParent>, serial: Node, rust: Node) {
        let mut self2 = self.0.as_ref().borrow_mut();
        if !ancestry.is_empty() {
            self2.serial_root.borrow_mut().lifted_serial_deps.insert(rust.id(), rust.clone());
            for (i, level) in ancestry.iter().enumerate() {
                match level {
                    SomeNestingParent::Enum(level) => {
                        if i == 0 {
                            // 1A
                            serial.set_rust(rust);

                            // 1B
                            level.enum_.borrow_mut().external_deps.insert(serial.id(), serial);
                        } else {
                            let parent = level.parent.upgrade().unwrap().borrow_mut();
                            let level_serial_root = parent.serial_root;

                            // 2A
                            level_serial_root.borrow_mut().lifted_serial_deps.insert(level.enum_.borrow().id.clone(), level.enum_.into());

                            // 2B
                            level.enum_.borrow_mut().external_deps.insert(level_serial_root.borrow().id.clone(), level_serial_root.into());

                            // 4
                            parent.has_external_deps = true;
                        }
                    },
                }
            }
        } else {
            // 3A
            serial.set_rust(rust);
        }
    }

    pub fn int(&mut self, range: Range, endian: Endian, signed: bool) -> S<NodeInt> {
        let mut range2 = range.0.as_ref().borrow_mut();
        let serial = &range2.serial;
        let ancestry = self.get_ancestry_to(&serial.borrow().scope.upgrade().unwrap());
        let using = self.modify_range(&mut range2, &ancestry, |alloc| {
            if alloc.avail == BVec::zero() {
                panic!("Range has no space available");
            }
            return alloc;
        });
        let mut self2 = self.0.as_ref().borrow_mut();
        let mut use_serial = RedirectRef::new(range2.serial);
        if !ancestry.is_empty() {
            // 3B
            use_serial.redirect = Some(self2.serial_root.into());
        }
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let serial2 = serial.borrow_mut();
        let rust = new_s(NodeInt::new(NodeIntArgs {
            scope: Rc::downgrade(&self.0),
            id: schema.take_id(),
            serial: use_serial,
            start: using.start,
            len: using.avail,
            signed: signed,
            endian: endian,
        }));
        self.lift_connect(&ancestry, Node::from(*serial), rust.into());
        return rust;
    }

    pub fn dynamic_bytes(&self, len: S<NodeInt>) -> S<NodeDynamicBytes> {
        let self2 = self.0.as_ref().borrow_mut();
        let serial = self.seg();
        let len_ancestry = self.get_ancestry_to(&len.borrow().scope.upgrade().unwrap());
        let mut use_len = RedirectRef::new(len);
        if !len_ancestry.is_empty() {
            // 3B
            use_len.redirect = Some(self2.serial_root.into());
        }
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let rust = new_s(NodeDynamicBytes {
            scope: Rc::downgrade(&self.0),
            id: schema.take_id(),
            serial_before: root.children.last().map(|x| Node::from(*x)),
            serial: self2.serial_root.into(),
            serial_len: use_len,
            rust: None,
        });
        self.lift_connect(&len_ancestry, len.into(), rust.into());
        return rust;
    }

    pub fn dynamic_array(&self, len: S<NodeInt>, obj_name: impl Display) -> (S<NodeDynamicArray>, Object) {
        let serial = self.seg();
        let obj_name = obj_name.to_string();
        let self2 = self.0.as_ref().borrow_mut();
        let len_ancestry = self.get_ancestry_to(&len.borrow().scope.upgrade().unwrap());
        let mut use_len = RedirectRef::new(len);
        if !len_ancestry.is_empty() {
            // 3B
            use_len.redirect = Some(self2.serial_root.into());
        }
        let schema = self2.schema.upgrade().unwrap();
        let schema2 = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let element = Object::new(&schema, obj_name);
        let rust = new_s(NodeDynamicArray {
            scope: Rc::downgrade(&self.0),
            id: schema2.take_id(),
            serial_before: root.children.last().map(|x| Node::from(*x)),
            serial: serial,
            serial_len: use_len,
            element: element.clone(),
            rust: None,
        });
        self.lift_connect(&len_ancestry, len.into(), rust.into());
        return (rust, element);
    }

    pub fn enum_(&self, tag: Node, enum_name: impl Display) -> (S<NodeEnum>, Enum) {
        let serial = self.seg();
        let enum_name = enum_name.to_string();
        let self2 = self.0.as_ref().borrow_mut();
        let tag_ancestry = self.get_ancestry_to(&tag.scope());
        let mut use_tag = RedirectRef::new(tag);
        if !tag_ancestry.is_empty() {
            // 3B
            use_tag.redirect = Some(self2.serial_root.into());
        }
        let schema = self2.schema.upgrade().unwrap();
        let schema2 = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let rust = new_s(NodeEnum {
            scope: Rc::downgrade(&self.0),
            id: schema2.take_id(),
            type_name: enum_name.clone(),
            serial_before: root.children.last().map(|x| Node::from(*x)),
            serial: serial,
            serial_tag: use_tag,
            variants: vec![],
            rust: None,
            external_deps: BTreeMap::new(),
        });
        schema2.enums.entry(enum_name).or_insert_with(Vec::new).push(rust);
        self.lift_connect(&tag_ancestry, tag.into(), rust.into());
        return (rust, Enum {
            schema: schema.clone(),
            obj: self.clone(),
            enum_: rust,
        });
    }

    pub fn rust_const_int(&self, serial: impl Into<Node>, value: TokenStream) {
        let serial: Node = serial.into();
        let mut self2 = self.0.as_ref().borrow_mut();
        let serial_ancestry = self.get_ancestry_to(&serial.scope());
        let mut use_serial = RedirectRef::new(serial);
        if !serial_ancestry.is_empty() {
            // 3B
            use_serial.redirect = Some(self2.serial_root.into());
        }
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let rust = new_s(NodeConst {
            id: schema.take_id(),
            serial: use_serial,
            expect: value,
        });
        self2.rust_const_roots.push(rust);
        self.lift_connect(&serial_ancestry, serial, rust.into());
    }

    pub fn rust_field(&self, serial: impl Into<Node>, name: impl Display) {
        let serial = serial.into();
        let self2 = self.0.as_ref().borrow_mut();
        let serial_ancestry = self.get_ancestry_to(&serial.scope());
        let mut use_serial = RedirectRef::new(serial);
        if !serial_ancestry.is_empty() {
            // 3B
            use_serial.redirect = Some(self2.serial_root.into());
        }
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let rust = new_s(NodeRustField {
            id: schema.take_id(),
            field_name: name.to_string(),
            serial: use_serial,
            obj: self2.rust_root,
        });
        self2.rust_root.borrow_mut().fields.push(rust);
        self.lift_connect(&serial_ancestry, serial, rust.into());
    }
}

impl Enum {
    pub fn variant(&self, variant_name: impl Display, obj_name: impl Display, tag: TokenStream) -> Object {
        let variant_name = variant_name.to_string();
        let enum_ = self.enum_.borrow_mut();
        let element = Object::new(&self.schema, obj_name.to_string());
        enum_.variants.push(EnumVariant {
            var_name: variant_name.clone(),
            tag: tag,
            element: element,
        });
        element.0.borrow_mut().nesting_parent = NestingParent::Enum(NestingParentEnum {
            enum_: self.enum_.clone(),
            variant_name: variant_name,
            parent: Rc::downgrade(&self.obj.0),
        });
        return element;
    }
}
