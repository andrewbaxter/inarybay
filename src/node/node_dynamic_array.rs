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
    util::{
        LateInit,
    },
    node::{
        node::{
            Node,
            NodeMethods,
            RedirectRef,
            ToDep,
            Node_,
        },
        node_int::NodeInt,
        node_serial::NodeSerialSegment,
    },
    derive_forward_node_methods,
    schema::{
        GenerateContext,
        generate_write,
        generate_read,
    },
    scope::Scope,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeDynamicArrayMut_ {
    pub(crate) serial_len: LateInit<RedirectRef<NodeInt, Node>>,
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeDynamicArray_ {
    pub(crate) scope: Scope,
    pub(crate) id: String,
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: NodeSerialSegment,
    pub(crate) element: Scope,
    pub(crate) mut_: GcCell<NodeDynamicArrayMut_>,
}

impl NodeMethods for NodeDynamicArray_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.mut_.borrow().serial_len.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self, gen_ctx: &GenerateContext) -> TokenStream {
        let dest_ident = &self.id_ident;
        let source_len_ident = self.mut_.borrow().serial_len.as_ref().unwrap().primary.0.id_ident.clone();
        let elem_code = generate_read(gen_ctx, &self.element);
        let elem_dest_ident = self.element.get_rust_root().id_ident();
        let outer_serial_ident = &self.scope.0.serial_root.0.id_ident;
        let inner_serial_ident = &self.element.0.serial_root.0.id_ident;
        return quote!{
            let mut #dest_ident = vec ![];
            for _ in 0..#source_len_ident {
                let #inner_serial_ident =& mut * #outer_serial_ident;
                //. .
                #elem_code 
                //. .
                #dest_ident.push(#elem_dest_ident);
            }
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self, gen_ctx: &GenerateContext) -> TokenStream {
        let source_len_ident = self.id_ident();
        let len = self.mut_.borrow().serial_len.as_ref().unwrap().primary.0.clone();
        let dest_len_ident = len.id_ident();
        let dest_len_type = &len.rust_type;
        let dest_ident = self.serial.0.id_ident();
        let elem_code = generate_write(gen_ctx, &self.element);
        let elem_source_ident = self.element.get_rust_root().id_ident();
        let elem_dest_ident = &self.element.0.serial_root.0.id_ident;
        return quote!{
            #dest_len_ident = #source_len_ident.len() as #dest_len_type;
            #dest_ident = vec ![];
            for #elem_source_ident in #source_len_ident {
                let #elem_dest_ident =& mut #dest_ident;
                //. .
                #elem_code
            }
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
        let elem_type_ident = &self.element.0.mut_.borrow().rust_root.as_ref().unwrap().rust_type();
        return quote!(std:: vec:: Vec < #elem_type_ident >);
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeDynamicArray(pub(crate) Gc<NodeDynamicArray_>);

impl Into<Node> for NodeDynamicArray {
    fn into(self) -> Node {
        return Node(Node_::DynamicArray(self));
    }
}

derive_forward_node_methods!(NodeDynamicArray);
