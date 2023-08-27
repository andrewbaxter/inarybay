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

    fn generate_read(&self) -> TokenStream {
        let source_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.id().ident();
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
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NodeConst(pub(crate) Gc<NodeConst_>);

impl Into<Node> for NodeConst {
    fn into(self) -> Node {
        return Node(crate::node::Node_::Const(self));
    }
}

derive_forward_node_methods!(NodeConst);
