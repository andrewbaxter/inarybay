use gc::{
    Finalize,
    Trace,
    GcCell,
    Gc,
};
use proc_macro2::{
    TokenStream,
    Ident,
};
use quote::ToTokens;
use crate::{
    util::{
        LateInit,
    },
    node::node::{
        Node,
        RedirectRef,
        NodeMethods,
        ToDep,
    },
    derive_forward_node_methods,
    schema::GenerateContext,
    scope::Scope,
};

use super::node::Node_;

#[derive(Trace, Finalize)]
pub(crate) struct NodeCustomMut_ {
    pub(crate) serial: Vec<LateInit<RedirectRef<Node, Node>>>,
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeCustom_ {
    pub(crate) scope: Scope,
    pub(crate) id: String,
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
    #[unsafe_ignore_trace]
    pub(crate) rust_type: TokenStream,
    #[unsafe_ignore_trace]
    pub(crate) read_code: Box<dyn Fn(&Vec<Ident>, &TokenStream) -> TokenStream>,
    #[unsafe_ignore_trace]
    pub(crate) write_code: Box<dyn Fn(&TokenStream, &Vec<Ident>) -> TokenStream>,
    pub(crate) mut_: GcCell<NodeCustomMut_>,
}

impl NodeMethods for NodeCustom_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().serial.dep();
    }

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let dest_ident = &self.id_ident;
        let mut source_idents = vec![];
        for serial in &self.mut_.borrow().serial {
            source_idents.push(serial.as_ref().unwrap().primary.id_ident());
        }
        return (self.read_code)(&source_idents, &dest_ident.into_token_stream());
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let mut dest_idents = vec![];
        for serial in &self.mut_.borrow().serial {
            dest_idents.push(serial.as_ref().unwrap().primary.id_ident());
        }
        return (self.write_code)(&self.id_ident.clone().into_token_stream(), &dest_idents);
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

    fn scope(&self) -> Scope {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }

    fn id_ident(&self) -> Ident {
        return self.id_ident.clone();
    }

    fn rust_type(&self) -> TokenStream {
        return self.rust_type.clone();
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeCustom(pub(crate) Gc<NodeCustom_>);

impl Into<Node> for NodeCustom {
    fn into(self) -> Node {
        return Node(Node_::Custom(self));
    }
}

derive_forward_node_methods!(NodeCustom);
