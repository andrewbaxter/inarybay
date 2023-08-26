use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    node::{
        Node,
        RedirectRef,
        NodeMethods,
        ToDep,
    },
    node_serial::NodeSerial,
    util::{
        S,
        ToIdent,
    },
    node_int::NodeInt,
    object::WeakObj,
};

pub(crate) struct NodeDynamicBytes {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: S<NodeSerial>,
    pub(crate) serial_len: RedirectRef<S<NodeInt>, Node>,
    pub(crate) rust: Option<RedirectRef<Node, Node>>,
}

impl NodeMethods for NodeDynamicBytes {
    fn read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.serial_len.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        let source_ident = self.serial.borrow().id.ident();
        let source_len_ident = self.serial_len.primary.borrow().id.ident();
        let dest_ident = self.id.ident();
        return quote!{
            let mut #dest_ident = #source_ident.read_len(#source_len_ident) ?;
        };
    }

    fn write_deps(&self) -> Vec<Node> {
        return self.rust.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let source_ident = self.id.ident();
        let dest_ident = self.serial.borrow().id.ident();
        let dest_len_ident = self.serial_len.primary.borrow().id.ident();
        let dest_len_type = self.serial_len.primary.borrow().rust_type;
        return quote!{
            let #dest_len_ident = #source_ident.len() as #dest_len_type;
            let #dest_ident = #source_ident.as_bytes();
        };
    }
}
