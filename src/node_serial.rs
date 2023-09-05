use std::collections::BTreeMap;
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
use quote::quote;
use crate::{
    node::{
        Node,
        NodeMethods,
        ToDep,
    },
    util::{
        ToIdent,
        generate_basic_write,
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
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
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

    fn id_ident(&self) -> Ident {
        return self.id_ident.clone();
    }

    fn rust_type(&self) -> TokenStream {
        unreachable!();
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
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
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
        if let Some(rust) = self.mut_.borrow().rust.as_ref() {
            match &rust.0 {
                crate::node::Node_::Align(align) => {
                    let align_ident = "align__".ident().unwrap();
                    let align_ident2 = "align__2".ident().unwrap();
                    let align_expr = align.0.align_expr();
                    let align_write = generate_basic_write(gen_ctx, &align_ident2, &self.serial_root.0.id_ident);
                    return quote!{
                        let #align_ident = #align_expr;
                        if #align_ident > 0 {
                            let mut #align_ident2 = vec ![];
                            #align_ident2.resize(#align_ident, 0u8);
                            #align_write
                        }
                    };
                },
                _ => { },
            }
        }
        return generate_basic_write(gen_ctx, &self.id_ident, &self.serial_root.0.id_ident);
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

    fn id_ident(&self) -> Ident {
        return self.id_ident.clone();
    }

    fn rust_type(&self) -> TokenStream {
        unreachable!();
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
