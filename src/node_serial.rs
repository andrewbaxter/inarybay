use std::collections::BTreeMap;
use gc::{
    Finalize,
    Trace,
};
use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    node::{
        Node,
        NodeMethods,
        ToDep,
        NodeMethods_,
    },
    util::{
        S,
        ToIdent,
    },
    object::Object,
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeSerial_ {
    pub(crate) id: String,
    pub(crate) children: Vec<NodeSerialSegment>,
    pub(crate) lifted_serial_deps: BTreeMap<String, Node>,
}

impl NodeMethods_ for NodeSerial_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_read(&self) -> TokenStream {
        return quote!();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.children.dep());
        out.extend(self.lifted_serial_deps.values().cloned());
        return out;
    }

    fn generate_write(&self) -> TokenStream {
        let mut code = vec![];
        let serial_ident = self.id.ident();
        for child in &self.children {
            let child_ident = child.0.borrow().id.ident();
            code.push(quote!{
                #serial_ident.write(& #child_ident) ?;
            });
        }
        return quote!(#(#code) *);
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
pub(crate) struct NodeSerial(pub(crate) S<NodeSerial_>);

impl Into<Node> for NodeSerial {
    fn into(self) -> Node {
        return Node(crate::node::Node_::Serial(self));
    }
}

derive_forward_node_methods!(NodeSerial);

#[derive(Trace, Finalize)]
pub(crate) struct NodeSerialSegment_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) serial_root: NodeSerial,
    pub(crate) serial_before: Option<NodeSerialSegment>,
    pub(crate) rust: Option<Node>,
}

impl NodeMethods_ for NodeSerialSegment_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_root.dep());
        out.extend(self.serial_before.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        return quote!();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.rust.dep());
        return out;
    }

    fn generate_write(&self) -> TokenStream {
        let serial_ident = self.serial_root.0.borrow().id.ident();
        let ident = self.id.ident();
        return quote!{
            #serial_ident.write(& #ident);
            drop(#ident);
        }
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
pub(crate) struct NodeSerialSegment(pub(crate) S<NodeSerialSegment_>);

impl Into<Node> for NodeSerialSegment {
    fn into(self) -> Node {
        return Node(crate::node::Node_::SerialSegment(self));
    }
}

derive_forward_node_methods!(NodeSerialSegment);
