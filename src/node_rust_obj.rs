use crate::{
    node::Node,
    util::{
        S,
        RedirectRef,
    },
};

pub(crate) struct NodeRustField {
    pub(crate) id: String,
    pub(crate) field_name: String,
    pub(crate) serial: RedirectRef<Node, Node>,
    pub(crate) obj: S<NodeRustObj>,
}

impl NodeRustField {
    pub(crate) fn read_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_default(&self) -> TokenStream { }
}

pub(crate) struct NodeRustObj {
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) fields: Vec<S<NodeRustField>>,
}

impl NodeRustObj {
    pub(crate) fn read_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_default(&self) -> TokenStream { }
}
