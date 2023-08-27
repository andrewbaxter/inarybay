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
    util::{
        S,
        ToIdent,
        LateInit,
    },
    object::Object,
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeRustField_ {
    pub(crate) id: String,
    pub(crate) field_name: String,
    pub(crate) serial: LateInit<RedirectRef<Node, Node>>,
    pub(crate) obj: NodeRustObj,
}

impl NodeMethods_ for NodeRustField_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.serial.dep();
    }

    fn generate_read(&self) -> TokenStream {
        return quote!();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.obj.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let obj_ident = self.obj.0.borrow().id.ident();
        let dest_ident = self.serial.as_ref().unwrap().primary.id().ident();
        let field_ident = self.field_name.ident();
        return quote!{
            let #dest_ident = #obj_ident.#field_ident;
        };
    }

    fn set_rust(&mut self, _rust: Node) {
        unreachable!();
    }

    fn scope(&self) -> Object {
        unreachable!();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NodeRustField(pub(crate) S<NodeRustField_>);

impl Into<Node> for NodeRustField {
    fn into(self) -> Node {
        return Node(crate::node::Node_::RustField(self));
    }
}

derive_forward_node_methods!(NodeRustField);

#[derive(Trace, Finalize)]
pub(crate) struct NodeRustObj_ {
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) fields: Vec<NodeRustField>,
}

impl NodeMethods_ for NodeRustObj_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.fields.dep();
    }

    fn generate_read(&self) -> TokenStream {
        let type_ident = &self.type_name;
        let dest_ident = self.id.ident();
        let mut fields = vec![];
        for f in &self.fields {
            let f = f.0.borrow();
            let field_ident = &f.field_name;
            let value_ident = f.serial.as_ref().unwrap().primary.id().ident();
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

    fn gather_write_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_write(&self) -> TokenStream {
        return quote!();
    }

    fn set_rust(&mut self, _rust: Node) {
        unreachable!();
    }

    fn scope(&self) -> Object {
        unreachable!();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NodeRustObj(pub(crate) S<NodeRustObj_>);

impl Into<Node> for NodeRustObj {
    fn into(self) -> Node {
        return Node(crate::node::Node_::RustObj(self));
    }
}

derive_forward_node_methods!(NodeRustObj);
