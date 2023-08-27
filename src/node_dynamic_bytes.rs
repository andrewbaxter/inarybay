use gc::{
    Finalize,
    Trace,
};
use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    node::{
        Node,
        RedirectRef,
        NodeMethods,
        ToDep,
        NodeMethods_,
    },
    node_serial::{
        NodeSerialSegment,
    },
    util::{
        S,
        ToIdent,
        LateInit,
    },
    node_int::NodeInt,
    object::Object,
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeDynamicBytes_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: NodeSerialSegment,
    pub(crate) serial_len: LateInit<RedirectRef<NodeInt, Node>>,
    pub(crate) rust: Option<Node>,
}

impl NodeMethods_ for NodeDynamicBytes_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.serial_len.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        let source_ident = self.serial.0.borrow().serial_root.0.borrow().id.ident();
        let source_len_ident = self.serial_len.as_ref().unwrap().primary.0.borrow().id.ident();
        let dest_ident = self.id.ident();
        return quote!{
            let mut #dest_ident = #source_ident.read_len(#source_len_ident) ?;
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.rust.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let source_ident = self.id.ident();
        let dest_ident = self.serial.0.borrow().serial_root.0.borrow().id.ident();
        let serial_len = self.serial_len.as_ref().unwrap().primary.0.borrow();
        let dest_len_ident = serial_len.id.ident();
        let dest_len_type = &serial_len.rust_type;
        return quote!{
            let #dest_len_ident = #source_ident.len() as #dest_len_type;
            let #dest_ident = #source_ident.as_bytes();
        };
    }

    fn set_rust(&mut self, rust: Node) {
        if let Some(r) = &self.rust {
            if r.id() != rust.id() {
                panic!("Rust end of {} already connected to node {}", self.id, r.id());
            }
        }
        self.rust = Some(rust);
    }

    fn scope(&self) -> Object {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeDynamicBytes(pub(crate) S<NodeDynamicBytes_>);

impl Into<Node> for NodeDynamicBytes {
    fn into(self) -> Node {
        return Node(crate::node::Node_::DynamicBytes(self));
    }
}

derive_forward_node_methods!(NodeDynamicBytes);
