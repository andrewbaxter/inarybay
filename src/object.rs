use std::{
    rc::{
        Rc,
        Weak,
    },
    cell::RefCell,
    fmt::Display,
};
use proc_macro2::TokenStream;
use quote::IdentFragment;
use crate::{
    util::{
        S,
        new_s,
        Coord,
        ToIdent,
        RedirectRef,
    },
    node_serial::NodeSerial,
    node_rust_obj::{
        NodeRustObj,
        NodeRustField,
    },
    node_rust_const::NodeRustConst,
    schema::{
        Schema_,
    },
    node::{
        Node,
    },
    node_fixed_range::NodeSerialFixedRange,
    node_int::{
        NodeInt,
        NodeIntArgs,
        Endian,
    },
    node_dynamic_range::NodeDynamicRange,
    node_dynamic_array::NodeDynamicArray,
    node_enum::{
        NodeEnum,
        EnumVariant,
    },
    node_option::NodeOption,
};

pub(crate) enum NestingParent {
    None,
    Enum(S<NodeEnum>, Object),
    Option(S<NodeOption>, Object),
}

pub(crate) struct Object_ {
    pub(crate) schema: Weak<RefCell<Schema_>>,
    pub(crate) nesting_parent: NestingParent,
    pub(crate) id: String,
    pub(crate) serial_root: S<NodeSerial>,
    pub(crate) rust_root: S<NodeRustObj>,
    pub(crate) rust_const_roots: Vec<S<NodeRustConst>>,
    pub(crate) external_deps: Vec<Node>,
}

#[derive(Clone)]
pub(crate) struct Object(pub(crate) Rc<RefCell<Object_>>);

pub(crate) type WeakObj = Weak<RefCell<Object_>>;

