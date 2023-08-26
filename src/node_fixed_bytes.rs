use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    node::{
        Node,
        NodeMethods,
        ToDep,
    },
    util::{
        S,
        ToIdent,
    },
    object::WeakObj,
    node_serial::NodeSerialSegment,
};

pub(crate) struct NodeFixedBytes {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    // NodeSerial or NodeRange
    pub(crate) serial: S<NodeSerialSegment>,
    pub(crate) len_bytes: usize,
    pub(crate) sub_ranges: Vec<S<NodeFixedBytes>>,
    pub(crate) rust: Option<Node>,
}

impl NodeMethods for NodeFixedBytes {
    fn read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial.dep());
        out.extend(self.serial_before.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        let serial_ident = self.serial.borrow().id.ident();
        let ident = self.id.ident();
        let bytes = self.len_bytes;
        return quote!{
            let mut #ident = #serial_ident.read_len(#bytes) ?;
        };
    }

    fn write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.rust.dep());
        out.extend(self.sub_ranges.dep());
        return out;
    }

    fn generate_write(&self) -> TokenStream {
        let dest_ident = self.serial.borrow().id.ident();
        let source_ident = self.id.ident();
        return quote!{
            let #dest_ident = #source_ident;
        }
    }
}

impl NodeFixedBytes {
    pub(crate) fn generate_pre_write(&self) -> TokenStream {
        let dest_ident = self.id.ident();
        let len = self.len_bytes;
        return quote!{
            let mut #dest_ident = std:: vec:: Vec:: new();
            #dest_ident.resize(#len, 0u8);
        };
    }
}
