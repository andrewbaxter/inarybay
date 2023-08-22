use crate::{
    node::Node,
    object::Object,
};

pub(crate) struct NodeOption {
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial_switch: Node,
    pub(crate) element: Object,
    pub(crate) rust: Option<Node>,
}
