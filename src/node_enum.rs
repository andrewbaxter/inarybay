use proc_macro2::Ident;
use crate::{
    object::Object,
    node::Node,
};

pub(crate) struct EnumVariant {
    pub(crate) var_ident: Ident,
    pub(crate) tag: Vec<u8>,
    pub(crate) element: Object,
}

pub(crate) struct NodeEnum {
    pub(crate) id: String,
    pub(crate) type_ident: Ident,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial_tag: Node,
    pub(crate) variants: Vec<EnumVariant>,
    pub(crate) rust: Option<Node>,
}
