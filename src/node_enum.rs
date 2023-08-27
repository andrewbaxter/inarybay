use std::collections::BTreeMap;
use gc::{
    Finalize,
    Trace,
};
use proc_macro2::{
    TokenStream,
};
use quote::quote;
use crate::{
    object::{
        Object,
    },
    node::{
        Node,
        NodeMethods,
        ToDep,
        RedirectRef,
        NodeMethods_,
    },
    node_serial::NodeSerialSegment,
    util::{
        S,
        ToIdent,
        LateInit,
    },
    schema::{
        generate_write,
        generate_read,
    },
    derive_forward_node_methods,
};

#[derive(Trace, Finalize)]
pub(crate) struct EnumVariant {
    pub(crate) var_name: String,
    #[unsafe_ignore_trace]
    pub(crate) tag: TokenStream,
    pub(crate) element: Object,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeEnum_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial_tag: LateInit<RedirectRef<Node, Node>>,
    pub(crate) serial: NodeSerialSegment,
    pub(crate) variants: Vec<EnumVariant>,
    pub(crate) rust: Option<Node>,
    pub(crate) external_deps: BTreeMap<String, Node>,
}

impl NodeMethods_ for NodeEnum_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.serial_tag.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        let type_ident = &self.type_name;
        let source_ident = self.serial.0.borrow().serial_root.0.borrow().id.ident();
        let source_tag_ident = self.serial_tag.as_ref().unwrap().primary.id().ident();
        let dest_ident = self.id.ident();
        let mut var_code = vec![];
        for v in &self.variants {
            let tag = &v.tag;
            let var_ident = &v.var_name;
            let elem = v.element.0.borrow();
            let elem_ident = elem.id.ident();
            let elem_code;
            if self.external_deps.is_empty() {
                let elem_type_ident = elem.rust_root.0.borrow().type_name.ident();
                elem_code = quote!{
                    let #elem_ident = #elem_type_ident:: read(#source_ident);
                }
            } else {
                elem_code = generate_read(&*elem);
            }
            var_code.push(quote!{
                #tag => {
                    #elem_code 
                    //. .
                    #dest_ident = #type_ident:: #var_ident(#elem_ident);
                },
            });
        }
        return quote!{
            let #dest_ident;
            match #source_tag_ident {
                #(#var_code) * 
                //. .
                _ => {
                    return Err("Unknown variant with tag {}", #source_tag_ident);
                }
            };
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.rust.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let enum_name = &self.type_name;
        let source_ident = self.id.ident();
        let dest_tag_ident = self.serial_tag.as_ref().unwrap().primary.id().ident();
        let dest_ident = self.serial.0.borrow().id.ident();
        let mut var_code = vec![];
        let mut anchor_external_deps = vec![];
        for dep in self.external_deps.values() {
            if dep.id() == self.scope.0.borrow().serial_root.0.borrow().id {
                continue;
            }
            let ident = dep.id().ident();
            anchor_external_deps.push(quote!{
                let #ident;
            });
        }
        for v in &self.variants {
            let tag = &v.tag;
            let variant_name = &v.var_name;
            let element = v.element.0.borrow();
            let elem_source_ident = element.rust_root.0.borrow().id.ident();
            let elem_dest_ident = element.serial_root.0.borrow().id.ident();
            let elem_code;
            if self.external_deps.is_empty() {
                elem_code = quote!{
                    let mut #elem_dest_ident = vec ![];
                    #elem_source_ident.write(& mut #elem_dest_ident);
                };
            } else {
                elem_code = generate_write(&*element);
            }
            var_code.push(quote!{
                #enum_name:: #variant_name(#elem_source_ident) => {
                    #dest_tag_ident = #tag;
                    #elem_code 
                    //. .
                    #dest_ident.extend(#elem_dest_ident);
                },
            });
        }
        return quote!{
            let #dest_tag_ident;
            let mut #dest_ident = vec ![];
            //. .
            #(#anchor_external_deps) * 
            //. .
            match #source_ident {
                #(#var_code) *
            };
        };
    }

    fn set_rust(&mut self, rust: Node) {
        if let Some(r) = &self.rust {
            if r.id() != rust.id() {
                panic!("Rust end of {} already connected to node {}", self.id, r.id());
            }
        }
        self.rust = Some(rust);
    }

    fn scope(&self) -> Object {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeEnum(pub(crate) S<NodeEnum_>);

impl Into<Node> for NodeEnum {
    fn into(self) -> Node {
        return Node(crate::node::Node_::Enum(self));
    }
}

derive_forward_node_methods!(NodeEnum);
