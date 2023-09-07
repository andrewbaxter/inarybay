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
};
use crate::{
    node::{
        node::{
            Node,
            NodeMethods,
            ToDep,
        },
        node_serial::NodeSerialSegment,
    },
    util::{
        generate_basic_read,
        rust_type_bytes,
    },
    derive_forward_node_methods,
    schema::GenerateContext,
    scope::Scope,
};

use super::node::Node_;

#[derive(Trace, Finalize)]
pub(crate) struct NodeFixedRangeMut_ {
    pub(crate) rust: BTreeMap<String, Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeFixedRange_ {
    pub(crate) scope: Scope,
    pub(crate) id: String,
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
    pub(crate) serial_before: Option<Node>,
    // NodeSerial or NodeRange
    pub(crate) serial: NodeSerialSegment,
    pub(crate) len_bytes: usize,
    pub(crate) mut_: GcCell<NodeFixedRangeMut_>,
}

impl NodeMethods for NodeFixedRange_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial.dep());
        out.extend(self.serial_before.dep());
        return out;
    }

    fn generate_read(&self, gen_ctx: &GenerateContext) -> TokenStream {
        let len = self.len_bytes;
        return generate_basic_read(
            gen_ctx,
            &self.id,
            &self.id_ident,
            &self.serial.0.serial_root.0.id_ident,
            quote!(#len),
        );
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.mut_.borrow().rust.values().cloned());
        return out;
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let dest_ident = &self.serial.0.id_ident;
        let source_ident = &self.id_ident;
        return quote!{
            #dest_ident = #source_ident;
        }
    }

    fn set_rust(&self, rust: Node) {
        self.mut_.borrow_mut().rust.insert(rust.id(), rust);
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
        return rust_type_bytes();
    }
}

impl NodeFixedRange_ {
    pub(crate) fn generate_pre_write(&self) -> TokenStream {
        let dest_ident = &self.id_ident;
        let len = self.len_bytes;
        return quote!{
            #dest_ident = std:: vec:: Vec:: new();
            #dest_ident.resize(#len, 0u8);
        };
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NodeFixedRange(pub(crate) Gc<NodeFixedRange_>);

impl Into<Node> for NodeFixedRange {
    fn into(self) -> Node {
        return Node(Node_::FixedRange(self));
    }
}

derive_forward_node_methods!(NodeFixedRange);