struct Range_ {
    serial: S<NodeSerialFixedRange>,
    /// Relative to serial
    start: Coord,
    avail: Coord,
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
            }),
            rust_root: new_s(NodeRustObj {
                id: schema2.take_id(),
                type_name: name.ident(),
                fields: vec![],
            }),
            rust_const_roots: vec![],
            updeps: vec![],
        })));
        schema2.objects.entry(name).or_insert_with(Vec::new).push(out);
        return out;
    }

    pub fn fixed_range(&self, bytes: usize) -> Range {
        let self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let range = new_s(NodeSerialFixedRange {
            scope: Rc::downgrade(&self.0),
            id: schema.take_id(),
            serial_before: root.children.last().copied(),
            serial: self2.serial_root.into(),
            len_bytes: bytes,
            sub_ranges: vec![],
            rust: None,
        });
        root.children.push(range.into());
        return Range(Rc::new(RefCell::new(Range_ {
            serial: range,
            start: Coord::zero(),
            avail: Coord::bytes(bytes),
        })));
    }

    pub fn dynamic_range(&self, len: S<NodeInt>) -> S<NodeDynamicRange> {
        let self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let n = new_s(NodeDynamicRange {
            scope: Rc::downgrade(&self.0),
            id: schema.take_id(),
            serial_before: root.children.last().copied(),
            serial: self2.serial_root.into(),
            serial_len: len,
            rust: None,
        });
        root.children.push(n.into());
        return n;
    }

    pub fn int(&mut self, range: Range, endian: Endian, signed: bool) -> S<NodeInt> {
        let mut range = range.0.as_ref().borrow_mut();
        let external = self.lift_single_nesting(range.serial.into());
        let mut self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let serial = range.serial.borrow_mut();
        if range.avail == Coord::zero() {
            panic!("Range has no space available");
        }
        let node = new_s(NodeInt::new(NodeIntArgs {
            scope: Rc::downgrade(&self.0),
            id: schema.take_id(),
            serial: range.serial,
            start: range.start,
            len: range.avail,
            signed: signed,
            endian: endian,
        }));
        serial.rust = Some(node.into());
        return node;
    }

    pub fn dynamic_array(&self, len: S<NodeInt>, obj_name: impl Display) -> (S<NodeDynamicArray>, Object) {
        let obj_name = obj_name.to_string();
        let external = self.lift_single_nesting(len.into());
        let self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema2 = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let element = Object::new(&schema, obj_name);
        let node = new_s(NodeDynamicArray {
            scope: Rc::downgrade(&self.0),
            id: schema2.take_id(),
            serial_before: root.children.last().copied(),
            serial_len: len,
            element: element.clone(),
            rust: None,
        });
        root.children.push(node.into());
        return (node, element);
    }

    pub fn enum_(&self, tag: Node, enum_name: impl Display) -> (S<NodeEnum>, Enum) {
        let enum_name = enum_name.to_string();
        let external = self.lift_single_nesting(tag);
        let self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema2 = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let node = new_s(NodeEnum {
            scope: Rc::downgrade(&self.0),
            id: schema2.take_id(),
            type_name: enum_name.ident(),
            serial_before: root.children.last().copied(),
            serial_tag: tag,
            variants: vec![],
            rust: None,
        });
        schema2.enums.entry(enum_name).or_insert_with(Vec::new).push(node);
        root.children.push(node.into());
        return (node, Enum {
            schema: schema.clone(),
            obj: self.clone(),
            enum_: node,
        });
    }

    pub fn option(&self, switch: Node, obj_name: impl Display) -> (S<NodeOption>, Object) {
        let external = self.lift_single_nesting(switch.into());
        let self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema2 = schema.as_ref().borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let element = Object::new(&schema, obj_name.to_string());
        let node = new_s(NodeOption {
            scope: Rc::downgrade(&self.0),
            id: schema2.take_id(),
            serial_before: root.children.last().copied(),
            serial_switch: switch,
            element: element.clone(),
            rust: None,
        });
        element.0.borrow_mut().nesting_parent = NestingParent::Option(node, self.clone());
        root.children.push(node.into());
        return (node, element);
    }

    /// Returns the parent node of self at the same level graph as the serial node it
    /// depends on if the serial node is at a different graph level
    fn lift_single_nesting(&self, val: Node) -> Option<Node> {
        // Need to
        //
        // 1. Replace serial source link to self with a link to the parent at that level (+ vice
        //    versa)
        //
        // 2. Replace self link to serial source with effective link to this level's serial root
        //    (+ vice versa)
        let mut at = self.0.clone();
        let mut out = None;
        loop {
            // # Confirm obj-extra deps are singly-mapped (not in an array)
            if val.scope_ptr() == at.as_ptr() {
                // # Not external, always ok
                return out;
            }
            match self.0.borrow().nesting_parent {
                NestingParent::None => {
                    // # External, but not found through single-mapped parent chain
                    panic!("Field data isn't in any nesting parent of field object");
                },
                NestingParent::Enum(parent, parent_parent) => {
                    // # External, lift dep to parent and leave checking to parent.
                    at.as_ref().borrow_mut().updeps.push(val);
                    out = Some(parent.into());
                    at = parent_parent.0.clone();
                },
                NestingParent::Option(parent, parent_parent) => {
                    // # External, lift dep to parent and leave checking to parent.
                    at.as_ref().borrow_mut().updeps.push(val);
                    out = Some(parent.into());
                    at = parent_parent.0.clone();
                },
            }
        }
    }

    pub fn rust_const_int(&self, serial: impl Into<Node>, value: TokenStream) {
        let serial = serial.into();
        let external = self.lift_single_nesting(serial);
        let mut self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let const_ = new_s(NodeRustConst {
            id: schema.take_id(),
            serial: RedirectRef {
                primary: serial,
                redirect: external,
            },
            expect: value,
        });
        self2.rust_const_roots.push(const_);
        serial.set_rust(|| match external {
            Some(parent) => parent.into(),
            None => const_.into(),
        });
    }

    pub fn rust_field(&self, serial: impl Into<Node>, name: impl IdentFragment) {
        let serial = serial.into();
        let external = self.lift_single_nesting(serial);
        let self2 = self.0.as_ref().borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.as_ref().borrow_mut();
        let field = new_s(NodeRustField {
            id: schema.take_id(),
            field_name: name,
            serial,
            value_external: external.is_some(),
            obj: self2.rust_root,
        });
        self2.rust_root.borrow_mut().fields.push(field);
        serial.set_rust(|| match external {
            Some(parent) => parent.into(),
            None => field.into(),
        });
    }
}

impl Range {
    pub fn subrange(&self, bytes: usize, bits: usize) -> Range {
        let mut self2 = self.0.as_ref().borrow_mut();
        let using = Coord {
            bytes: bytes,
            bits: bits,
        };
        if using.bits > 8 {
            panic!("Bits should be in excess of bytes (i.e. < 8), but got argument of {}b", using.bits);
        }
        if using > self2.avail {
            panic!("Range has {} available, but subrange consumes {}", self2.avail, using);
        }
        let out = Range(Rc::new(RefCell::new(Range_ {
            serial: self2.serial,
            start: self2.start,
            avail: using,
        })));
        self2.start += using;
        self2.avail -= using;
        return out;
    }
}

impl Enum {
    pub fn variant(&self, variant_name: impl IdentFragment, obj_name: impl Display, tag: TokenStream) -> Object {
        let enum_ = self.enum_.borrow_mut();
        let element = Object::new(&self.schema, obj_name.to_string());
        enum_.variants.push(EnumVariant {
            var_name: variant_name.ident(),
            tag: tag,
            element: element,
        });
        element.0.borrow_mut().nesting_parent = NestingParent::Enum(self.enum_, self.obj.clone());
        return element;
    }
}
