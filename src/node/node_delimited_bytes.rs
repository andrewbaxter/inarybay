use gc::{
    Finalize,
    Trace,
    Gc,
    GcCell,
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
        node_serial::{
            NodeSerialSegment,
        },
    },
    util::{
        generate_delimited_read,
        rust_type_bytes,
    },
    derive_forward_node_methods,
    schema::GenerateContext,
    scope::Scope,
};

use super::node::Node_;

#[derive(Trace, Finalize)]
pub(crate) struct NodeDelimitedBytesMut_ {
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeDelimitedBytes_ {
    pub(crate) scope: Scope,
    pub(crate) id: String,
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: NodeSerialSegment,
    pub(crate) delim_len: usize,
    #[unsafe_ignore_trace]
    pub(crate) delim_bytes: TokenStream,
    pub(crate) mut_: GcCell<NodeDelimitedBytesMut_>,
}

impl NodeMethods for NodeDelimitedBytes_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self, gen_ctx: &GenerateContext) -> TokenStream {
        return generate_delimited_read(
            gen_ctx,
            &self.id,
            self.id_ident.clone(),
            self.serial.0.serial_root.0.id_ident(),
            &self.delim_bytes,
        );
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let source_ident = &self.id_ident;
        let dest_ident = self.serial.0.id_ident();
        let delim_len = &self.delim_len;
        let delim_bytes = &self.delim_bytes;
        return quote!{
            let mut #dest_ident = #source_ident.clone();
            #dest_ident.resize(#dest_ident.len() + #delim_len, 0u8);
            #dest_ident[#source_ident.len()..].copy_from_slice(#delim_bytes);
        };
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
        return rust_type_bytes();
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeDelimitedBytes(pub(crate) Gc<NodeDelimitedBytes_>);

impl Into<Node> for NodeDelimitedBytes {
    fn into(self) -> Node {
        return Node(Node_::DelimitedBytes(self));
    }
}

derive_forward_node_methods!(NodeDelimitedBytes);
