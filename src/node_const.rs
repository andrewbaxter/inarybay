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
        LateInit,
        ToIdent,
    },
    object::Object,
    derive_forward_node_methods,
    schema::GenerateContext,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeConstMut_ {
    pub(crate) serial: LateInit<RedirectRef<Node, Node>>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeConst_ {
    pub(crate) id: String,
    pub(crate) mut_: GcCell<NodeConstMut_>,
    #[unsafe_ignore_trace]
    pub(crate) expect: TokenStream,
}

impl NodeMethods for NodeConst_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().serial.dep();
    }

    fn generate_read(&self, gen_ctx: &GenerateContext) -> TokenStream {
        let source_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.id().ident();
        let expect = &self.expect;
        let to_err =
            gen_ctx.new_read_err(
                &self.id,
                quote!(format!("Expected magic value {:?} but got {:?}", #expect, #source_ident)),
            );
        return quote!{
            if #source_ident != #expect {
                return Err(#to_err);
            }
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let dest_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.id().ident();
        let expect = &self.expect;
        return quote!{
            let #dest_ident = #expect;
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

    fn rust_type(&self) -> TokenStream {
        unreachable!();
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NodeConst(pub(crate) Gc<NodeConst_>);

impl Into<Node> for NodeConst {
    fn into(self) -> Node {
        return Node(crate::node::Node_::Const(self));
    }
}

derive_forward_node_methods!(NodeConst);
