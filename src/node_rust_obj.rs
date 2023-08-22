use proc_macro2::Ident;
use crate::{
    node::Node,
    util::S,
};

pub(crate) struct NodeRustField {
    pub(crate) id: String,
    pub(crate) field_ident: Ident,
    pub(crate) value: Node,
    pub(crate) obj: S<NodeRustObj>,
}

pub(crate) struct NodeRustObj {
    pub(crate) id: String,
    pub(crate) type_ident: Ident,
    pub(crate) fields: Vec<S<NodeRustField>>,
}
