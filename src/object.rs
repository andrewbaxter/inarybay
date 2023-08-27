use std::{
    fmt::Display,
    collections::{
        HashMap,
        BTreeMap,
    },
    borrow::BorrowMut,
};
use gc::{
    Finalize,
    Trace,
    GcCell,
    Gc,
};
use proc_macro2::TokenStream;
use crate::{
    util::{
        new_s,
        BVec,
        LateInit,
    },
    node_serial::{
        NodeSerial,
        NodeSerialSegment,
        NodeSerial_,
        NodeSerialSegment_,
    },
    node_rust::{
        NodeRustObj,
        NodeRustField,
        NodeRustObj_,
        NodeRustField_,
    },
    node_const::{
        NodeConst,
        NodeConst_,
    },
    schema::{
        Schema,
    },
    node::{
        Node,
        RedirectRef,
        NodeMethods,
    },
    node_fixed_bytes::{
        NodeFixedBytes,
        NodeFixedBytes_,
    },
    node_int::{
        NodeInt,
        NodeIntArgs,
        Endian,
    },
    node_dynamic_bytes::{
        NodeDynamicBytes,
        NodeDynamicBytes_,
    },
    node_dynamic_array::{
        NodeDynamicArray,
        NodeDynamicArray_,
    },
    node_enum::{
        NodeEnum,
        EnumVariant,
        NodeEnum_,
    },
};

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NestingParentEnum {
    enum_: NodeEnum,
    variant_name: String,
    parent: Object,
}

#[derive(Clone, Trace, Finalize)]
pub(crate) enum NestingParent {
    None,
    Enum(NestingParentEnum),
}

#[derive(Clone, Trace, Finalize)]
pub(crate) enum SomeNestingParent {
    Enum(NestingParentEnum),
}

#[derive(Trace, Finalize)]
pub(crate) struct Object_ {
    pub(crate) schema: Schema,
    pub(crate) nesting_parent: NestingParent,
    pub(crate) id: String,
    pub(crate) serial_root: NodeSerial,
    pub(crate) rust_root: NodeRustObj,
    pub(crate) rust_const_roots: Vec<NodeConst>,
    pub(crate) has_external_deps: bool,
}

#[derive(Clone, Trace, Finalize)]
pub struct Object(pub(crate) Gc<GcCell<Object_>>);

#[derive(Clone, Copy)]
struct RangeAllocSingle {
    /// Relative to root serial
    start: BVec,
    avail: BVec,
}

unsafe impl Trace for RangeAllocSingle {
    unsafe fn trace(&self) { }

    unsafe fn root(&self) { }

    unsafe fn unroot(&self) { }

    fn finalize_glue(&self) { }
}

impl Finalize for RangeAllocSingle {
    fn finalize(&self) { }
}

#[derive(Trace, Finalize)]
struct RangeAllocEnum {
    enum_id: String,
    template_alloc: RangeAllocSingle,
    variants: HashMap<String, RangeAlloc>,
}

#[derive(Trace, Finalize)]
enum RangeAlloc {
    Unset(RangeAllocSingle),
    Local(RangeAllocSingle),
    Enum(RangeAllocEnum),
}

#[derive(Trace, Finalize)]
struct Range_ {
    serial: NodeFixedBytes,
    alloc: RangeAlloc,
}

#[derive(Clone)]
pub struct Range(Gc<GcCell<Range_>>);

pub struct Enum {
    schema: Schema,
    obj: Object,
    enum_: NodeEnum,
}

impl Object {
    pub(crate) fn new(id: impl Into<String>, schema: &Schema, name: String) -> Object {
        let id = id.into();
        let mut schema2 = schema.0.as_ref().borrow_mut();
        let out = Object(Gc::new(GcCell::new(Object_ {
            schema: schema.clone(),
            nesting_parent: NestingParent::None,
            id: schema.0.as_ref().borrow_mut().take_id(id.clone()),
            serial_root: NodeSerial(new_s(NodeSerial_ {
                id: schema.0.as_ref().borrow_mut().take_id(format!("{}__serial", id)),
                children: vec![],
                lifted_serial_deps: BTreeMap::new(),
            })),
            rust_root: NodeRustObj(new_s(NodeRustObj_ {
                id: schema.0.as_ref().borrow_mut().take_id(format!("{}__rust", id)),
                type_name: name.clone(),
                fields: vec![],
            })),
            rust_const_roots: vec![],
            has_external_deps: false,
        })));
        schema2.objects.entry(name).or_insert_with(Vec::new).push(out.clone());
        return out;
    }

