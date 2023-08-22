use crate::{
    util::S,
    node::Node,
    node_int::NodeInt,
    object::Object,
};

pub(crate) struct NodeArray {
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial_len: S<NodeInt>,
    pub(crate) element: Object,
    pub(crate) rust: Option<Node>,
}
