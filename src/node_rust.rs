use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    node::{
        Node,
        RedirectRef,
        NodeMethods,
        ToDep,
    },
    util::{
        S,
        ToIdent,
    },
};

pub(crate) struct NodeRustField {
    pub(crate) id: String,
    pub(crate) field_name: String,
    pub(crate) serial: RedirectRef<Node, Node>,
    pub(crate) obj: S<NodeRustObj>,
}

pub(crate) struct NodeRustObj {
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) fields: Vec<S<NodeRustField>>,
}

impl NodeMethods for NodeRustField {
    fn read_deps(&self) -> Vec<Node> {
        return self.serial.dep();
    }

    fn generate_read(&self) -> TokenStream {
        return quote!();
    }

    fn write_deps(&self) -> Vec<Node> {
        return self.obj.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let obj_ident = self.obj.borrow().id.ident();
        let dest_ident = self.serial.primary.id().ident();
        let field_ident = self.field_name.ident();
        return quote!{
            let #dest_ident = #obj_ident.#field_ident;
        };
    }
}

impl NodeMethods for NodeRustObj {
    fn read_deps(&self) -> Vec<Node> {
        return self.fields.dep();
    }

    fn generate_read(&self) -> TokenStream {
        let type_ident = self.type_name;
        let dest_ident = self.id.ident();
        let mut fields = vec![];
        for f in self.fields {
            let f = f.borrow();
            let field_ident = f.field_name;
            let value_ident = f.serial.primary.id().ident();
            fields.push(quote!{
                #field_ident: #value_ident,
            });
        }
        return quote!{
            #dest_ident = #type_ident {
                #(#fields) *
            };
        };
    }

    fn write_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_write(&self) -> TokenStream {
        return quote!();
    }
}
