use crate::{
    node::Node,
    node_serial::NodeSerial,
    util::S,
    node_int::NodeInt,
};

pub(crate) struct NodeString {
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: S<NodeSerial>,
    pub(crate) serial_len: S<NodeInt>,
    pub(crate) rust: Option<Node>,
}
