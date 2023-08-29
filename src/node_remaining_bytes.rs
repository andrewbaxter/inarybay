use gc::{
    Finalize,
    Trace,
    Gc,
    GcCell,
};
use proc_macro2::TokenStream;
use quote::{
    quote,
};
use crate::{
    node::{
        Node,
        NodeMethods,
        ToDep,
    },
    node_serial::{
        NodeSerialSegment,
    },
    util::{
        ToIdent,
        generate_basic_read,
        rust_type_bytes,
    },
    object::Object,
    derive_forward_node_methods,
    schema::GenerateContext,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeSuffixBytesMut_ {
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeRemainingBytes_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: NodeSerialSegment,
    pub(crate) mut_: GcCell<NodeSuffixBytesMut_>,
}

impl NodeMethods for NodeRemainingBytes_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self, gen_ctx: &GenerateContext) -> TokenStream {
        return generate_basic_read(
            gen_ctx,
            &self.id,
            self.id.ident(),
            self.serial.0.serial_root.0.id.ident(),
            quote!(0),
        );
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let source_ident = self.id.ident();
        let dest_ident = self.serial.0.id.ident();
        return quote!{
            let #dest_ident = #source_ident;
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

    fn rust_type(&self) -> TokenStream {
        return rust_type_bytes();
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeRemainingBytes(pub(crate) Gc<NodeRemainingBytes_>);

impl Into<Node> for NodeRemainingBytes {
    fn into(self) -> Node {
        return Node(crate::node::Node_::RemainingBytes(self));
    }
}

derive_forward_node_methods!(NodeRemainingBytes);
