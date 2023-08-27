use gc::{
    Finalize,
    Trace,
    Gc,
    GcCell,
};
use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    util::{
        ToIdent,
        LateInit,
    },
    node::{
        Node,
        NodeMethods,
        RedirectRef,
        ToDep,
        Node_,
    },
    node_int::NodeInt,
    object::{
        Object,
    },
    node_serial::NodeSerialSegment,
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct NodeDynamicArrayMut_ {
    pub(crate) serial_len: LateInit<RedirectRef<NodeInt, Node>>,
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeDynamicArray_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: NodeSerialSegment,
    pub(crate) element: Object,
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

    fn generate_read(&self) -> TokenStream {
        let dest_ident = self.id.ident();
        let source_ident = self.serial.0.serial_root.0.id.ident();
        let elem_type_ident = self.element.0.rust_root.0.type_name.ident();
        return quote!{
            let mut #dest_ident = vec ![];
            for _ in 0..len_ident {
                #dest_ident.push(#elem_type_ident:: read(#source_ident));
            }
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let source_ident = self.id.ident();
        let len = self.mut_.borrow().serial_len.as_ref().unwrap().primary.0.clone();
        let dest_len_ident = len.id.ident();
        let dest_len_type = &len.rust_type;
        let dest_ident = self.serial.0.id.ident();
        return quote!{
            let #dest_len_ident = #source_ident.len() as #dest_len_type;
            let mut #dest_ident = vec ![];
            for e in #source_ident {
                #dest_ident.extend(e.write());
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

    fn scope(&self) -> Object {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
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
