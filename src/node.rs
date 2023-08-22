use crate::{
    util::S,
    node_serial::NodeSerial,
    node_serial_range::NodeSerialRange,
    node_int::NodeInt,
    node_string::NodeString,
    node_array::NodeArray,
    node_option::NodeOption,
    node_rust_const::NodeRustConst,
    node_rust_obj::{
        NodeRustField,
        NodeRustObj,
    },
    node_enum::NodeEnum,
};

#[samevariant::samevariant(NodeSameVariant)]
pub(crate) enum Node_ {
    Serial(S<NodeSerial>),
    SerialRange(S<NodeSerialRange>),
    Int(S<NodeInt>),
    String(S<NodeString>),
    Array(S<NodeArray>),
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

impl From<S<NodeSerialRange>> for Node {
    fn from(value: S<NodeSerialRange>) -> Self {
        return Node(Box::leak(Box::new(Node_::SerialRange(value))));
    }
}

impl From<S<NodeInt>> for Node {
    fn from(value: S<NodeInt>) -> Self {
        return Node(Box::leak(Box::new(Node_::Int(value))));
    }
}

impl From<S<NodeString>> for Node {
    fn from(value: S<NodeString>) -> Self {
        return Node(Box::leak(Box::new(Node_::String(value))));
    }
}

impl From<S<NodeArray>> for Node {
    fn from(value: S<NodeArray>) -> Self {
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
    pub(crate) fn read_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_deps(&self) -> Vec<Node> { }

    pub(crate) fn id(&self) -> String { }
}
