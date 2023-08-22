use proc_macro2::TokenStream;
use crate::node::Node;

pub(crate) struct NodeRustConst {
    pub(crate) id: String,
    pub(crate) value: Node,
    pub(crate) expect: TokenStream,
}
