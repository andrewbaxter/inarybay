use proc_macro2::TokenStream;
use crate::node::Node;

pub(crate) struct NodeSerial {
    pub(crate) id: String,
    pub(crate) children: Vec<Node>,
}

impl NodeSerial {
    pub(crate) fn read_deps(&self) -> Vec<Node> {
        return vec![];
    }

    pub(crate) fn write_deps(&self) -> Vec<Node> {
        return self.children.clone();
    }

    pub(crate) fn write_default(&self) -> TokenStream {
        unreachable!();
    }
}
