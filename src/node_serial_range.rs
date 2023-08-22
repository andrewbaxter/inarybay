use crate::{
    node::Node,
    util::S,
};

pub(crate) struct NodeSerialRange {
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    // NodeSerial or NodeRange
    pub(crate) serial: Node,
    pub(crate) len_bytes: usize,
    pub(crate) sub_ranges: Vec<S<NodeSerialRange>>,
    pub(crate) rust: Option<Node>,
}
