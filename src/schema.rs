use std::{
    cell::{
        RefCell,
    },
    rc::{
        Rc,
    },
    collections::{
        HashMap,
        BTreeMap,
        HashSet,
    },
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
        S,
    },
    node_enum::NodeEnum,
};

pub struct Schema_ {
    next_id: usize,
    pub(crate) objects: BTreeMap<String, Vec<Object>>,
    pub(crate) enums: BTreeMap<String, Vec<S<NodeEnum>>>,
}

impl Schema_ {
    pub(crate) fn take_id(&mut self) -> String {
        let out = format!("x{}", self.next_id);
        self.next_id += 1;
        return out;
    }
}

pub struct Schema(Rc<RefCell<Schema_>>);

impl Schema {
    pub fn new() -> Schema {
        return Schema(Rc::new(RefCell::new(Schema_ {
            next_id: 0,
            objects: BTreeMap::new(),
            enums: BTreeMap::new(),
        })));
    }

    /// Define a new de/serializable object.
    pub fn object(&self, name: String) -> Object {
        return Object::new(&self.0, name);
    }

    /// Generate code for the described schema.
    pub fn generate(&self, read: bool, write: bool) -> TokenStream {
        let self2 = self.0.borrow();

        // Generate types
        let mut code = vec![];
        for (name, enums) in self2.enums {
            let mut first = enums.first().unwrap();

            // Make sure all definitions are consistent
            for other in &enums[1..] {
                let mut other_variants = HashMap::new();
                for v in other.borrow().variants {
                    other_variants.insert(v.var_name.to_string(), v.element.clone());
                }
                for first_v in first.borrow().variants {
                    if let Some(other_v) = other_variants.remove(&first_v.var_name) {
                        let first_type = first_v.element.0.borrow().rust_root.borrow().type_name;
                        let other_type = other_v.0.borrow().rust_root.borrow().type_name;
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
            let enum2 = first.borrow();
            let type_ident = enum2.type_name;
            let mut variants = vec![];
            for v in &enum2.variants {
                let var_ident = v.var_name;
                let var_type_ident = v.element.0.borrow().rust_root.borrow().type_name.clone();
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
        for (name, objs) in self2.objects {
            let mut first = objs.first().unwrap();

            // Make sure all definitions are consistent
            for other in &objs[1..] {
                let other2 = other.0.borrow();
                let other = other2.rust_root.borrow();
                let first = first.0.borrow();
                let mut other_fields = HashMap::new();
                for f in other.fields {
                    let f = f.borrow();
                    other_fields.insert(f.field_name, f.serial.redirect.unwrap_or(f.serial.primary));
                }
                for first_f in first.rust_root.borrow().fields {
                    let first_f = first_f.borrow();
                    if let Some(other_f_value) = other_fields.remove(&first_f.field_name) {
                        match NodeSameVariant::pairs(&*other_f_value.0, &*first_f.serial.primary.0) {
                            NodeSameVariant::Int(l, r) => {
                                let l = l.borrow();
                                let r = r.borrow();
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
                            NodeSameVariant::Array(l, r) => {
                                let l_type = l.borrow().element.0.borrow().rust_root.borrow().type_name;
                                let r_type = r.borrow().element.0.borrow().rust_root.borrow().type_name;
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
                                let l = l.borrow();
                                let r = r.borrow();
                                let l_type = l.type_name;
                                let r_type = r.type_name;
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
                                for v in l.variants {
                                    l_variants.insert(v.var_name.to_string(), v.element);
                                }
                                for v in r.variants {
                                    if let Some(l_v) = l_variants.remove(&v.var_name.to_string()) {
                                        let l_type = l_v.0.borrow().rust_root.borrow().type_name;
                                        let r_type = v.element.0.borrow().rust_root.borrow().type_name;
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
            let obj_obj = obj2.rust_root.borrow();

            // Generate code
            let type_ident = obj_obj.type_name;
            let mut fields = vec![];
            for f in &obj_obj.fields {
                let f = f.borrow();
                let field_ident = f.field_name;
                let field_type = match &*f.serial.primary.0 {
                    Node_::Int(n) => n.borrow().rust_type.to_token_stream(),
                    Node_::DynamicBytes(_) => quote!(std:: vec:: Vec < u8 >),
                    Node_::Array(n) => {
                        let inner = n.borrow().element.0.borrow().rust_root.borrow().type_name.clone();
                        quote!(std:: vec:: Vec < #inner >)
                    },
                    Node_::Enum(n) => n.borrow().type_name.into_token_stream(),
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
            if root.external_deps.is_empty() {
                let root_obj = root.rust_root.borrow();
                let obj_ident = root_obj.id.ident();
                let serial_ident = root.serial_root.borrow().id.ident();
                let methods = vec![];
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
                let type_ident = root_obj.type_name;
                code.push(quote!(impl #type_ident {
                    #(#methods) *
                }));
            }
        }
        return quote!(#(#code) *);
    }
}

pub(crate) fn generate_read(obj: &Object_) -> TokenStream {
    let obj_obj = obj.rust_root.borrow();
    let mut seen = HashSet::new();
    let mut stack: Vec<(Node, bool)> = vec![];
    for c in obj.rust_const_roots {
        stack.push((c.into(), false));
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
            stack.push((node, true));
            for dep in node.0.read_deps() {
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
    stack.push((obj.serial_root.into(), false));
    let mut code = vec![];
    while let Some((node, first_visit)) = stack.pop() {
        if first_visit {
            let node_id = node.id();
            if seen.contains(&node_id) {
                continue;
            }
            seen.insert(node_id);

            // Prep before visiting deps
            match &*node.0 {
                Node_::FixedBytes(n) => {
                    code.push(n.borrow().generate_pre_write());
                },
                _ => { },
            }

            // Visit deps, then visit this node again
            stack.push((node, true));
            for dep in node.0.write_deps() {
                stack.push((dep, false));
            }
        } else {
            // Post-deps, now do main processing
            code.push(node.0.generate_write());
        }
    }
    return quote!(#(#code) *);
}
