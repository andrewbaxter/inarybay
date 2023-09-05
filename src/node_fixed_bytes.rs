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
use crate::{
    util::{
        LateInit,
    },
    node_fixed_range::NodeFixedRange,
    node::{
        Node,
        RedirectRef,
        NodeMethods,
        ToDep,
    },
    object::Object,
    derive_forward_node_methods,
    schema::GenerateContext,
};
use quote::{
    quote,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeFixedBytesMut_ {
    pub(crate) serial: LateInit<RedirectRef<NodeFixedRange, Node>>,
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeFixedBytes_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
    pub(crate) start: usize,
    pub(crate) len: usize,
    #[unsafe_ignore_trace]
    pub(crate) rust_type: TokenStream,
    pub(crate) mut_: GcCell<NodeFixedBytesMut_>,
}

impl NodeMethods for NodeFixedBytes_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().serial.dep();
    }

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let dest_ident = &self.id_ident;
        let source_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.0.id_ident();
        let serial_start = self.start;
        let serial_len = self.len;
        return quote!{
            #dest_ident = #source_ident[#serial_start..#serial_start + #serial_len].try_into().unwrap();
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let source_ident = &self.id_ident;
        let dest_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.0.id_ident();
        let serial_start = self.start;
        let serial_bytes = self.len;
        return quote!{
            #dest_ident[#serial_start..#serial_start + #serial_bytes].copy_from_slice(& #source_ident);
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

    fn scope(&self) -> Object {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }

    fn id_ident(&self) -> Ident {
        return self.id_ident.clone();
    }

    fn rust_type(&self) -> TokenStream {
        return self.rust_type.clone();
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeFixedBytes(pub(crate) Gc<NodeFixedBytes_>);

impl Into<Node> for NodeFixedBytes {
    fn into(self) -> Node {
        return Node(crate::node::Node_::FixedBytes(self));
    }
}

derive_forward_node_methods!(NodeFixedBytes);
