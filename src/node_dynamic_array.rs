use crate::{
    util::{
        S,
        RedirectRef,
    },
    node::Node,
    node_int::NodeInt,
    object::{
        Object,
        WeakObj,
    },
};

pub(crate) struct NodeDynamicArray {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial_len: RedirectRef<S<NodeInt>, Node>,
    pub(crate) element: Object,
    pub(crate) rust: Option<RedirectRef<Node, Node>>,
}

impl NodeDynamicArray {
    pub(crate) fn read_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_default(&self) -> TokenStream { }
}
