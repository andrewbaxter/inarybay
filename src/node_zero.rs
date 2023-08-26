use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    node::{
        Node,
        RedirectRef,
        NodeMethods,
        ToDep,
    },
    util::ToIdent,
};

pub(crate) struct NodeZero {
    pub(crate) id: String,
    // Bytes (fixed or ?)
    pub(crate) serial: RedirectRef<Node, Node>,
}

impl NodeMethods for NodeZero {
    fn read_deps(&self) -> Vec<Node> {
        return self.serial.dep();
    }

    fn generate_read(&self) -> TokenStream {
        let source_ident = self.serial.primary.id().ident();
        return quote!{
            for b in #source_ident {
                if b != 0 {
                    return Err("Not all zero TODO");
                }
            }
        };
    }

    fn write_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_write(&self) -> TokenStream {
        // All buffers are 0-initialized anyway, do nothing
        return quote!();
    }
}
