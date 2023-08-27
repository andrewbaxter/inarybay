use std::{
    collections::{
        HashMap,
        BTreeMap,
    },
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
        BVec,
        LateInit,
    },
    node_serial::{
        NodeSerial,
        NodeSerialSegment,
        NodeSerial_,
        NodeSerialSegment_,
        NodeSerialMut_,
        NodeSerialSegmentMut_,
    },
    node_rust::{
        NodeRustObj,
        NodeRustField,
        NodeRustObj_,
        NodeRustField_,
        NodeRustObjMut_,
    },
    node_const::{
        NodeConst,
        NodeConst_,
        NodeConstMut_,
    },
    schema::{
        Schema,
    },
    node::{
        Node,
        RedirectRef,
        NodeMethods,
    },
    node_fixed_range::{
        NodeFixedRange,
        NodeFixedRange_,
        NodeFixedRangeMut_,
    },
    node_int::{
        NodeInt,
        NodeIntArgs,
        Endian,
    },
    node_dynamic_bytes::{
        NodeDynamicBytes,
        NodeDynamicBytes_,
        NodeDynamicBytesMut_,
    },
    node_dynamic_array::{
        NodeDynamicArray,
        NodeDynamicArray_,
        NodeDynamicArrayMut_,
    },
    node_enum::{
        NodeEnum,
        EnumVariant,
        NodeEnum_,
    },
    node_fixed_bytes::{
        NodeFixedBytes,
        NodeFixedBytesArgs,
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
pub(crate) struct ObjectMut_ {
    pub(crate) nesting_parent: NestingParent,
    pub(crate) rust_const_roots: Vec<NodeConst>,
    pub(crate) has_external_deps: bool,
}

#[derive(Trace, Finalize)]
pub(crate) struct Object_ {
    pub(crate) schema: Schema,
    pub(crate) id: String,
    pub(crate) serial_root: NodeSerial,
    pub(crate) rust_root: NodeRustObj,
    pub(crate) mut_: GcCell<ObjectMut_>,
}

#[derive(Clone, Trace, Finalize)]
pub struct Object(pub(crate) Gc<Object_>);

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
    serial: NodeFixedRange,
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
        let serial_root = NodeSerial(Gc::new(NodeSerial_ {
            id: schema.0.as_ref().borrow_mut().take_id(format!("{}__serial", id)),
            mut_: GcCell::new(NodeSerialMut_ {
                children: vec![],
                lifted_serial_deps: BTreeMap::new(),
            }),
        }));
        let rust_root = NodeRustObj(Gc::new(NodeRustObj_ {
            id: schema.0.as_ref().borrow_mut().take_id(format!("{}__rust", id)),
            type_name: name.clone(),
            mut_: GcCell::new(NodeRustObjMut_ { fields: vec![] }),
        }));
        let out = Object(Gc::new(Object_ {
            schema: schema.clone(),
            id: schema.0.as_ref().borrow_mut().take_id(id.clone()),
            serial_root: serial_root,
            rust_root: rust_root,
            mut_: GcCell::new(ObjectMut_ {
                nesting_parent: NestingParent::None,
                rust_const_roots: vec![],
                has_external_deps: false,
            }),
        }));
        schema.0.as_ref().borrow_mut().objects.entry(name).or_insert_with(Vec::new).push(out.clone());
        return out;
    }

    fn id(&self) -> String {
        return self.0.id.clone();
    }

    fn seg(&self, id: &str) -> NodeSerialSegment {
        let out = NodeSerialSegment(Gc::new(NodeSerialSegment_ {
            scope: self.clone(),
            id: self.0.schema.0.as_ref().borrow_mut().take_id(format!("{}__serial_seg", id)),
            serial_root: self.0.serial_root.clone().into(),
            serial_before: self.0.serial_root.0.mut_.borrow().children.last().cloned(),
            mut_: GcCell::new(NodeSerialSegmentMut_ { rust: None }),
        }));
        self.0.serial_root.0.mut_.borrow_mut().children.push(out.clone());
        return out;
    }

    pub fn fixed_range(&self, id: impl Into<String>, bytes: usize) -> Range {
        let id = id.into();
        let node_id = self.0.schema.0.as_ref().borrow_mut().take_id(id.clone());
        let seg = self.seg(&id);
        let serial = NodeFixedRange(Gc::new(NodeFixedRange_ {
            scope: self.clone(),
            id: node_id,
            serial_before: self.0.serial_root.0.mut_.borrow().children.last().map(|x| x.clone().into()),
            serial: seg.clone(),
            len_bytes: bytes,
            mut_: GcCell::new(NodeFixedRangeMut_ { rust: BTreeMap::new() }),
        }));
        seg.0.mut_.borrow_mut().rust = Some(serial.clone().into());
        return Range(Gc::new(GcCell::new(Range_ {
            serial: serial,
            alloc: RangeAlloc::Unset(RangeAllocSingle {
                start: BVec::zero(),
                avail: BVec::bytes(bytes),
            }),
        })));
    }

    fn get_ancestry_to(&self, serial: &dyn NodeMethods) -> Vec<SomeNestingParent> {
        let mut ancestry = vec![];
        let mut at = self.clone();
        loop {
            if at.id() == serial.scope().id() {
                break;
            }
            let nesting_parent = at.0.mut_.borrow().nesting_parent.clone();
            match &nesting_parent {
                NestingParent::None => {
                    panic!(
                        "Serial-side dependency {} is not from any containing scope; maybe this is within an array context and so can't depend on higher scopes",
                        serial.id()
                    );
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
                                enum_id: ancestor_enum.enum_.0.type_name.clone(),
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
                            if existing.enum_id != ancestor_enum.enum_.0.type_name {
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

    pub fn subrange(&self, range: &Range, bytes: usize, bits: usize) -> Range {
        let mut range2 = range.0.as_ref().borrow_mut();
        let using = BVec {
            bytes: bytes,
            bits: bits,
        };
        if using.bits > 8 {
            panic!("Bits should be in excess of bytes (i.e. < 8), but got argument of {}b", using.bits);
        }
        let start = {
            let ancestry = self.get_ancestry_to(&range2.serial);
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
                            level.enum_.0.mut_.borrow_mut().external_deps.insert(serial.id(), serial.clone().into());
                        } else {
                            let level_serial_root = &level.parent.0.serial_root;

                            // 2A
                            level_serial_root.0.mut_.borrow_mut().lifted_serial_deps.insert(level.enum_.0.id.clone(), level.enum_.clone().into());

                            // 2B
                            level
                                .enum_
                                .0
                                .mut_
                                .borrow_mut()
                                .external_deps
                                .insert(level_serial_root.0.id.clone(), level_serial_root.clone().into());

                            // 4
                            level.parent.0.mut_.borrow_mut().has_external_deps = true;
                        }
                    },
                }
            }

            // 3A
            self.0.serial_root.0.mut_.borrow_mut().lifted_serial_deps.insert(rust.id(), rust.clone());

            // 3B
            *rust_field = Some(RedirectRef {
                primary: serial.clone(),
                redirect: Some(self.0.serial_root.clone().into()),
            });
        }
    }

    pub fn bytes(&self, id: impl Into<String>, range: Range) -> NodeFixedBytes {
        let id = id.into();
        let mut range2 = range.0.as_ref().borrow_mut();
        let ancestry = self.get_ancestry_to(&range2.serial);
        let using = self.modify_range(&mut range2, &ancestry, |alloc| {
            if alloc.avail == BVec::zero() {
                panic!("Range has no space available");
            }
            return alloc.clone();
        });
        if using.start.bits != 0 {
            panic!("Must be byte-aligned but has non-zero bit offset");
        }
        if using.avail.bits != 0 {
            panic!("Must be whole-byte-sized but has non-zero bit length");
        }
        let rust = NodeFixedBytes::new(NodeFixedBytesArgs {
            scope: self.clone(),
            id: self.0.schema.0.as_ref().borrow_mut().take_id(id),
            start: using.start.bytes,
            len: using.avail.bytes,
        });
        self.lift_connect(&ancestry, &range2.serial, rust.clone().into(), &mut rust.0.mut_.borrow_mut().serial);
        return rust;
    }

    pub fn int(&self, id: impl Into<String>, range: Range, endian: Endian, signed: bool) -> NodeInt {
        let id = id.into();
        let mut range2 = range.0.as_ref().borrow_mut();
        let ancestry = self.get_ancestry_to(&range2.serial);
        let using = self.modify_range(&mut range2, &ancestry, |alloc| {
            if alloc.avail == BVec::zero() {
                panic!("Range has no space available");
            }
            return alloc.clone();
        });
        let rust = NodeInt::new(NodeIntArgs {
            scope: self.clone(),
            id: self.0.schema.0.as_ref().borrow_mut().take_id(id),
            start: using.start,
            len: using.avail,
            signed: signed,
            endian: endian,
        });
        self.lift_connect(&ancestry, &range2.serial, rust.clone().into(), &mut rust.0.mut_.borrow_mut().serial);
        return rust;
    }

    pub fn dynamic_bytes(&self, id: impl Into<String>, len: NodeInt) -> NodeDynamicBytes {
        let id = id.into();
        let serial = self.seg(&id);
        let rust = NodeDynamicBytes(Gc::new(NodeDynamicBytes_ {
            scope: self.clone(),
            id: self.0.schema.0.as_ref().borrow_mut().take_id(id),
            serial_before: self.0.serial_root.0.mut_.borrow().children.last().map(|x| x.clone().into()),
            serial: serial,
            mut_: GcCell::new(NodeDynamicBytesMut_ {
                serial_len: None,
                rust: None,
            }),
        }));
        self.lift_connect(
            &self.get_ancestry_to(&len),
            &len,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial_len,
        );
        return rust;
    }

    pub fn dynamic_array(
        &self,
        id: impl Into<String>,
        len: NodeInt,
        obj_name: impl Into<String>,
    ) -> (NodeDynamicArray, Object) {
        let id = id.into();
        let serial = self.seg(&id);
        let obj_name = obj_name.into();
        let element = Object::new(&format!("{}__elem", id), &self.0.schema, obj_name);
        let rust = NodeDynamicArray(Gc::new(NodeDynamicArray_ {
            scope: self.clone(),
            id: self.0.schema.0.as_ref().borrow_mut().take_id(id),
            serial_before: self.0.serial_root.0.mut_.borrow().children.last().map(|x| x.clone().into()),
            serial: serial,
            element: element.clone(),
            mut_: GcCell::new(NodeDynamicArrayMut_ {
                serial_len: None,
                rust: None,
            }),
        }));
        self.lift_connect(
            &self.get_ancestry_to(&len),
            &len,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial_len,
        );
        return (rust, element);
    }

    pub fn enum_(
        &self,
        id: impl Into<String>,
        tag: impl Into<Node>,
        enum_name: impl Into<String>,
    ) -> (NodeEnum, Enum) {
        let id = id.into();
        let tag = tag.into();
        let serial = self.seg(&id);
        let enum_name = enum_name.into();
        let rust = NodeEnum(Gc::new(NodeEnum_ {
            scope: self.clone(),
            id: self.0.schema.0.as_ref().borrow_mut().take_id(id),
            type_name: enum_name.clone(),
            serial_before: self.0.serial_root.0.mut_.borrow().children.last().map(|x| x.clone().into()),
            serial: serial,
            mut_: GcCell::new(crate::node_enum::NodeEnumMut_ {
                serial_tag: None,
                variants: vec![],
                rust: None,
                external_deps: BTreeMap::new(),
            }),
        }));
        self.0.schema.0.as_ref().borrow_mut().enums.entry(enum_name).or_insert_with(Vec::new).push(rust.clone());
        self.lift_connect(
            &self.get_ancestry_to(&tag),
            &tag,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial_tag,
        );
        return (rust.clone(), Enum {
            schema: self.0.schema.clone(),
            obj: self.clone(),
            enum_: rust,
        });
    }

    pub fn rust_const(&self, id: impl Into<String>, serial: impl Into<Node>, value: TokenStream) {
        let id = id.into();
        let serial: Node = serial.into();
        let rust = NodeConst(Gc::new(NodeConst_ {
            id: self.0.schema.0.as_ref().borrow_mut().take_id(id),
            expect: value,
            mut_: GcCell::new(NodeConstMut_ { serial: None }),
        }));
        self.0.mut_.borrow_mut().rust_const_roots.push(rust.clone());
        self.lift_connect(
            &self.get_ancestry_to(&serial),
            &serial,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial,
        );
    }

    pub fn rust_field(&self, id: impl Into<String>, serial: impl Into<Node>, name: impl Into<String>) {
        let id = id.into();
        let serial = serial.into();
        let rust = NodeRustField(Gc::new(NodeRustField_ {
            id: self.0.schema.0.as_ref().borrow_mut().take_id(id),
            field_name: name.into(),
            obj: self.0.rust_root.clone(),
            mut_: GcCell::new(crate::node_rust::NodeRustFieldMut_ { serial: None }),
        }));
        self.0.rust_root.0.mut_.borrow_mut().fields.push(rust.clone());
        self.lift_connect(
            &self.get_ancestry_to(&serial),
            &serial,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial,
        );
    }
}

impl Enum {
    pub fn variant(
        &self,
        id: impl Into<String>,
        variant_name: impl Into<String>,
        obj_name: impl Into<String>,
        tag: TokenStream,
    ) -> Object {
        let id = id.into();
        let variant_name = variant_name.into();
        let element = Object::new(id, &self.schema, obj_name.into());
        self.enum_.0.mut_.borrow_mut().variants.push(EnumVariant {
            var_name: variant_name.clone(),
            tag: tag,
            element: element.clone(),
        });
        element.0.mut_.borrow_mut().nesting_parent = NestingParent::Enum(NestingParentEnum {
            enum_: self.enum_.clone(),
            variant_name: variant_name,
            parent: self.obj.clone(),
        });
        return element;
    }
}
