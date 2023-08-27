use std::{
    collections::{
        HashMap,
        BTreeMap,
        HashSet,
    },
};
use gc::{
    Finalize,
    Trace,
    GcCell,
    Gc,
};
use proc_macro2::{
    TokenStream,
};
use quote::{
    quote,
    ToTokens,
};
use crate::{
    object::{
        Object,
        Object_,
    },
    node::{
        Node,
        Node_,
        NodeSameVariant,
        NodeMethods,
    },
    util::{
        ToIdent,
    },
    node_enum::NodeEnum,
};

#[derive(Trace, Finalize)]
pub struct Schema_ {
    seen_ids: HashSet<String>,
    pub(crate) objects: BTreeMap<String, Vec<Object>>,
    pub(crate) enums: BTreeMap<String, Vec<NodeEnum>>,
}

impl Schema_ {
    pub(crate) fn take_id(&mut self, id: String) -> String {
        if !self.seen_ids.insert(id.clone()) {
            panic!("Id {} already used", id);
        }
        return id;
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct Schema(pub(crate) Gc<GcCell<Schema_>>);

impl Schema {
    pub fn new() -> Schema {
        return Schema(Gc::new(GcCell::new(Schema_ {
            seen_ids: HashSet::new(),
            objects: BTreeMap::new(),
            enums: BTreeMap::new(),
        })));
    }

    /// Define a new de/serializable object.
    pub fn object(&self, id: impl Into<String>, name: impl Into<String>) -> Object {
        return Object::new(id, &self, name.into());
    }

    /// Generate code for the described schema.
    pub fn generate(&self, read: bool, write: bool) -> TokenStream {
        let self2 = self.0.borrow();

        // Generate types
        let mut code = vec![];
        for (name, enums) in &self2.enums {
            let first = enums.first().unwrap();

            // Make sure all definitions are consistent
            for other in &enums[1..] {
                let mut other_variants = HashMap::new();
                for v in &other.0.borrow().variants {
                    other_variants.insert(v.var_name.to_string(), v.element.clone());
                }
                for first_v in &first.0.borrow().variants {
                    if let Some(other_v) = other_variants.remove(&first_v.var_name) {
                        let first_type = first_v.element.0.borrow().rust_root.0.borrow().type_name.clone();
                        let other_type = other_v.0.borrow().rust_root.0.borrow().type_name.clone();
                        if first_type != other_type {
                            panic!(
                                "Definitions of enum {} variant {} have mismatched types {} and {}",
                                name,
                                first_v.var_name,
                                first_type,
                                other_type
                            );
                        }
                    } else {
                        panic!("Some definitions of enum {} are missing variant {}", name, first_v.var_name);
                    }
                }
                for v in other_variants.keys() {
                    panic!("Some definitions of {} are missing variant {}", name, v);
                }
            }

            // Generate code
            let enum2 = first.0.borrow();
            let type_ident = &enum2.type_name;
            let mut variants = vec![];
            for v in &enum2.variants {
                let var_ident = &v.var_name;
                let var_type_ident = v.element.0.borrow().rust_root.0.borrow().type_name.clone();
                variants.push(quote!{
                    #var_ident(#var_type_ident),
                });
            }
            code.push(quote!{
                pub enum #type_ident {
                    #(#variants) *
                }
            });
        }
        for (name, objs) in &self2.objects {
            let first = objs.first().unwrap();

            // Make sure all definitions are consistent
            for other in &objs[1..] {
                let other2 = other.0.borrow();
                let other = other2.rust_root.0.borrow();
                let first = first.0.borrow();
                let mut other_fields = HashMap::new();
                for f in &other.fields {
                    let f = f.0.borrow();
                    let f_serial = f.serial.as_ref().unwrap();
                    other_fields.insert(
                        f.field_name.clone(),
                        f_serial.redirect.clone().unwrap_or_else(|| f_serial.primary.clone()),
                    );
                }
                for first_f in &first.rust_root.0.borrow().fields {
                    let first_f = first_f.0.borrow();
                    if let Some(other_f_value) = other_fields.remove(&first_f.field_name) {
                        let first_f_serial = first_f.serial.as_ref().unwrap();
                        match NodeSameVariant::pairs(&other_f_value.0, &first_f_serial.primary.0) {
                            NodeSameVariant::Int(l, r) => {
                                let l = l.0.borrow();
                                let r = r.0.borrow();
                                if l.rust_type.to_string() != r.rust_type.to_string() {
                                    panic!(
                                        "Definitions of {} field {} have mismatched rust types {} and {}",
                                        name,
                                        first_f.field_name,
                                        l.rust_type,
                                        r.rust_type
                                    );
                                }
                            },
                            NodeSameVariant::DynamicBytes(_, _) => { },
                            NodeSameVariant::DynamicArray(l, r) => {
                                let l_type = l.0.borrow().element.0.borrow().rust_root.0.borrow().type_name.clone();
                                let r_type = r.0.borrow().element.0.borrow().rust_root.0.borrow().type_name.clone();
                                if l_type != r_type {
                                    panic!(
                                        "Definitions of {} field {} have mismatched array element types {} and {}",
                                        name,
                                        first_f.field_name,
                                        l_type,
                                        r_type
                                    );
                                }
                            },
                            NodeSameVariant::Enum(l, r) => {
                                let l = l.0.borrow();
                                let r = r.0.borrow();
                                let l_type = &l.type_name;
                                let r_type = &r.type_name;
                                if l_type != r_type {
                                    panic!(
                                        "Definitions of {} field {} have mismatched enum inner types {} and {}",
                                        name,
                                        first_f.field_name,
                                        l_type,
                                        r_type
                                    );
                                }
                                let mut l_variants = HashMap::new();
                                for v in &l.variants {
                                    l_variants.insert(v.var_name.to_string(), v.element.clone());
                                }
                                for v in &r.variants {
                                    if let Some(l_v) = l_variants.remove(&v.var_name.to_string()) {
                                        let l_type = l_v.0.borrow().rust_root.0.borrow().type_name.clone();
                                        let r_type = v.element.0.borrow().rust_root.0.borrow().type_name.clone();
                                        if l_type != r_type {
                                            panic!(
                                                "Definitions of {} enum field {} variant {} have mismatched inner types {} and {}",
                                                name,
                                                first_f.field_name,
                                                v.var_name,
                                                l_type,
                                                r_type
                                            );
                                        }
                                    } else {
                                        panic!(
                                            "Definitions of {} enum field {} are missing variant {}",
                                            name,
                                            first_f.field_name,
                                            v.var_name
                                        );
                                    }
                                }
                            },
                            NodeSameVariant::Nonmatching(l, r) => {
                                panic!(
                                    "Some definitions of {} have value type {}, others {}",
                                    name,
                                    l.typestr(),
                                    r.typestr()
                                );
                            },
                            _ => unreachable!(),
                        }
                    } else {
                        panic!("Some definitions of {} are missing field {}", name, first_f.field_name);
                    }
                }
                for f in other_fields.keys() {
                    panic!("Some definitions of {} are missing field {}", name, f);
                }
            }
            let obj2 = first.0.borrow();
            let obj_obj = obj2.rust_root.0.borrow();

            // Generate code
            let type_ident = &obj_obj.type_name;
            let mut fields = vec![];
            for f in &obj_obj.fields {
                let f = f.0.borrow();
                let field_ident = &f.field_name;
                let field_type = match &f.serial.as_ref().unwrap().primary.0 {
                    Node_::Int(n) => n.0.borrow().rust_type.to_token_stream(),
                    Node_::DynamicBytes(_) => quote!(std:: vec:: Vec < u8 >),
                    Node_::DynamicArray(n) => {
                        let inner = n.0.borrow().element.0.borrow().rust_root.0.borrow().type_name.clone();
                        quote!(std:: vec:: Vec < #inner >)
                    },
                    Node_::Enum(n) => n.0.borrow().type_name.ident().into_token_stream(),
                    _ => unreachable!(),
                };
                fields.push(quote!{
                    pub #field_ident: #field_type,
                });
            }
            code.push(quote!{
                pub struct #type_ident {
                    #(#fields) *
                }
            });

            // Generate de/serialization methods for any objs with self-contained
            // serialization. (TODO Should maybe check for exactly one serialization format?).
            let root = first;
            let root = root.0.borrow();
            if root.has_external_deps {
                let root_obj = root.rust_root.0.borrow();
                let obj_ident = root_obj.id.ident();
                let serial_ident = root.serial_root.0.borrow().id.ident();
                let mut methods = vec![];
                if write {
                    let code = generate_write(&root);
                    methods.push(quote!{
                        pub fn write(&self, #serial_ident: std:: io:: Write) {
                            let #obj_ident = self;
                            #code
                        }
                    });
                }
                if read {
                    let code = generate_read(&root);
                    methods.push(quote!{
                        pub fn read(#serial_ident: std:: io:: Read) -> Self {
                            #code return #obj_ident;
                        }
                    });
                }
                let type_ident = &root_obj.type_name.ident();
                code.push(quote!(impl #type_ident {
                    #(#methods) *
                }));
            }
        }
        return quote!(#(#code) *);
    }
}

pub(crate) fn generate_read(obj: &Object_) -> TokenStream {
    let mut seen = HashSet::new();
    let mut stack: Vec<(Node, bool)> = vec![];
    for c in &obj.rust_const_roots {
        stack.push((c.clone().into(), false));
    }
    let mut code = vec![];
    while let Some((node, first_visit)) = stack.pop() {
        if first_visit {
            let node_id = node.id();
            if seen.contains(&node_id) {
                continue;
            }
            seen.insert(node_id);

            // Visit deps, then visit this node again
            stack.push((node.clone(), true));
            for dep in node.0.gather_read_deps() {
                stack.push((dep, false));
            }
        } else {
            // Post-deps, now do main processing
            code.push(node.0.generate_read());
        }
    }
    return quote!(#(#code) *);
}

pub(crate) fn generate_write(obj: &Object_) -> TokenStream {
    // Read from own id ident, write to serial-side id idents (except serial
    // segment/serial root)
    let mut seen = HashSet::new();
    let mut stack: Vec<(Node, bool)> = vec![];
    stack.push((obj.serial_root.clone().into(), false));
    let mut code = vec![];
    while let Some((node, first_visit)) = stack.pop() {
        if first_visit {
            let node_id = node.id();
            if seen.contains(&node_id) {
                continue;
            }
            seen.insert(node_id);

            // Prep before visiting deps
            match &node.0 {
                Node_::FixedRange(n) => {
                    code.push(n.0.borrow().generate_pre_write());
                },
                _ => { },
            }

            // Visit deps, then visit this node again
            stack.push((node.clone(), true));
            for dep in node.0.gather_write_deps() {
                stack.push((dep, false));
            }
        } else {
            // Post-deps, now do main processing
            code.push(node.0.generate_write());
        }
    }
    return quote!(#(#code) *);
}
