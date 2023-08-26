use std::collections::BTreeMap;
use proc_macro2::{
    TokenStream,
};
use quote::quote;
use crate::{
    object::{
        Object,
        WeakObj,
    },
    node::{
        Node,
        NodeMethods,
        ToDep,
        RedirectRef,
    },
    node_serial::NodeSerialSegment,
    util::{
        S,
        ToIdent,
    },
    schema::{
        generate_write,
        generate_read,
    },
};

pub(crate) struct EnumVariant {
    pub(crate) var_name: String,
    pub(crate) tag: TokenStream,
    pub(crate) element: Object,
}

pub(crate) struct NodeEnum {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial_tag: RedirectRef<Node, Node>,
    pub(crate) serial: S<NodeSerialSegment>,
    pub(crate) variants: Vec<EnumVariant>,
    pub(crate) rust: Option<RedirectRef<Node, Node>>,
    pub(crate) lifted_serial_deps: BTreeMap<String, Node>,
}

impl NodeMethods for NodeEnum {
    fn read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.serial_tag.dep());
        out.extend(self.serial.dep());
        return out;
    }

    fn generate_read(&self) -> TokenStream {
        let type_ident = self.type_name;
        let source_ident = self.serial.borrow().serial_root.borrow().id.ident();
        let source_tag_ident = self.serial_tag.primary.id().ident();
        let dest_ident = self.id.ident();
        let mut var_code = vec![];
        for v in self.variants {
            let tag = v.tag;
            let var_ident = v.var_name;
            let elem = v.element.0.borrow();
            let elem_ident = elem.id.ident();
            let elem_code;
            if self.lifted_serial_deps.is_empty() {
                let elem_type_ident = elem.rust_root.borrow().type_name.ident();
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

    fn write_deps(&self) -> Vec<Node> {
        return self.rust.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let enum_name = self.type_name;
        let source_ident = self.id.ident();
        let dest_tag_ident = self.serial_tag.primary.id().ident();
        let dest_ident = self.serial.borrow().id.ident();
        let mut var_code = vec![];
        let mut all_external_deps = BTreeMap::new();
        for v in self.variants {
            for external_dep in &v.element.0.borrow().external_deps {
                all_external_deps.entry(external_dep.id()).or_insert(external_dep.clone());
            }
        }
        let mut anchor_external_deps = vec![];
        for dep in all_external_deps.values() {
            let ident = dep.id().ident();
            anchor_external_deps.push(quote!{
                let #ident;
            });
        }
        for v in self.variants {
            let tag = v.tag;
            let variant_name = v.var_name;
            let element = v.element.0.borrow();
            let elem_source_ident = element.rust_root.borrow().id.ident();
            let elem_dest_ident = element.serial_root.borrow().id.ident();
            let elem_code;
            if element.external_deps.is_empty() {
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
}
