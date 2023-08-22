use std::{
    rc::{
        Rc,
        Weak,
    },
    cell::RefCell,
};
use crate::{
    util::{
        S,
        new_s,
        Coord,
        ToIdent,
    },
    node_serial::NodeSerial,
    node_rust_obj::{
        NodeRustObj,
        NodeRustField,
    },
    node_rust_const::NodeRustConst,
    schema::Schema_,
    node::{
        Node,
    },
    node_serial_range::NodeSerialRange,
    node_int::{
        NodeInt,
        NodeIntArgs,
        Endian,
    },
    node_string::NodeString,
    node_array::NodeArray,
    node_enum::NodeEnum,
    node_option::NodeOption,
};

pub(crate) struct Object_ {
    pub(crate) schema: Weak<RefCell<Schema_>>,
    pub(crate) parent: Option<Weak<RefCell<Object_>>>,
    pub(crate) id: String,
    pub(crate) serial_root: S<NodeSerial>,
    pub(crate) rust_root: S<NodeRustObj>,
    pub(crate) rust_const_roots: Vec<S<NodeRustConst>>,
    // Child dependencies lifted up to object
    pub(crate) updeps: Vec<Node>,
}

#[derive(Clone)]
pub(crate) struct Object(pub(crate) Rc<RefCell<Object_>>);

pub(crate) type WeakObj = Weak<RefCell<Object_>>;

struct Range {
    obj: Object,
    serial: S<NodeSerialRange>,
    /// Relative to serial
    start: Coord,
    avail: Coord,
}

impl Object {
    fn serial_range(&self, bytes: usize) -> Range {
        let self2 = self.0.borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let range = new_s(NodeSerialRange {
            id: schema.take_id(),
            serial_before: root.children.last().copied(),
            serial: self2.serial_root.into(),
            len_bytes: bytes,
            sub_ranges: vec![],
            rust: None,
        });
        root.children.push(range.into());
        return Range {
            obj: self.clone(),
            serial: range,
            start: Coord::zero(),
            avail: Coord::bytes(bytes),
        };
    }

    fn serial_string(&self, len: S<NodeInt>) -> S<NodeString> {
        let self2 = self.0.borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.borrow_mut();
        let root = self2.serial_root.borrow_mut();
        let n = new_s(NodeString {
            id: schema.take_id(),
            serial_before: root.children.last().copied(),
            serial: self2.serial_root.into(),
            serial_len: len,
            rust: None,
        });
        root.children.push(n.into());
        return n;
    }

    fn serial_array(&self, len: Node, obj_name: String) -> (S<NodeArray>, Object) { }

    fn serial_enum_(&self, tag: Node, enum_name: String) -> (S<NodeEnum>, EnumBuilder) { }

    fn serial_option(&self, switch: Node, obj_name: String) -> (S<NodeOption>, Object) { }

    fn rust_magic(&self, serial: Node, value: &[u8]) { }

    fn rust_field_int(&self, serial: S<NodeInt>, name: String) {
        let self2 = self.0.borrow_mut();
        let schema = self2.schema.upgrade().unwrap();
        let schema = schema.borrow_mut();
        let field = new_s(NodeRustField {
            id: schema.take_id(),
            field_ident: name.ident(),
            value: serial.into(),
            obj: self2.rust_root,
        });
        serial.borrow_mut().rust = Some(field.into());
        self2.rust_root.borrow_mut().fields.push(field);
    }
}

impl Range {
    pub fn int(&mut self, endian: Endian, signed: bool) -> S<NodeInt> {
        let serial = self.serial.borrow_mut();
        if !serial.sub_ranges.is_empty() {
            panic!(
                "Range is already used by subranges; if you want to use the remaining space, create a new subrange for that space first"
            );
        }
        if serial.rust.is_some() {
            panic!("Range is already consumed by another value");
        }
        let node = new_s(NodeInt::new(NodeIntArgs {
            scope: Rc::downgrade(&self.obj.0),
            id: self.obj.0.borrow().schema.upgrade().unwrap().borrow_mut().take_id(),
            serial: self.serial,
            start: self.start,
            len: self.avail,
            signed: signed,
            endian: endian,
        }));
        serial.rust = Some(node.into());
        return node;
    }

    pub fn subrange(&mut self, bytes: usize, bits: usize) -> Range {
        let using = Coord {
            bytes: bytes,
            bits: bits,
        };
        if using.bits > 8 {
            panic!("Bits should be in excess of bytes (i.e. < 8), but got argument of {}b", using.bits);
        }
        if using > self.avail {
            panic!("Range has {} available, but subrange consumes {}", self.avail, using);
        }
        let out = Range {
            obj: self.obj,
            serial: self.serial,
            start: self.start,
            avail: using,
        };
        self.start += using;
        self.avail -= using;
        return out;
    }
}
