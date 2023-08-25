use proc_macro2::TokenStream;
use crate::{
    node::Node,
    util::RedirectRef,
};

pub(crate) struct NodeRustConst {
    pub(crate) id: String,
    pub(crate) value: RedirectRef<Node, Node>,
    pub(crate) expect: TokenStream,
}

impl NodeRustConst {
    pub(crate) fn read_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_deps(&self) -> Vec<Node> { }

    pub(crate) fn write_default(&self) -> TokenStream { }
}
