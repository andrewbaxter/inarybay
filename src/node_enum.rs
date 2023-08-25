use std::collections::BTreeMap;
use proc_macro2::{
    TokenStream,
};
use crate::{
    object::{
        Object,
        WeakObj,
    },
    node::Node,
    util::RedirectRef,
};

pub(crate) struct EnumVariant {
    pub(crate) var_name: String,
    pub(crate) tag: TokenStream,
    pub(crate) element: Object,
}

pub(crate) struct NodeEnum {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial_tag: RedirectRef<Node, Node>,
    pub(crate) variants: Vec<EnumVariant>,
    pub(crate) rust: Option<RedirectRef<Node, Node>>,
    pub(crate) lifted_serial_deps: BTreeMap<String, Node>,
}
