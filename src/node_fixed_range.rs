use itertools::Itertools;
use proc_macro2::TokenStream;
use crate::{
    node::Node,
    util::{
        S,
        RedirectRef,
    },
    object::WeakObj,
};

pub(crate) struct NodeSerialFixedRange {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    // NodeSerial or NodeRange
    pub(crate) serial: Node,
    pub(crate) len_bytes: usize,
    pub(crate) sub_ranges: Vec<S<NodeSerialFixedRange>>,
    pub(crate) rust: Vec<RedirectRef<Node, Node>>,
}

impl NodeSerialFixedRange {
    pub(crate) fn read_deps(&self) -> Vec<Node> {
        return vec![self.serial.clone()];
    }

    pub(crate) fn write_deps(&self) -> Vec<Node> {
        return self.rust.iter().map(|e| e.redirect.unwrap_or(e.primary)).unique_by(|e| e.id()).collect();
    }

    pub(crate) fn write_default(&self) -> TokenStream {
        unreachable!();
    }
}
