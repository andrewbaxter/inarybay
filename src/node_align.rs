use gc::{
    Finalize,
    Trace,
    Gc,
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
        offset_ident,
    },
    object::Object,
    derive_forward_node_methods,
    schema::GenerateContext,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeAlign_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: NodeSerialSegment,
    pub(crate) multiple: usize,
}

impl NodeMethods for NodeAlign_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self, gen_ctx: &GenerateContext) -> TokenStream {
        let len_ident = "len__".ident();
        let read =
            generate_basic_read(
                gen_ctx,
                &self.id,
                self.id.ident(),
                self.serial.0.serial_root.0.id.ident(),
                quote!(#len_ident),
            );
        let multiple = self.multiple;
        let offset_ident = offset_ident();
        return quote!{
            let #len_ident = #multiple -(#offset_ident % #multiple);
            #read 
            //. .
            drop(#len_ident);
        }
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        return out;
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let dest_ident = self.serial.0.id.ident();
        let multiple = self.multiple;
        let offset_ident = offset_ident();
        return quote!{
            let mut #dest_ident = vec ![];
            #dest_ident.resize(#multiple -(#offset_ident % #multiple), 0u8);
        };
    }

    fn set_rust(&self, _rust: Node) {
        unreachable!();
    }

    fn scope(&self) -> Object {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }

    fn rust_type(&self) -> TokenStream {
        unreachable!();
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeAlign(pub(crate) Gc<NodeAlign_>);

impl Into<Node> for NodeAlign {
    fn into(self) -> Node {
        return Node(crate::node::Node_::Align(self));
    }
}

derive_forward_node_methods!(NodeAlign);
