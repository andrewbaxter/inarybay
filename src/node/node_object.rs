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
use quote::{
    quote,
    ToTokens,
};
use crate::{
    node::{
        node::{
            Node,
            RedirectRef,
            NodeMethods,
            ToDep,
        },
    },
    util::{
        LateInit,
        ToIdent,
    },
    derive_forward_node_methods,
    schema::{
        GenerateContext,
    },
    scope::{
        Scope,
    },
};

use super::node::Node_;

#[derive(Trace, Finalize)]
pub(crate) struct NodeObjFieldMut_ {
    pub(crate) serial: LateInit<RedirectRef<Node, Node>>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeObjField_ {
    pub(crate) scope: Scope,
    pub(crate) id: String,
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
    pub(crate) field_name: String,
    #[unsafe_ignore_trace]
    pub(crate) field_name_ident: Ident,
    pub(crate) obj: NodeObj,
    pub(crate) mut_: GcCell<NodeObjFieldMut_>,
}

impl NodeMethods for NodeObjField_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().serial.dep();
    }

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        return quote!();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.obj.dep();
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let obj_ident = &self.obj.0.id_ident;
        let dest_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.id_ident();
        let field_ident = &self.field_name_ident;
        return quote!{
            #dest_ident = #obj_ident.#field_ident;
        };
    }

    fn set_rust(&self, _rust: Node) {
        unreachable!();
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
        unreachable!();
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct NodeObjField(pub(crate) Gc<NodeObjField_>);

impl Into<Node> for NodeObjField {
    fn into(self) -> Node {
        return Node(Node_::ObjField(self));
    }
}

derive_forward_node_methods!(NodeObjField);

#[derive(Trace, Finalize)]
pub(crate) struct NodeObjMut_ {
    pub(crate) fields: Vec<NodeObjField>,
    #[unsafe_ignore_trace]
    pub(crate) type_attrs: Vec<TokenStream>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeObj_ {
    pub(crate) scope: Scope,
    pub(crate) id: String,
    #[unsafe_ignore_trace]
    pub(crate) id_ident: Ident,
    pub(crate) type_name: String,
    #[unsafe_ignore_trace]
    pub(crate) type_name_ident: Ident,
    pub(crate) mut_: GcCell<NodeObjMut_>,
}

impl NodeMethods for NodeObj_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().fields.dep();
    }

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let type_ident = &self.type_name_ident;
        let dest_ident = &self.id_ident;
        let mut fields = vec![];
        for f in &self.mut_.borrow().fields {
            let field_ident = &f.0.field_name_ident;
            let value_ident = f.0.mut_.borrow().serial.as_ref().unwrap().primary.id_ident();
            fields.push(quote!{
                #field_ident: #value_ident,
            });
        }
        return quote!{
            #dest_ident = #type_ident {
                #(#fields) *
            };
        };
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return vec![];
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        return quote!();
    }

    fn set_rust(&self, _rust: Node) {
        unreachable!();
    }

    fn scope(&self) -> Scope {
        unreachable!();
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

/// This represents a rust struct.  Add fields with `field`.
#[derive(Clone, Trace, Finalize)]
pub struct NodeObj(pub(crate) Gc<NodeObj_>);

impl NodeObj {
    /// Add a structure prefix line like `#[...]` to the object definition.  Call like
    /// `o.add_type_attrs(quote!(#[derive(x,y,z)]))`.
    pub fn add_type_attrs(&self, attrs: TokenStream) {
        self.0.mut_.borrow_mut().type_attrs.push(attrs);
    }

    /// Store the read `serial` value in the object with the given `name`.  The type
    /// will match whatever type `serial` is.
    pub fn field(&self, name: impl Into<String>, serial: impl Into<Node>) {
        let name = name.into();
        let id = name.clone();
        self.0.scope.take_id(&id, None);
        let serial = serial.into();
        let rust = NodeObjField(Gc::new(NodeObjField_ {
            scope: self.0.scope.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            field_name: name.clone(),
            field_name_ident: name.ident().expect("Couldn't convert field name into a rust identifier"),
            obj: self.clone(),
            mut_: GcCell::new(NodeObjFieldMut_ { serial: None }),
        }));
        self.0.mut_.borrow_mut().fields.push(rust.clone());
        self
            .0
            .scope
            .lift_connect(
                &self.0.scope.get_ancestry_to(&serial),
                &serial,
                rust.clone().into(),
                &mut rust.0.mut_.borrow_mut().serial,
            );
    }
}

impl Into<Node> for NodeObj {
    fn into(self) -> Node {
        return Node(Node_::Obj(self));
    }
}

derive_forward_node_methods!(NodeObj);
