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
    schema::GenerateContext,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeSerialMut_ {
    pub(crate) segments: Vec<NodeSerialSegment>,
    pub(crate) sub_segments: Vec<Node>,
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

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        return quote!();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.mut_.borrow().segments.dep());
        out.extend(self.mut_.borrow().lifted_serial_deps.values().cloned());
        return out;
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

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        return quote!();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.mut_.borrow().rust.dep());
        return out;
    }

    fn generate_write(&self, gen_ctx: &GenerateContext) -> TokenStream {
        let serial_ident = self.serial_root.0.id.ident();
        let ident = self.id.ident();
        let res_ident = "res__".ident();
        let do_await = gen_ctx.do_await(&res_ident);
        return quote!{
            let #res_ident = #serial_ident.write(& #ident);
            //. .
            #do_await 
            //. .
            let #res_ident = #res_ident ?;
            drop(#res_ident);
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
