use proc_macro2::TokenStream;
use crate::{
    util::S,
    node_serial::NodeSerial,
    node_fixed_range::NodeSerialFixedRange,
    node_int::NodeInt,
    node_dynamic_range::NodeDynamicRange,
    node_dynamic_array::NodeDynamicArray,
    node_option::NodeOption,
    node_rust_const::NodeRustConst,
    node_rust_obj::{
        NodeRustField,
        NodeRustObj,
    },
    node_enum::NodeEnum,
    object::Object_,
};

#[samevariant::samevariant(NodeSameVariant)]
pub(crate) enum Node_ {
    Serial(S<NodeSerial>),
    FixedRange(S<NodeSerialFixedRange>),
    Int(S<NodeInt>),
    DynamicRange(S<NodeDynamicRange>),
    Array(S<NodeDynamicArray>),
    Enum(S<NodeEnum>),
    Option(S<NodeOption>),
    Const(S<NodeRustConst>),
    RustField(S<NodeRustField>),
    RustObj(S<NodeRustObj>),
}

impl Node_ {
    pub(crate) fn typestr(&self) -> String { }
}

#[derive(Clone, Copy)]
pub(crate) struct Node(pub &'static Node_);

impl From<S<NodeSerial>> for Node {
    fn from(value: S<NodeSerial>) -> Self {
        return Node(Box::leak(Box::new(Node_::Serial(value))));
    }
}

impl From<S<NodeSerialFixedRange>> for Node {
    fn from(value: S<NodeSerialFixedRange>) -> Self {
        return Node(Box::leak(Box::new(Node_::FixedRange(value))));
    }
}

impl From<S<NodeInt>> for Node {
    fn from(value: S<NodeInt>) -> Self {
        return Node(Box::leak(Box::new(Node_::Int(value))));
    }
}

impl From<S<NodeDynamicRange>> for Node {
    fn from(value: S<NodeDynamicRange>) -> Self {
        return Node(Box::leak(Box::new(Node_::DynamicRange(value))));
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

impl From<S<NodeOption>> for Node {
    fn from(value: S<NodeOption>) -> Self {
        return Node(Box::leak(Box::new(Node_::Option(value))));
    }
}

impl From<S<NodeRustConst>> for Node {
    fn from(value: S<NodeRustConst>) -> Self {
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
    pub(crate) fn set_rust(&self, supplier: impl FnOnce() -> Node) { }

    pub(crate) fn scope_ptr(&self) -> *const Object_ {
        match &*self.0 {
            Node_::Serial(i) => unreachable!(),
            Node_::FixedRange(i) => i.borrow().scope.upgrade().unwrap().as_ptr(),
            Node_::Int(i) => i.borrow().scope.upgrade().unwrap().as_ptr(),
            Node_::DynamicRange(i) => i.borrow().scope.upgrade().unwrap().as_ptr(),
            Node_::Array(i) => i.borrow().scope.upgrade().unwrap().as_ptr(),
            Node_::Enum(i) => i.borrow().scope.upgrade().unwrap().as_ptr(),
            Node_::Option(i) => i.borrow().scope.upgrade().unwrap().as_ptr(),
            Node_::Const(i) => unreachable!(),
            Node_::RustField(i) => unreachable!(),
            Node_::RustObj(i) => unreachable!(),
        }
    }

    pub(crate) fn read_deps(&self) -> Vec<Node> {
        match self.0 {
            Node_::Serial(n) => return n.borrow().read_deps(),
            Node_::FixedRange(n) => return n.borrow().read_deps(),
            Node_::Int(n) => return n.borrow().read_deps(),
            Node_::DynamicRange(n) => return n.borrow().read_deps(),
            Node_::Array(n) => return n.borrow().read_deps(),
            Node_::Enum(n) => return n.borrow().read_deps(),
            Node_::Option(n) => return n.borrow().read_deps(),
            Node_::Const(n) => return n.borrow().read_deps(),
            Node_::RustField(n) => return n.borrow().read_deps(),
            Node_::RustObj(n) => return n.borrow().read_deps(),
        }
    }

    pub(crate) fn write_deps(&self) -> Vec<Node> {
        match self.0 {
            Node_::Serial(n) => return n.borrow().write_deps(),
            Node_::FixedRange(n) => return n.borrow().write_deps(),
            Node_::Int(n) => return n.borrow().write_deps(),
            Node_::DynamicRange(n) => return n.borrow().write_deps(),
            Node_::Array(n) => return n.borrow().write_deps(),
            Node_::Enum(n) => return n.borrow().write_deps(),
            Node_::Option(n) => return n.borrow().write_deps(),
            Node_::Const(n) => return n.borrow().write_deps(),
            Node_::RustField(n) => return n.borrow().write_deps(),
            Node_::RustObj(n) => return n.borrow().write_deps(),
        }
    }

    pub(crate) fn write_default(&self) -> TokenStream { }

    pub(crate) fn id(&self) -> String {
        match self.0 {
            Node_::Serial(n) => return n.borrow().id.clone(),
            Node_::FixedRange(n) => return n.borrow().id.clone(),
            Node_::Int(n) => return n.borrow().id.clone(),
            Node_::DynamicRange(n) => return n.borrow().id.clone(),
            Node_::Array(n) => return n.borrow().id.clone(),
            Node_::Enum(n) => return n.borrow().id.clone(),
            Node_::Option(n) => return n.borrow().id.clone(),
            Node_::Const(n) => return n.borrow().id.clone(),
            Node_::RustField(n) => return n.borrow().id.clone(),
            Node_::RustObj(n) => return n.borrow().id.clone(),
        }
    }
}
