use crate::{
    node::Node,
    node_serial::NodeSerial,
    util::{
        S,
        RedirectRef,
    },
    node_int::NodeInt,
    object::WeakObj,
};

pub(crate) struct NodeDynamicRange {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: S<NodeSerial>,
    pub(crate) serial_len: RedirectRef<S<NodeInt>, Node>,
    pub(crate) rust: Vec<RedirectRef<Node, Node>>,
}

impl NodeDynamicRange {
    pub(crate) fn read_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_default(&self) -> TokenStream { }
}