    fn id(&self) -> String {
        return self.0.borrow().id.clone();
    }

    fn seg(&self, id: &str) -> NodeSerialSegment {
        let self2 = self.0.as_ref().borrow_mut();
        let mut root = self2.serial_root.0.as_ref().borrow_mut();
        let out = NodeSerialSegment(new_s(NodeSerialSegment_ {
            scope: self.clone(),
            id: self.0.borrow().schema.0.as_ref().borrow_mut().take_id(format!("{}__serial_seg", id)),
            serial_root: self2.serial_root.clone().into(),
            serial_before: root.children.last().cloned(),
            rust: None,
        }));
        root.children.push(out.clone());
        return out;
    }

    pub fn fixed_bytes(&self, id: impl Into<String>, bytes: usize) -> Range {
        let id = id.into();
        let seg = self.seg(&id);
        let self2 = self.0.as_ref().borrow();
        let serial = NodeFixedBytes(new_s(NodeFixedBytes_ {
            scope: self.clone(),
            id: self.0.borrow().schema.0.as_ref().borrow_mut().take_id(id),
            serial_before: self2.serial_root.0.borrow().children.last().map(|x| x.clone().into()),
            serial: seg.clone(),
            len_bytes: bytes,
            rust: BTreeMap::new(),
        }));
        seg.0.as_ref().borrow_mut().rust = Some(serial.clone().into());
        return Range(Gc::new(GcCell::new(Range_ {
            serial: serial,
            alloc: RangeAlloc::Unset(RangeAllocSingle {
                start: BVec::zero(),
                avail: BVec::bytes(bytes),
            }),
        })));
    }

