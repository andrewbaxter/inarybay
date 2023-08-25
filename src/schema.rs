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
                    other_fields.insert(f.field_name, f.value.redirect.unwrap_or(f.value.primary));
                }
                for first_f in first.rust_root.borrow().fields {
                    let first_f = first_f.borrow();
                    if let Some(other_f_value) = other_fields.remove(&first_f.field_name) {
                        match NodeSameVariant::pairs(&*other_f_value.0, &*first_f.value.primary.0) {
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
                            NodeSameVariant::String(_, _) => { },
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
                            NodeSameVariant::Option(l, r) => {
                                let l_type = l.borrow().element.0.borrow().rust_root.borrow().type_name;
                                let r_type = r.borrow().element.0.borrow().rust_root.borrow().type_name;
                                if l_type != r_type {
                                    panic!(
                                        "Definitions of {} field {} have mismatched option inner types {} and {}",
                                        name,
                                        first_f.field_name,
                                        l_type,
                                        r_type
                                    );
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
                let field_type = match &*f.value.primary.0 {
                    Node_::Int(n) => n.borrow().rust_type.to_token_stream(),
                    Node_::DynamicRange(_) => quote!(std:: vec:: Vec < u8 >),
                    Node_::Array(n) => {
                        let inner = n.borrow().element.0.borrow().rust_root.borrow().type_name.clone();
                        quote!(std:: vec:: Vec < #inner >)
                    },
                    Node_::Enum(n) => n.borrow().type_name.into_token_stream(),
                    Node_::Option(n) => {
                        let inner = n.borrow().element.0.borrow().rust_root.borrow().type_name.clone();
                        quote!(Option < #inner >)
                    },
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

fn generate_read(obj: &Object_) -> TokenStream {
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
            stack.push((node, true));
            for dep in node.read_deps() {
                stack.push((dep, false));
            }
        } else {
            match &*node.0 {
                Node_::Serial(_) => { },
                Node_::FixedRange(n) => {
                    let n = n.borrow();
                    let serial_ident = n.serial.id().ident();
                    let ident = n.id.ident();
                    let bytes = n.len_bytes;
                    code.push(quote!{
                        let mut #ident = #serial_ident.read_len(#bytes) ?;
                    });
                },
                Node_::Int(n) => {
                    code.push(n.borrow().generate_read());
                },
                Node_::DynamicRange(n) => {
                    let n = n.borrow();
                    let source_ident = n.serial.borrow().id.ident();
                    let source_len_ident = n.serial_len.primary.borrow().id.ident();
                    let dest_ident = n.id.ident();
                    code.push(quote!{
                        let mut #dest_ident = #source_ident.read_len(#source_len_ident) ?;
                    });
                },
                Node_::Array(n) => {
                    let n = n.borrow();
                    let dest_ident = n.id.ident();
                    let source_len_ident = n.serial_len.primary.borrow().id.ident();
                    let element = n.element.0.borrow();
                    let elem_ident = element.id.ident();
                    let elem_code = generate_read(&*element);
                    code.push(quote!{
                        let mut #dest_ident = vec ![];
                        for _ in 0..len_ident {
                            #elem_code 
                            //. .
                            #dest_ident.push(#elem_ident);
                        }
                    });
                },
                Node_::Enum(n) => {
                    let n = n.borrow();
                    let type_ident = n.type_name;
                    let source_tag_ident = n.serial_tag.primary.id().ident();
                    let dest_ident = n.id.ident();
                    let mut var_code = vec![];
                    for v in n.variants {
                        let tag = v.tag;
                        let var_ident = v.var_name;
                        let elem = v.element.0.borrow();
                        let elem_ident = elem.id.ident();
                        let elem_code = generate_read(&*elem);
                        var_code.push(quote!{
                            #tag => {
                                #elem_code #dest_ident = #type_ident:: #var_ident(#elem_ident);
                            },
                        });
                    }
                    code.push(quote!{
                        let #dest_ident;
                        match #source_tag_ident {
                            #(#var_code) * _ => {
                                return Err("Unknown variant with tag {}", #source_tag_ident);
                            }
                        };
                    });
                },
                Node_::Option(n) => {
                    let n = n.borrow();
                    let source_switch_ident = n.serial_switch.primary.id().ident();
                    let element = n.element.0.borrow();
                    let elem_ident = element.id.ident();
                    let elem_code = generate_read(&element);
                    let dest_ident = n.id.ident();
                    code.push(quote!{
                        let #dest_ident;
                        if #source_switch_ident {
                            #elem_code #dest_ident = Some(#elem_ident);
                        }
                        else {
                            #dest_ident = None;
                        };
                    });
                },
                Node_::Const(n) => {
                    let n = n.borrow();
                    let source_ident = n.value.primary.id().ident();
                    let expect = n.expect;
                    code.push(quote!{
                        if #source_ident != #expect {
                            return Err("Magic mismatch at TODO");
                        }
                    });
                },
                Node_::RustField(n) => { },
                Node_::RustObj(n) => {
                    let n = n.borrow();
                    let type_ident = n.type_name;
                    let dest_ident = n.id.ident();
                    let mut fields = vec![];
                    for f in n.fields {
                        let f = f.borrow();
                        let field_ident = f.field_name;
                        let value_ident = f.value.primary.id().ident();
                        fields.push(quote!{
                            #field_ident: #value_ident,
                        });
                    }
                    code.push(quote!{
                        #dest_ident = #type_ident {
                            #(#fields) *
                        };
                    });
                },
            };
        }
    }
    return quote!(#(#code) *);
}

fn generate_write(obj: &Object_) -> TokenStream {
    // Aside from int, read from rust-side ref ident, write to own id ident
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
                Node_::FixedRange(n) => {
                    let n = n.borrow();
                    let dest_ident = n.id.ident();
                    let len = n.len_bytes;
                    code.push(quote!{
                        let mut #dest_ident = std:: vec:: Vec:: new();
                        #dest_ident.resize(#len, 0u8);
                    });
                },
                _ => { },
            }

            // Visit deps
            stack.push((node, true));
            for dep in node.write_deps() {
                stack.push((dep, false));
            }
        } else {
            // Post-deps, now do main processing
            match &*node.0 {
                Node_::Serial(n) => {
                    let n = n.borrow();
                    let serial_ident = n.id.ident();
                    for child in n.children {
                        let child_ident = child.id().ident();
                        code.push(quote!{
                            #serial_ident.write(& #child_ident) ?;
                        });
                    }
                },
                Node_::FixedRange(n) => { },
                Node_::Int(n) => {
                    code.push(n.borrow().generate_write());
                },
                Node_::DynamicRange(n) => {
                    let n = n.borrow();
                    let source_ident = n.rust.expect("").primary.id().ident();
                    let dest_ident = n.id.ident();
                    let dest_len_ident = n.serial_len.primary.borrow().id.ident();
                    let dest_len_type = n.serial_len.primary.borrow().rust_type;
                    code.push(quote!{
                        let #dest_len_ident = #source_ident.len() as #dest_len_type;
                        let #dest_ident = #source_ident.as_bytes();
                    });
                },
                Node_::Array(n) => {
                    let n = n.borrow();
                    let source_ident = n.id.ident();
                    let dest_ident = n.rust.expect("").primary.id().ident();
                    let dest_len_ident = n.serial_len.primary.borrow().id.ident();
                    let dest_len_type = n.serial_len.primary.borrow().rust_type;
                    let element = n.element.0.borrow();
                    let elem_source_ident = element.rust_root.borrow().id.ident();
                    let elem_code = generate_write(&*element);
                    code.push(quote!{
                        let #dest_len_ident = #source_ident.len();
                        let mut #dest_ident = vec ![];
                        for #elem_source_ident in #source_ident {
                            #elem_source_ident.write(& mut #dest_ident);
                        }
                    });
                },
                Node_::Enum(n) => {
                    let n = n.borrow();
                    let enum_name = n.type_name;
                    let source_ident = n.rust.expect("").primary.id().ident();
                    let dest_ident = n.id.ident();
                    let dest_tag_ident = n.serial_tag.primary.id().ident();
                    let mut var_code = vec![];
                    let mut all_external_deps = BTreeMap::new();
                    for v in n.variants {
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
                    for v in n.variants {
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

                        // Fill in defaults for any external deps this variant didn't write to
                        let mut missed_external_deps = all_external_deps.clone();
                        for external_dep in &element.external_deps {
                            missed_external_deps.remove(&external_dep.id());
                        }
                        let mut default_external_deps = vec![];
                        for external_dep in missed_external_deps.values() {
                            default_external_deps.push(external_dep.write_default());
                        }

                        // Assemble
                        var_code.push(quote!{
                            #enum_name:: #variant_name(#elem_source_ident) => {
                                #dest_tag_ident = #tag;
                                #elem_code 
                                //. .
                                #(#default_external_deps) * 
                                //. .
                                #dest_ident.extend(#elem_dest_ident);
                            },
                        });
                    }
                    code.push(quote!{
                        let #dest_tag_ident;
                        let mut #dest_ident = vec ![];
                        //. .
                        #(#anchor_external_deps) * 
                        //. .
                        match #source_ident {
                            #(#var_code) *
                        };
                    });
                },
                Node_::Option(n) => {
                    let n = n.borrow();
                    let source_ident = n.rust.expect("Option unused").primary.id().ident();
                    let dest_ident = n.id.ident();
                    let dest_switch_ident = n.serial_switch.primary.id().ident();
                    let element = n.element.0.borrow();
                    let elem_source_ident = element.rust_root.borrow().id.ident();
                    let elem_dest_ident = element.serial_root.borrow().id.ident();
                    let elem_code;
                    let mut default_external_deps = vec![];
                    let mut anchor_external_deps = vec![];
                    if element.external_deps.is_empty() {
                        elem_code = quote!{
                            let mut #elem_dest_ident = vec ![];
                            #elem_source_ident.write(& mut #elem_dest_ident);
                        };
                    } else {
                        elem_code = generate_write(&*element);
                        for dep in n.lifted_serial_deps.values() {
                            default_external_deps.push(dep.write_default());
                            let ident = dep.id().ident();
                            anchor_external_deps.push(quote!{
                                let #ident;
                            });
                        }
                    }
                    code.push(quote!{
                        let #dest_switch_ident;
                        let mut #dest_ident = vec ![];
                        //. .
                        #(#anchor_external_deps) * 
                        //. .
                        if let Some(#elem_source_ident) = #source_ident {
                            #dest_switch_ident = true;
                            #elem_code 
                            //. .
                            #dest_ident.extend(#elem_dest_ident);
                        }
                        else {
                            #dest_switch_ident = false;
                            #(#default_external_deps) *
                        };
                    });
                },
                Node_::Const(n) => {
                    let n = n.borrow();
                    let dest_ident = n.id.ident();
                    let expect = n.expect;
                    code.push(quote!{
                        let #dest_ident = #expect;
                    });
                },
                Node_::RustField(n) => {
                    let n = n.borrow();
                    let dest_ident = n.id.ident();
                    let field_ident = n.field_name;
                    let obj_ident = n.obj.borrow().id.ident();
                    code.push(quote!{
                        let #dest_ident = #obj_ident.#field_ident;
                    });
                },
                Node_::RustObj(n) => { },
            };
        }
    }
    return quote!(#(#code) *);
}
