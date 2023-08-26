use std::{
    cell::RefCell,
    rc::Rc,
};
use enum_dispatch::enum_dispatch;
use proc_macro2::TokenStream;
use crate::{
    util::S,
    node_serial::{
        NodeSerial,
        NodeSerialSegment,
    },
    node_fixed_bytes::NodeFixedBytes,
    node_int::NodeInt,
    node_dynamic_bytes::NodeDynamicBytes,
    node_dynamic_array::NodeDynamicArray,
    node_const::NodeConst,
    node_rust::{
        NodeRustField,
        NodeRustObj,
    },
    node_enum::NodeEnum,
    object::Object_,
    node_zero::NodeZero,
};

#[enum_dispatch]
pub(crate) trait NodeMethods {
    fn read_deps(&self) -> Vec<Node>;
    fn generate_read(&self) -> TokenStream;
    fn write_deps(&self) -> Vec<Node>;
    fn generate_write(&self) -> TokenStream;
}

impl<T: NodeMethods> NodeMethods for S<T> {
    fn read_deps(&self) -> Vec<Node> {
        return self.read_deps();
    }

    fn generate_read(&self) -> TokenStream {
        return self.generate_read();
    }

    fn write_deps(&self) -> Vec<Node> {
        return self.write_deps();
    }

    fn generate_write(&self) -> TokenStream {
        return self.generate_write();
    }
}

#[samevariant::samevariant(NodeSameVariant)]
#[enum_dispatch(NodeMethods)]
pub(crate) enum Node_ {
    Serial(S<NodeSerial>),
    SerialSegment(S<NodeSerialSegment>),
    FixedBytes(S<NodeFixedBytes>),
    Int(S<NodeInt>),
    DynamicBytes(S<NodeDynamicBytes>),
    Array(S<NodeDynamicArray>),
    Enum(S<NodeEnum>),
    Const(S<NodeConst>),
    Zero(S<NodeZero>),
    RustField(S<NodeRustField>),
    RustObj(S<NodeRustObj>),
}

impl Node_ {
    pub(crate) fn typestr(&self) -> &'static str {
        match self {
            Node_::Serial(_) => unreachable!(),
            Node_::SerialSegment(_) => unreachable!(),
            Node_::FixedBytes(_) => "fixed bytes",
            Node_::Int(_) => "int",
            Node_::DynamicBytes(_) => "dynamic bytes",
            Node_::Array(_) => "array",
            Node_::Enum(_) => "enum",
            Node_::Const(_) => "const",
            Node_::Zero(_) => "zero",
            Node_::RustField(_) => unreachable!(),
            Node_::RustObj(_) => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Node(pub &'static Node_);

impl From<S<NodeSerial>> for Node {
    fn from(value: S<NodeSerial>) -> Self {
        return Node(Box::leak(Box::new(Node_::Serial(value))));
    }
}

impl From<S<NodeSerialSegment>> for Node {
    fn from(value: S<NodeSerialSegment>) -> Self {
        return Node(Box::leak(Box::new(Node_::SerialSegment(value))));
    }
}

impl From<S<NodeFixedBytes>> for Node {
    fn from(value: S<NodeFixedBytes>) -> Self {
        return Node(Box::leak(Box::new(Node_::FixedBytes(value))));
    }
}

impl From<S<NodeInt>> for Node {
    fn from(value: S<NodeInt>) -> Self {
        return Node(Box::leak(Box::new(Node_::Int(value))));
    }
}

impl From<S<NodeDynamicBytes>> for Node {
    fn from(value: S<NodeDynamicBytes>) -> Self {
        return Node(Box::leak(Box::new(Node_::DynamicBytes(value))));
    }
}

impl From<S<NodeDynamicArray>> for Node {
    fn from(value: S<NodeDynamicArray>) -> Self {
        return Node(Box::leak(Box::new(Node_::Array(value))));
    }
}

impl From<S<NodeEnum>> for Node {
    fn from(value: S<NodeEnum>) -> Self {
        return Node(Box::leak(Box::new(Node_::Enum(value))));
    }
}

impl From<S<NodeConst>> for Node {
    fn from(value: S<NodeConst>) -> Self {
        return Node(Box::leak(Box::new(Node_::Const(value))));
    }
}

impl From<S<NodeRustField>> for Node {
    fn from(value: S<NodeRustField>) -> Self {
        return Node(Box::leak(Box::new(Node_::RustField(value))));
    }
}

impl From<S<NodeRustObj>> for Node {
    fn from(value: S<NodeRustObj>) -> Self {
        return Node(Box::leak(Box::new(Node_::RustObj(value))));
    }
}

impl Node {
    pub(crate) fn set_rust(&self, rust: Node) { }

    pub(crate) fn scope(&self) -> Rc<RefCell<Object_>> {
        match &*self.0 {
            Node_::Serial(i) => unreachable!(),
            Node_::FixedBytes(i) => i.borrow().scope,
            Node_::Int(i) => i.borrow().scope,
            Node_::DynamicBytes(i) => i.borrow().scope,
            Node_::Array(i) => i.borrow().scope,
            Node_::Enum(i) => i.borrow().scope,
            Node_::Const(i) => unreachable!(),
            Node_::RustField(i) => unreachable!(),
            Node_::RustObj(i) => unreachable!(),
        }.upgrade().unwrap()
    }

    pub(crate) fn id(&self) -> String {
        match self.0 {
            Node_::Serial(n) => return n.borrow().id.clone(),
            Node_::FixedBytes(n) => return n.borrow().id.clone(),
            Node_::Int(n) => return n.borrow().id.clone(),
            Node_::DynamicBytes(n) => return n.borrow().id.clone(),
            Node_::Array(n) => return n.borrow().id.clone(),
            Node_::Enum(n) => return n.borrow().id.clone(),
            Node_::Const(n) => return n.borrow().id.clone(),
            Node_::RustField(n) => return n.borrow().id.clone(),
            Node_::RustObj(n) => return n.borrow().id.clone(),
        }
    }
}

pub(crate) trait ToDep {
    fn dep(&self) -> Vec<Node>;
}

impl<T: Into<Node>> ToDep for T {
    fn dep(&self) -> Vec<Node> {
        return vec![(*self).into()];
    }
}

impl<T: ToDep> ToDep for Option<T> {
    fn dep(&self) -> Vec<Node> {
        return self.iter().flat_map(|x| x.dep()).collect();
    }
}

impl<T: ToDep> ToDep for Vec<T> {
    fn dep(&self) -> Vec<Node> {
        return self.iter().flat_map(|x| x.dep()).collect();
    }
}

pub(crate) struct RedirectRef<T, U> {
    // Original ref
    pub(crate) primary: T,
    // Replacement for dep resolution
    pub(crate) redirect: Option<U>,
}

impl<T: Into<Node>, U: Into<Node>> ToDep for RedirectRef<T, U> {
    fn dep(&self) -> Vec<Node> {
        return vec![self.redirect.map(|x| x.into()).unwrap_or_else(|| self.primary.into())];
    }
}

impl<T, U> RedirectRef<T, U> {
    pub(crate) fn new(v: T) -> Self {
        Self {
            primary: v,
            redirect: None,
        }
    }
}
