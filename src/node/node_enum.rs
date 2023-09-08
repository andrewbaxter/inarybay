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
use quote::{
    quote,
    ToTokens,
};
use crate::{
    node::{
        node::{
            Node,
            NodeMethods,
            ToDep,
            RedirectRef,
        },
        node_serial::NodeSerialSegment,
    },
    util::{
        LateInit,
        ToIdent,
    },
    schema::{
        generate_write,
        generate_read,
        GenerateContext,
    },
    derive_forward_node_methods,
    scope::{
        Scope,
        EscapableParentEnum,
        EscapableParent,
    },
};

use super::node::Node_;

#[derive(Trace, Finalize)]
pub(crate) struct NodeEnumDummyMut_ {
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeEnumDummy_ {
    pub(crate) scope: Scope,
    pub(crate) id: String,
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
    #[unsafe_ignore_trace]
    pub(crate) rust_type: TokenStream,
    pub(crate) mut_: GcCell<NodeEnumDummyMut_>,
}

impl NodeMethods for NodeEnumDummy_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        return quote!();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        return quote!();
    }

    fn set_rust(&self, rust: Node) {
        let mut mut_ = self.mut_.borrow_mut();
        if let Some(r) = &mut_.rust {
            if r.id() != rust.id() {
                panic!("Rust end of {} already connected to node {}", self.id, r.id());
            }
        }
        mut_.rust = Some(rust);
        self.scope.0.mut_.borrow_mut().has_external_deps = true;
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
        return self.rust_type.clone();
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NodeEnumDummy(pub(crate) Gc<NodeEnumDummy_>);

impl Into<Node> for NodeEnumDummy {
    fn into(self) -> Node {
        return Node(Node_::EnumDummy(self));
    }
}

derive_forward_node_methods!(NodeEnumDummy);

#[derive(Trace, Finalize)]
pub(crate) struct EnumVariant {
    pub(crate) var_name: String,
    #[unsafe_ignore_trace]
    pub(crate) var_name_ident: Ident,
    #[unsafe_ignore_trace]
    pub(crate) tag: TokenStream,
    pub(crate) element: Scope,
}

#[derive(Trace, Finalize)]
pub(crate) struct EnumDefaultVariant {
    pub(crate) var_name: String,
    #[unsafe_ignore_trace]
    pub(crate) var_name_ident: Ident,
    #[unsafe_ignore_trace]
    pub(crate) tag: Node,
    pub(crate) element: Scope,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeEnumMut_ {
    pub(crate) serial_tag: LateInit<RedirectRef<Node, Node>>,
    pub(crate) variants: Vec<EnumVariant>,
    pub(crate) default_variant: Option<EnumDefaultVariant>,
    pub(crate) rust: Option<Node>,
    pub(crate) external_deps: BTreeMap<String, Node>,
    #[unsafe_ignore_trace]
    pub(crate) type_attrs: Vec<TokenStream>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeEnum_ {
    pub(crate) scope: Scope,
    pub(crate) id: String,
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
    pub(crate) type_name: String,
    #[unsafe_ignore_trace]
    pub(crate) type_name_ident: Ident,
    pub(crate) serial_before: Option<Node>,
    pub(crate) serial: NodeSerialSegment,
    pub(crate) mut_: GcCell<NodeEnumMut_>,
}

impl NodeMethods for NodeEnum_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        let mut out = vec![];
        out.extend(self.serial_before.dep());
        out.extend(self.mut_.borrow().serial_tag.dep());
        out.extend(self.serial.dep());
        out.extend(self.mut_.borrow().external_deps.values().cloned());
        return out;
    }

    fn generate_read(&self, gen_ctx: &GenerateContext) -> TokenStream {
        let type_ident = &self.type_name_ident;
        let source_tag_ident = self.mut_.borrow().serial_tag.as_ref().unwrap().primary.id_ident();
        let dest_ident = &self.id_ident;
        let mut var_code = vec![];
        for v in &self.mut_.borrow().variants {
            let tag = &v.tag;
            let var_ident = &v.var_name_ident;
            let rust_root = v.element.get_rust_root();
            let elem_ident = rust_root.id_ident();
            let elem_code = generate_read(gen_ctx, &v.element);
            let outer_serial_ident = &self.scope.0.serial_root.0.id_ident;
            let inner_serial_ident = &v.element.0.serial_root.0.id_ident;
            var_code.push(quote!{
                #tag => {
                    let #inner_serial_ident = #outer_serial_ident;
                    #elem_code 
                    //. .
                    #dest_ident = #type_ident:: #var_ident(#elem_ident);
                },
            });
        }
        let default_code;
        if let Some(default_v) = &self.mut_.borrow().default_variant {
            let tag_ident = default_v.tag.id_ident();
            let var_ident = &default_v.var_name_ident;
            let rust_root = default_v.element.get_rust_root();
            let elem_ident = rust_root.id_ident();
            let elem_code = generate_read(gen_ctx, &default_v.element);
            let outer_serial_ident = &self.scope.0.serial_root.0.id_ident;
            let inner_serial_ident = &default_v.element.0.serial_root.0.id_ident;
            default_code = quote!{
                #tag_ident => {
                    let #inner_serial_ident = #outer_serial_ident;
                    #elem_code 
                    //. .
                    #dest_ident = #type_ident:: #var_ident(#elem_ident);
                }
            };
        } else {
            let err =
                gen_ctx.new_read_err(
                    &self.id,
                    "Unknown variant tag",
                    quote!(format!("Unknown variant tag {:?}", #source_tag_ident)),
                );
            default_code = quote!{
                _ => {
                    return Err(#err);
                }
            };
        }
        return quote!{
            match #source_tag_ident {
                #(#var_code) * 
                //. .
                #default_code
            };
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self, gen_ctx: &GenerateContext) -> TokenStream {
        let enum_name = &self.type_name_ident;
        let source_ident = &self.id_ident;
        let dest_tag_ident = &self.mut_.borrow().serial_tag.as_ref().unwrap().primary.id_ident();
        let dest_ident = &self.serial.0.id_ident;
        let mut var_code = vec![];
        for v in &self.mut_.borrow().variants {
            let tag = &v.tag;
            let variant_name = &v.var_name_ident;
            let elem_source_ident = v.element.get_rust_root().id_ident();
            let elem_dest_ident = &v.element.0.serial_root.0.id_ident;
            let elem_code = generate_write(gen_ctx, &v.element);
            var_code.push(quote!{
                #enum_name:: #variant_name(#elem_source_ident) => {
                    let mut #elem_dest_ident = std:: vec:: Vec::< u8 >:: new();
                    #elem_code 
                    //. .
                    #dest_ident.extend(#elem_dest_ident);
                    #dest_tag_ident = #tag;
                },
            });
        }
        if let Some(default_v) = &self.mut_.borrow().default_variant {
            let variant_name = &default_v.var_name_ident;
            let elem_source_ident = default_v.element.get_rust_root().id_ident();
            let elem_dest_ident = &default_v.element.0.serial_root.0.id_ident;
            let tag_ident = &default_v.tag.id_ident();
            let tag_type_ident = default_v.tag.rust_type();
            let write = generate_write(gen_ctx, &default_v.element);
            let elem_code = quote!{
                let mut #tag_ident: #tag_type_ident;
                #write 
                //. .
                #dest_tag_ident = #tag_ident;
            };
            var_code.push(quote!{
                #enum_name:: #variant_name(#elem_source_ident) => {
                    let mut #elem_dest_ident = std:: vec:: Vec::< u8 >:: new();
                    #elem_code 
                    //. .
                    #dest_ident.extend(#elem_dest_ident);
                },
            });
        }
        return quote!{
            #dest_ident = vec ![];
            //. .
            match #source_ident {
                #(#var_code) *
            };
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
        return self.type_name_ident.to_token_stream();
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeEnum(pub(crate) Gc<NodeEnum_>);

impl NodeEnum {
    /// Add a structure prefix line like `#[...]` to the object definition.  Call like
    /// `o.add_type_attrs(quote!(#[derive(x,y,z)]))`.
    pub fn add_type_attrs(&self, attrs: TokenStream) {
        self.0.mut_.borrow_mut().type_attrs.push(attrs);
    }

    /// Define a new variant in the enum.  `tag` is a literal that will be used in the
    /// match case for the tag value the enum reads.
    pub fn variant(&self, id: impl Into<String>, variant_name: impl Into<String>, tag: TokenStream) -> Scope {
        let id = id.into();
        let variant_name = variant_name.into();
        let element = Scope::new(id, &self.0.scope.0.schema, None);
        self.0.mut_.borrow_mut().variants.push(EnumVariant {
            var_name: variant_name.clone(),
            var_name_ident: variant_name.ident().expect("Couldn't convert variant name into a rust identifier"),
            tag: tag,
            element: element.clone(),
        });
        element.0.mut_.borrow_mut().escapable_parent = EscapableParent::Enum(EscapableParentEnum {
            enum_: self.clone(),
            variant_name: variant_name,
            parent: self.0.scope.clone(),
        });
        return element;
    }

    /// Define the default variant.  This returns the `Object` for defining the variant
    /// type, as well as a `Node` that represents the unmatched tag value which can be
    /// used by nodes within the variant type.  If you only need to read the value,
    /// this can be ignored.  If you need to round-trip, the value needs to be consumed
    /// so it can be output again upon serialization.
    pub fn default(&self, id: impl Into<String>, variant_name: impl Into<String>) -> (Scope, Node) {
        let id = id.into();
        let variant_name = variant_name.into();
        let element = Scope::new(id.clone(), &self.0.scope.0.schema, None);
        let dummy_id = format!("{}__tag", id);
        let dummy_rust_type = self.0.mut_.borrow().serial_tag.as_ref().unwrap().primary.rust_type();
        let dummy: Node = NodeEnumDummy(Gc::new(NodeEnumDummy_ {
            scope: element.clone(),
            id: dummy_id.clone(),
            id_ident: dummy_id.ident().expect("Couldn't convert id into a rust identifier"),
            rust_type: dummy_rust_type,
            mut_: GcCell::new(NodeEnumDummyMut_ { rust: None }),
        })).into();
        element.take_id(&dummy_id, None);
        let old = self.0.mut_.borrow_mut().default_variant.replace(EnumDefaultVariant {
            var_name: variant_name.clone(),
            var_name_ident: variant_name.ident().expect("Couldn't convert variant name into a rust identifier"),
            tag: dummy.clone(),
            element: element.clone(),
        });
        if old.is_some() {
            panic!("Default variant already set.");
        }
        {
            let mut el_mut = element.0.mut_.borrow_mut();
            el_mut.serial_extra_roots.push(dummy.clone());
            el_mut.escapable_parent = EscapableParent::Enum(EscapableParentEnum {
                enum_: self.clone(),
                variant_name: variant_name,
                parent: self.0.scope.clone(),
            });
        }
        return (element, dummy);
    }
}

impl Into<Node> for NodeEnum {
    fn into(self) -> Node {
        return Node(Node_::Enum(self));
    }
}

derive_forward_node_methods!(NodeEnum);
