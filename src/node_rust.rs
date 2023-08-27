use gc::{
    Finalize,
    Trace,
    GcCell,
    Gc,
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
    util::{
        ToIdent,
        LateInit,
    },
    object::Object,
    derive_forward_node_methods,
    schema::GenerateContext,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeRustFieldMut_ {
    pub(crate) serial: LateInit<RedirectRef<Node, Node>>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeRustField_ {
    pub(crate) id: String,
    pub(crate) field_name: String,
    pub(crate) obj: NodeRustObj,
    pub(crate) mut_: GcCell<NodeRustFieldMut_>,
}

impl NodeMethods for NodeRustField_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().serial.dep();
    }

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        return quote!();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.obj.dep();
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let obj_ident = self.obj.0.id.ident();
        let dest_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.id().ident();
        let field_ident = self.field_name.ident();
        return quote!{
            let #dest_ident =& #obj_ident.#field_ident;
        };
    }

    fn set_rust(&self, _rust: Node) {
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
pub(crate) struct NodeRustField(pub(crate) Gc<NodeRustField_>);

impl Into<Node> for NodeRustField {
    fn into(self) -> Node {
        return Node(crate::node::Node_::RustField(self));
    }
}

derive_forward_node_methods!(NodeRustField);

#[derive(Trace, Finalize)]
pub(crate) struct NodeRustObjMut_ {
    pub(crate) fields: Vec<NodeRustField>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeRustObj_ {
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) mut_: GcCell<NodeRustObjMut_>,
}

impl NodeMethods for NodeRustObj_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().fields.dep();
    }

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let type_ident = &self.type_name.ident();
        let dest_ident = self.id.ident();
        let mut fields = vec![];
        for f in &self.mut_.borrow().fields {
            let field_ident = &f.0.field_name.ident();
            let value_ident = f.0.mut_.borrow().serial.as_ref().unwrap().primary.id().ident();
            fields.push(quote!{
                #field_ident: #value_ident,
            });
        }
        return quote!{
            let #dest_ident = #type_ident {
                #(#fields) *
            };
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        return quote!();
    }

    fn set_rust(&self, _rust: Node) {
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
pub(crate) struct NodeRustObj(pub(crate) Gc<NodeRustObj_>);

impl Into<Node> for NodeRustObj {
    fn into(self) -> Node {
        return Node(crate::node::Node_::RustObj(self));
    }
}

derive_forward_node_methods!(NodeRustObj);
