use std::collections::BTreeMap;
use gc::{
    Finalize,
    Trace,
    Gc,
    GcCell,
};
use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    node::{
        Node,
        NodeMethods,
        ToDep,
    },
    util::{
        ToIdent,
    },
    object::Object,
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeSerialMut_ {
    pub(crate) children: Vec<NodeSerialSegment>,
    pub(crate) lifted_serial_deps: BTreeMap<String, Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeSerial_ {
    pub(crate) id: String,
    pub(crate) mut_: GcCell<NodeSerialMut_>,
}

impl NodeMethods for NodeSerial_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_read(&self) -> TokenStream {
        return quote!();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.mut_.borrow().children.dep());
        out.extend(self.mut_.borrow().lifted_serial_deps.values().cloned());
        return out;
    }

    fn generate_write(&self) -> TokenStream {
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
pub(crate) struct NodeSerial(pub(crate) Gc<NodeSerial_>);

impl Into<Node> for NodeSerial {
    fn into(self) -> Node {
        return Node(crate::node::Node_::Serial(self));
    }
}

derive_forward_node_methods!(NodeSerial);

#[derive(Trace, Finalize)]
pub(crate) struct NodeSerialSegmentMut_ {
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeSerialSegment_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) serial_root: NodeSerial,
    pub(crate) serial_before: Option<NodeSerialSegment>,
    pub(crate) mut_: GcCell<NodeSerialSegmentMut_>,
}

impl NodeMethods for NodeSerialSegment_ {
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
        out.extend(self.mut_.borrow().rust.dep());
        return out;
    }

    fn generate_write(&self) -> TokenStream {
        let serial_ident = self.serial_root.0.id.ident();
        let ident = self.id.ident();
        return quote!{
            #serial_ident.write(& #ident);
            drop(#ident);
        }
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
pub(crate) struct NodeSerialSegment(pub(crate) Gc<NodeSerialSegment_>);

impl Into<Node> for NodeSerialSegment {
    fn into(self) -> Node {
        return Node(crate::node::Node_::SerialSegment(self));
    }
}

derive_forward_node_methods!(NodeSerialSegment);
