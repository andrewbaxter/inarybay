use std::collections::BTreeMap;
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
use quote::{
    quote,
    ToTokens,
};
use crate::{
    node::{
        Node,
        NodeMethods,
        ToDep,
    },
    util::{
        ToIdent,
    },
    object::{
        Object,
    },
    node_serial::NodeSerialSegment,
    derive_forward_node_methods,
    schema::GenerateContext,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeFixedRangeMut_ {
    pub(crate) rust: BTreeMap<String, Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeFixedRange_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    // NodeSerial or NodeRange
    pub(crate) serial: NodeSerialSegment,
    pub(crate) len_bytes: usize,
    pub(crate) mut_: GcCell<NodeFixedRangeMut_>,
}

pub(crate) fn generate_read_bytes(
    gen_ctx: &GenerateContext,
    node: &str,
    serial_ident: &Ident,
    ident: &Ident,
    len: TokenStream,
) -> TokenStream {
    let res_ident = "res__".ident();
    let do_await = gen_ctx.do_await(&res_ident);
    let raise_err = gen_ctx.do_raise_err(&res_ident, node);
    return quote!{
        let mut #ident = std:: vec:: Vec:: new();
        #ident.resize(#len as usize, 0u8);
        let #res_ident = #serial_ident.read_exact(#ident.as_mut_slice());
        //. .
        #do_await 
        //. .
        #raise_err 
        //. .
        drop(#res_ident);
    };
}

impl NodeMethods for NodeFixedRange_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial.dep());
        out.extend(self.serial_before.dep());
        return out;
    }

    fn generate_read(&self, gen_ctx: &GenerateContext) -> TokenStream {
        let serial_ident = self.serial.0.serial_root.0.id.ident();
        let ident = self.id.ident();
        let bytes = self.len_bytes;
        return generate_read_bytes(gen_ctx, &self.id, &serial_ident, &ident, bytes.to_token_stream());
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.mut_.borrow().rust.values().cloned());
        return out;
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let dest_ident = self.serial.0.id.ident();
        let source_ident = self.id.ident();
        return quote!{
            let #dest_ident = #source_ident;
        }
    }

    fn set_rust(&self, rust: Node) {
        self.mut_.borrow_mut().rust.insert(rust.id(), rust);
    }

    fn scope(&self) -> Object {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }
}

impl NodeFixedRange_ {
    pub(crate) fn generate_pre_write(&self) -> TokenStream {
        let dest_ident = self.id.ident();
        let len = self.len_bytes;
        return quote!{
            let mut #dest_ident = std:: vec:: Vec:: new();
            #dest_ident.resize(#len, 0u8);
        };
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NodeFixedRange(pub(crate) Gc<NodeFixedRange_>);

impl Into<Node> for NodeFixedRange {
    fn into(self) -> Node {
        return Node(crate::node::Node_::FixedRange(self));
    }
}

derive_forward_node_methods!(NodeFixedRange);
