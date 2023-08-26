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

pub(crate) struct NodeConst {
    pub(crate) id: String,
    pub(crate) serial: RedirectRef<Node, Node>,
    pub(crate) expect: TokenStream,
}

impl NodeMethods for NodeConst {
    fn read_deps(&self) -> Vec<Node> {
        return self.serial.dep();
    }

    fn generate_read(&self) -> TokenStream {
        let source_ident = self.serial.primary.id().ident();
        let expect = self.expect;
        return quote!{
            if #source_ident != #expect {
                return Err("Magic mismatch at TODO");
            }
        };
    }

    fn write_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_write(&self) -> TokenStream {
        let dest_ident = self.serial.primary.id().ident();
        let expect = self.expect;
        return quote!{
            let #dest_ident = #expect;
        };
    }
}
