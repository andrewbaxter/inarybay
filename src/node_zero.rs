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
        ToIdent,
        S,
    },
    object::Object,
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeZero_ {
    pub(crate) id: String,
    // Bytes (fixed or ?)
    pub(crate) serial: RedirectRef<Node, Node>,
}

impl NodeMethods_ for NodeZero_ {
    fn gather_read_deps(&self) -> Vec<Node> {
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

    fn gather_write_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_write(&self) -> TokenStream {
        // All buffers are 0-initialized anyway, do nothing
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
pub(crate) struct NodeZero(pub(crate) S<NodeZero_>);

impl Into<Node> for NodeZero {
    fn into(self) -> Node {
        return Node(crate::node::Node_::Zero(self));
    }
}

derive_forward_node_methods!(NodeZero);
