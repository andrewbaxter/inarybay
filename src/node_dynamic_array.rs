use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    util::{
        S,
        ToIdent,
    },
    node::{
        Node,
        NodeMethods,
        RedirectRef,
        ToDep,
    },
    node_int::NodeInt,
    object::{
        Object,
        WeakObj,
    },
    node_serial::NodeSerialSegment,
};

pub(crate) struct NodeDynamicArray {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial_len: S<NodeInt>,
    pub(crate) serial: S<NodeSerialSegment>,
    pub(crate) element: Object,
    pub(crate) rust: Option<RedirectRef<Node, Node>>,
}

impl NodeMethods for NodeDynamicArray {
    fn read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.serial_len.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        let dest_ident = self.id.ident();
        let source_len_ident = self.serial_len.borrow().id.ident();
        let source_ident = self.serial.borrow().serial_root.borrow().id.ident();
        let element = self.element.0.borrow();
        let elem_type_ident = element.rust_root.borrow().type_name.ident();
        return quote!{
            let mut #dest_ident = vec ![];
            for _ in 0..len_ident {
                #dest_ident.push(#elem_type_ident:: read(#source_ident));
            }
        };
    }

    fn write_deps(&self) -> Vec<Node> {
        return self.rust.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let source_ident = self.id.ident();
        let dest_len_ident = self.serial_len.borrow().id.ident();
        let dest_len_type = self.serial_len.borrow().rust_type;
        let dest_ident = self.serial.borrow().id.ident();
        let element = self.element.0.borrow();
        return quote!{
            let #dest_len_ident = #source_ident.len() as #dest_len_type;
            let mut #dest_ident = vec ![];
            for e in #source_ident {
                #dest_ident.extend(e.write());
            }
        };
    }
}
