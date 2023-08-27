use gc::{
    Finalize,
    Trace,
    Gc,
    GcCell,
};
use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    node::{
        Node,
        RedirectRef,
        NodeMethods,
        ToDep,
    },
    node_serial::{
        NodeSerialSegment,
    },
    util::{
        ToIdent,
        LateInit,
    },
    node_int::NodeInt,
    object::Object,
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeDynamicBytesMut_ {
    pub(crate) serial_len: LateInit<RedirectRef<NodeInt, Node>>,
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeDynamicBytes_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: NodeSerialSegment,
    pub(crate) mut_: GcCell<NodeDynamicBytesMut_>,
}

impl NodeMethods for NodeDynamicBytes_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.mut_.borrow().serial_len.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        let source_ident = self.serial.0.serial_root.0.id.ident();
        let source_len_ident = self.mut_.borrow().serial_len.as_ref().unwrap().primary.0.id.ident();
        let dest_ident = self.id.ident();
        return quote!{
            let mut #dest_ident = #source_ident.read_len(#source_len_ident) ?;
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let source_ident = self.id.ident();
        let dest_ident = self.serial.0.serial_root.0.id.ident();
        let serial_len = self.mut_.borrow().serial_len.as_ref().unwrap().primary.0.clone();
        let dest_len_ident = serial_len.id.ident();
        let dest_len_type = &serial_len.rust_type;
        return quote!{
            let #dest_len_ident = #source_ident.len() as #dest_len_type;
            let #dest_ident = #source_ident.as_bytes();
        };
    }

    fn set_rust(&self, rust: Node) {
        let mut mut_ = self.mut_.borrow_mut();
        if let Some(r) = &mut_.rust {
            if r.id() != rust.id() {
                panic!("Rust end of {} already connected to node {}", self.id, r.id());
            }
        }
        mut_.rust = Some(rust);
    }

    fn scope(&self) -> Object {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeDynamicBytes(pub(crate) Gc<NodeDynamicBytes_>);

impl Into<Node> for NodeDynamicBytes {
    fn into(self) -> Node {
        return Node(crate::node::Node_::DynamicBytes(self));
    }
}

derive_forward_node_methods!(NodeDynamicBytes);
