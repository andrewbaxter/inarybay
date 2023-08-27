use std::collections::BTreeMap;
use gc::{
    Finalize,
    Trace,
};
use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    node::{
        Node,
        NodeMethods,
        ToDep,
        NodeMethods_,
    },
    util::{
        S,
        ToIdent,
    },
    object::{
        Object,
    },
    node_serial::NodeSerialSegment,
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeFixedBytes_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    // NodeSerial or NodeRange
    pub(crate) serial: NodeSerialSegment,
    pub(crate) len_bytes: usize,
    pub(crate) rust: BTreeMap<String, Node>,
}

impl NodeMethods_ for NodeFixedBytes_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial.dep());
        out.extend(self.serial_before.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        let serial_ident = self.serial.0.borrow().id.ident();
        let ident = self.id.ident();
        let bytes = self.len_bytes;
        return quote!{
            let mut #ident = #serial_ident.read_len(#bytes) ?;
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.rust.values().cloned());
        return out;
    }

    fn generate_write(&self) -> TokenStream {
        let dest_ident = self.serial.0.borrow().id.ident();
        let source_ident = self.id.ident();
        return quote!{
            let #dest_ident = #source_ident;
        }
    }

    fn set_rust(&mut self, rust: Node) {
        self.rust.insert(rust.id(), rust);
    }

    fn scope(&self) -> Object {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }
}

impl NodeFixedBytes_ {
    pub(crate) fn generate_pre_write(&self) -> TokenStream {
        let dest_ident = self.id.ident();
        let len = self.len_bytes;
        return quote!{
            let mut #dest_ident = std:: vec:: Vec:: new();
            #dest_ident.resize(#len, 0u8);
        };
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NodeFixedBytes(pub(crate) S<NodeFixedBytes_>);

impl Into<Node> for NodeFixedBytes {
    fn into(self) -> Node {
        return Node(crate::node::Node_::FixedBytes(self));
    }
}

derive_forward_node_methods!(NodeFixedBytes);
