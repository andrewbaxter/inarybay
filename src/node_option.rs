use std::collections::BTreeMap;
use crate::{
    node::Node,
    object::{
        Object,
        WeakObj,
    },
    util::RedirectRef,
};

pub(crate) struct NodeOption {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial_switch: RedirectRef<Node, Node>,
    pub(crate) element: Object,
    pub(crate) rust: Option<RedirectRef<Node, Node>>,
    pub(crate) lifted_serial_deps: BTreeMap<String, Node>,
}

impl NodeOption {
    pub(crate) fn read_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_default(&self) -> TokenStream { }
}
