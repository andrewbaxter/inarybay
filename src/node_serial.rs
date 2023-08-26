use std::collections::BTreeMap;
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
};

pub(crate) struct NodeSerial {
    pub(crate) id: String,
    pub(crate) children: Vec<S<NodeSerialSegment>>,
    pub(crate) lifted_serial_deps: BTreeMap<String, Node>,
}

pub(crate) struct NodeSerialSegment {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial_root: S<NodeSerial>,
    pub(crate) serial_before: Option<S<NodeSerialSegment>>,
    pub(crate) rust: Option<Node>,
}

impl NodeMethods for NodeSerial {
    fn read_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_read(&self) -> TokenStream {
        return quote!();
    }

    fn write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.children.dep());
        out.extend(self.lifted_serial_deps.values().cloned());
        return out;
    }

    fn generate_write(&self) -> TokenStream {
        let mut code = vec![];
        let serial_ident = self.id.ident();
        for child in self.children {
            let child_ident = child.borrow().id.ident();
            code.push(quote!{
                #serial_ident.write(& #child_ident) ?;
            });
        }
        return quote!(#(#code) *);
    }
}

impl NodeMethods for NodeSerialSegment {
    fn read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_root.dep());
        out.extend(self.serial_before.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        return quote!();
    }

    fn write_deps(&self) -> Vec<Node> {
        let mut out: Vec<Node>;
        out.extend(self.serial_before.dep());
        out.extend(self.rust.dep());
        return out;
    }

    fn generate_write(&self) -> TokenStream {
        let serial_ident = self.serial_root.borrow().id.ident();
        let ident = self.id.ident();
        return quote!{
            #serial_ident.write(& #ident);
            drop(#ident);
        }
    }
}
