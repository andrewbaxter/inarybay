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
        LateInit,
        ToIdent,
        S,
    },
    object::Object,
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeConst_ {
    pub(crate) id: String,
    pub(crate) serial: LateInit<RedirectRef<Node, Node>>,
    #[unsafe_ignore_trace]
    pub(crate) expect: TokenStream,
}

impl NodeMethods_ for NodeConst_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.serial.dep();
    }

    fn generate_read(&self) -> TokenStream {
        let source_ident = self.serial.as_ref().unwrap().primary.id().ident();
        let expect = &self.expect;
        return quote!{
            if #source_ident != #expect {
                return Err("Magic mismatch at TODO");
            }
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_write(&self) -> TokenStream {
        let dest_ident = self.serial.as_ref().unwrap().primary.id().ident();
        let expect = &self.expect;
        return quote!{
            let #dest_ident = #expect;
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
pub(crate) struct NodeConst(pub(crate) S<NodeConst_>);

impl Into<Node> for NodeConst {
    fn into(self) -> Node {
        return Node(crate::node::Node_::Const(self));
    }
}

derive_forward_node_methods!(NodeConst);