    fn get_ancestry_to(&self, level: &Object) -> Vec<SomeNestingParent> {
        let mut ancestry = vec![];
        let mut at = self.clone();
        loop {
            if at.id() == level.id() {
                break;
            }
            let nesting_parent = at.0.borrow().nesting_parent.clone();
            match &nesting_parent {
                NestingParent::None => {
                    panic!("TODO different tree");
                },
                NestingParent::Enum(e) => {
                    ancestry.push(SomeNestingParent::Enum(e.clone()));
                    at = e.parent.clone();
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
        let mut at = &mut range.alloc;
        for e in ancestry {
            match e {
                SomeNestingParent::Enum(ancestor_enum) => {
                    match at {
                        RangeAlloc::Unset(alloc) => {
                            let alloc = alloc.clone();
                            *at = RangeAlloc::Enum(RangeAllocEnum {
                                enum_id: ancestor_enum.enum_.0.borrow().type_name.clone(),
                                template_alloc: alloc,
                                variants: HashMap::new(),
                            });
                            match at {
                                RangeAlloc::Enum(e) => {
                                    e.variants.insert(ancestor_enum.variant_name.clone(), RangeAlloc::Unset(alloc));
                                    at = e.variants.get_mut(&ancestor_enum.variant_name).unwrap();
                                },
                                _ => unreachable!(),
                            };
                        },
                        RangeAlloc::Local(_) => panic!("TODO already used in object X"),
                        RangeAlloc::Enum(existing) => {
                            if existing.enum_id != ancestor_enum.enum_.0.borrow().type_name {
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
        let start = {
            let ancestry = self.get_ancestry_to(&range2.serial.0.borrow().scope);
            self.modify_range(&mut range2, &ancestry, |alloc| {
                if using > alloc.avail {
                    panic!("Range has {} available, but subrange consumes {}", alloc.avail, using);
                }
                let out = alloc.start;
                alloc.start += using;
                alloc.avail -= using;
                return out;
            })
        };
        let out = Range(Gc::new(GcCell::new(Range_ {
            serial: range2.serial.clone(),
            alloc: RangeAlloc::Unset(RangeAllocSingle {
                start: start,
                avail: using,
            }),
        })));
        return out;
    }

    fn lift_connect<
        T: Clone + NodeMethods + Into<Node> + Trace + Finalize,
    >(
        &self,
        ancestry: &Vec<SomeNestingParent>,
        serial: &T,
        rust: Node,
        rust_field: &mut LateInit<RedirectRef<T, Node>>,
    ) {
        let self2 = self.0.as_ref().borrow();
        if ancestry.is_empty() {
            serial.set_rust(rust.clone());
            *rust_field = Some(RedirectRef::new(serial.clone()));
        } else {
            for (i, level) in ancestry.iter().enumerate() {
                match level {
                    SomeNestingParent::Enum(level) => {
                        if i == 0 {
                            // 1A
                            serial.set_rust(rust.clone());

                            // 1B
                            level.enum_.0.as_ref().borrow_mut().external_deps.insert(serial.id(), serial.clone().into());
                        } else {
                            let mut parent = level.parent.0.as_ref().borrow_mut();
                            let level_serial_root = &parent.serial_root;

                            // 2A
                            level_serial_root
                                .0
                                .as_ref()
                                .borrow_mut()
                                .lifted_serial_deps
                                .insert(level.enum_.0.borrow().id.clone(), level.enum_.clone().into());

                            // 2B
                            level
                                .enum_
                                .0
                                .as_ref()
                                .borrow_mut()
                                .external_deps
                                .insert(level_serial_root.0.borrow().id.clone(), level_serial_root.clone().into());

                            // 4
                            parent.has_external_deps = true;
                        }
                    },
                }
            }

            // 3A
            self2.serial_root.0.as_ref().borrow_mut().lifted_serial_deps.insert(rust.id(), rust.clone());

            // 3B
            *rust_field = Some(RedirectRef {
                primary: serial.clone(),
                redirect: Some(self2.serial_root.clone().into()),
            });
        }
    }

    pub fn int(&mut self, id: impl Into<String>, range: Range, endian: Endian, signed: bool) -> NodeInt {
        let id = id.into();
        let mut range2 = range.0.as_ref().borrow_mut();
        let ancestry = self.get_ancestry_to(&range2.serial.0.borrow().scope);
        let using = self.modify_range(&mut range2, &ancestry, |alloc| {
            if alloc.avail == BVec::zero() {
                panic!("Range has no space available");
            }
            return alloc.clone();
        });
        let rust = NodeInt::new(NodeIntArgs {
            scope: self.clone(),
            id: self.0.borrow().schema.0.as_ref().borrow_mut().take_id(id),
            start: using.start,
            len: using.avail,
            signed: signed,
            endian: endian,
        });
        self.lift_connect(&ancestry, &range2.serial, rust.clone().into(), &mut rust.0.as_ref().borrow_mut().serial);
        return rust;
    }

    pub fn dynamic_bytes(&self, id: impl Into<String>, len: NodeInt) -> NodeDynamicBytes {
        let id = id.into();
        let serial = self.seg(&id);
        let rust = NodeDynamicBytes(new_s(NodeDynamicBytes_ {
            scope: self.clone(),
            id: self.0.borrow().schema.0.as_ref().borrow_mut().take_id(id),
            serial_before: self.0.borrow().serial_root.0.borrow().children.last().map(|x| x.clone().into()),
            serial: serial,
            serial_len: None,
            rust: None,
        }));
        self.lift_connect(
            &self.get_ancestry_to(&len.0.borrow().scope),
            &len,
            rust.clone().into(),
            &mut rust.0.as_ref().borrow_mut().serial_len,
        );
        return rust;
    }

    pub fn dynamic_array(
        &self,
        id: impl Into<String>,
        len: NodeInt,
        obj_name: impl Display,
    ) -> (NodeDynamicArray, Object) {
        let id = id.into();
        let serial = self.seg(&id);
        let obj_name = obj_name.to_string();
        let self2 = self.0.as_ref().borrow_mut();
        let root = self2.serial_root.0.as_ref().borrow_mut();
        let element = Object::new(&id, &self2.schema, obj_name);
        let rust = NodeDynamicArray(new_s(NodeDynamicArray_ {
            scope: self.clone(),
            id: self2.schema.0.as_ref().borrow_mut().take_id(id),
            serial_before: root.children.last().map(|x| x.clone().into()),
            serial: serial,
            serial_len: None,
            element: element.clone(),
            rust: None,
        }));
        self.lift_connect(
            &self.get_ancestry_to(&len.0.borrow().scope),
            &len,
            rust.clone().into(),
            &mut rust.0.as_ref().borrow_mut().serial_len,
        );
        return (rust, element);
    }

    pub fn enum_(&self, id: impl Into<String>, tag: Node, enum_name: impl Display) -> (NodeEnum, Enum) {
        let id = id.into();
        let serial = self.seg(&id);
        let enum_name = enum_name.to_string();
        let self2 = self.0.as_ref().borrow_mut();
        let root = self2.serial_root.0.as_ref().borrow_mut();
        let rust = NodeEnum(new_s(NodeEnum_ {
            scope: self.clone(),
            id: self2.schema.0.as_ref().borrow_mut().take_id(id),
            type_name: enum_name.clone(),
            serial_before: root.children.last().map(|x| x.clone().into()),
            serial: serial,
            serial_tag: None,
            variants: vec![],
            rust: None,
            external_deps: BTreeMap::new(),
        }));
        self2.schema.0.as_ref().borrow_mut().enums.entry(enum_name).or_insert_with(Vec::new).push(rust.clone());
        self.lift_connect(
            &self.get_ancestry_to(&tag.scope()),
            &tag,
            rust.clone().into(),
            &mut rust.0.as_ref().borrow_mut().serial_tag,
        );
        return (rust.clone(), Enum {
            schema: self2.schema.clone(),
            obj: self.clone(),
            enum_: rust,
        });
    }

    pub fn rust_const_int(&self, id: impl Into<String>, serial: impl Into<Node>, value: TokenStream) {
        let id = id.into();
        let serial: Node = serial.into();
        let mut self2 = self.0.as_ref().borrow_mut();
        let rust = NodeConst(new_s(NodeConst_ {
            id: self2.schema.0.as_ref().borrow_mut().take_id(id),
            serial: None,
            expect: value,
        }));
        self2.rust_const_roots.push(rust.clone());
        self.lift_connect(
            &self.get_ancestry_to(&serial.scope()),
            &serial,
            rust.clone().into(),
            &mut rust.0.as_ref().borrow_mut().serial,
        );
    }

    pub fn rust_field(&self, id: impl Into<String>, serial: impl Into<Node>, name: impl Display) {
        let id = id.into();
        let serial = serial.into();
        let self2 = self.0.as_ref().borrow_mut();
        let rust = NodeRustField(new_s(NodeRustField_ {
            id: self2.schema.0.as_ref().borrow_mut().take_id(id),
            field_name: name.to_string(),
            serial: None,
            obj: self2.rust_root.clone(),
        }));
        self2.rust_root.0.as_ref().borrow_mut().fields.push(rust.clone());
        self.lift_connect(
            &self.get_ancestry_to(&serial.scope()),
            &serial,
            rust.clone().into(),
            &mut rust.0.as_ref().borrow_mut().serial,
        );
    }
}

impl Enum {
    pub fn variant(
        &self,
        id: impl Into<String>,
        variant_name: impl Display,
        obj_name: impl Display,
        tag: TokenStream,
    ) -> Object {
        let id = id.into();
        let variant_name = variant_name.to_string();
        let mut enum_ = self.enum_.0.as_ref().borrow_mut();
        let element = Object::new(id, &self.schema, obj_name.to_string());
        enum_.variants.push(EnumVariant {
            var_name: variant_name.clone(),
            tag: tag,
            element: element.clone(),
        });
        element.0.as_ref().borrow_mut().nesting_parent = NestingParent::Enum(NestingParentEnum {
            enum_: self.enum_.clone(),
            variant_name: variant_name,
            parent: self.obj.clone(),
        });
        return element;
    }
}
