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
    format_ident,
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
        new_s,
        ToIdent,
    },
    node_rust_obj::NodeRustObj,
    node_serial::NodeSerial,
};

pub struct Schema_ {
    next_id: usize,
    objects: BTreeMap<String, Vec<Object>>,
}

impl Schema_ {
    pub(crate) fn take_id(&mut self) -> String {
        let out = format!("x{}", self.next_id);
        self.next_id += 1;
        return out;
    }
}

struct Schema(Rc<RefCell<Schema_>>);

impl Schema {
    pub fn new() -> Schema {
        return Schema(Rc::new(RefCell::new(Schema_ {
            next_id: 0,
            objects: BTreeMap::new(),
        })));
    }

    /// Define a new de/serializable object.
    pub fn object(&self, name: String) -> Object {
        let mut self2 = self.0.borrow_mut();
        let out = Object(Rc::new(RefCell::new(Object_ {
            schema: Rc::downgrade(&self.0),
            parent: None,
            id: self2.take_id(),
            serial_root: new_s(NodeSerial {
                id: self2.take_id(),
                children: vec![],
            }),
            rust_root: new_s(NodeRustObj {
                id: self2.take_id(),
                type_ident: format_ident!("{}", name),
                fields: vec![],
            }),
            rust_const_roots: vec![],
            updeps: vec![],
        })));
        self2.objects.entry(name).or_insert_with(Vec::new).push(out);
        return out;
    }

    /// Generate code for the described schema.
    pub fn generate(&self, read: bool, write: bool) -> TokenStream {
        let self2 = self.0.borrow();

        // Generate types
        let mut code = vec![];
        for (name, objs) in self2.objects {
            let mut first: Option<Object> = None;
            for obj in objs {
                let obj2 = obj.0.borrow();
                let obj_obj = obj2.rust_root.borrow();

                // Make sure all definitions are consistent
                if let Some(first) = first {
                    let first = first.0.borrow();
                    let mut obj_fields = HashMap::new();
                    for f in obj_obj.fields {
                        let f = f.borrow();
                        obj_fields.insert(f.field_ident.to_string(), f.value);
                    }
                    for f in first.rust_root.borrow().fields {
                        let f = f.borrow();
                        if let Some(obj_f_value) = obj_fields.remove(&f.field_ident.to_string()) {
                            match NodeSameVariant::pairs(&*obj_f_value.0, &*f.value.0) {
                                NodeSameVariant::Int(l, r) => {
                                    let l = l.borrow();
                                    let r = r.borrow();
                                    if l.rust_type.to_string() != r.rust_type.to_string() {
                                        panic!(
                                            "Definitions of {} field {} have mismatched rust types {} and {}",
                                            name,
                                            f.field_ident,
                                            l.rust_type,
                                            r.rust_type
                                        );
                                    }
                                },
                                NodeSameVariant::String(_, _) => { },
                                NodeSameVariant::Array(l, r) => {
                                    let l_type = l.borrow().element.0.borrow().rust_root.borrow().type_ident;
                                    let r_type = r.borrow().element.0.borrow().rust_root.borrow().type_ident;
                                    if l_type != r_type {
                                        panic!(
                                            "Definitions of {} field {} have mismatched array element types {} and {}",
                                            name,
                                            f.field_ident,
                                            l_type,
                                            r_type
                                        );
                                    }
                                },
                                NodeSameVariant::Enum(l, r) => {
                                    let l = l.borrow();
                                    let r = r.borrow();
                                    let l_type = l.type_ident;
                                    let r_type = r.type_ident;
                                    if l_type != r_type {
                                        panic!(
                                            "Definitions of {} field {} have mismatched enum inner types {} and {}",
                                            name,
                                            f.field_ident,
                                            l_type,
                                            r_type
                                        );
                                    }
                                    let mut l_variants = HashMap::new();
                                    for v in l.variants {
                                        l_variants.insert(v.var_ident.to_string(), v.element);
                                    }
                                    for v in r.variants {
                                        if let Some(l_v) = l_variants.remove(&v.var_ident.to_string()) {
                                            let l_type = l_v.0.borrow().rust_root.borrow().type_ident;
                                            let r_type = v.element.0.borrow().rust_root.borrow().type_ident;
                                            if l_type != r_type {
                                                panic!(
                                                    "Definitions of {} enum field {} variant {} have mismatched inner types {} and {}",
                                                    name,
                                                    f.field_ident,
                                                    v.var_ident,
                                                    l_type,
                                                    r_type
                                                );
                                            }
                                        } else {
                                            panic!(
                                                "Definitions of {} enum field {} are missing variant {}",
                                                name,
                                                f.field_ident,
                                                v.var_ident
                                            );
                                        }
                                    }
                                },
                                NodeSameVariant::Option(l, r) => {
                                    let l_type = l.borrow().element.0.borrow().rust_root.borrow().type_ident;
                                    let r_type = r.borrow().element.0.borrow().rust_root.borrow().type_ident;
                                    if l_type != r_type {
                                        panic!(
                                            "Definitions of {} field {} have mismatched option inner types {} and {}",
                                            name,
                                            f.field_ident,
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
                            panic!("Some definitions of {} are missing field {}", name, f.field_ident);
                        }
                    }
                    for f in obj_fields.keys() {
                        panic!("Some definitions of {} are missing field {}", name, f);
                    }
                } else {
                    first = Some(obj);
                }

                // Generate code
                let type_ident = obj_obj.type_ident;
                let mut fields = vec![];
                for f in &obj_obj.fields {
                    let f = f.borrow();
                    let field_ident = f.field_ident;
                    let field_type = match &*f.value.0 {
                        Node_::Int(n) => n.borrow().rust_type.to_token_stream(),
                        Node_::String(_) => quote!(std:: vec:: Vec < u8 >),
                        Node_::Array(n) => {
                            let inner = n.borrow().element.0.borrow().rust_root.borrow().type_ident.clone();
                            quote!(std:: vec:: Vec < #inner >)
                        },
                        Node_::Enum(n) => n.borrow().type_ident.into_token_stream(),
                        Node_::Option(n) => {
                            let inner = n.borrow().element.0.borrow().rust_root.borrow().type_ident.clone();
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
            }

            // Generate de/serialization methods for any objs with self-contained
            // serialization. (TODO Should maybe check for exactly one serialization format?).
            let root = first.unwrap();
            let root = root.0.borrow();
            if root.updeps.is_empty() {
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
                let type_ident = root_obj.type_ident;
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
                Node_::SerialRange(n) => {
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
                Node_::String(n) => {
                    let n = n.borrow();
                    let source_ident = n.serial.borrow().id.ident();
                    let source_len_ident = n.serial_len.borrow().id.ident();
                    let dest_ident = n.id.ident();
                    code.push(quote!{
                        let mut #dest_ident = #source_ident.read_len(#source_len_ident) ?;
                    });
                },
                Node_::Array(n) => {
                    let n = n.borrow();
                    let dest_ident = n.id.ident();
                    let source_len_ident = n.serial_len.borrow().id.ident();
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
                    let type_ident = n.type_ident;
                    let source_tag_ident = n.serial_tag.id().ident();
                    let dest_ident = n.id.ident();
                    let mut var_code = vec![];
                    for v in n.variants {
                        let tag = hex::encode(v.tag);
                        let var_ident = v.var_ident;
                        let elem = v.element.0.borrow();
                        let elem_ident = elem.id.ident();
                        let elem_code = generate_read(&*elem);
                        var_code.push(quote!{
                            hex_literal:: hex !(#tag) => {
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
                    let source_switch_ident = n.serial_switch.id().ident();
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
                    let source_ident = n.value.id().ident();
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
                    let type_ident = n.type_ident;
                    let dest_ident = n.id.ident();
                    let mut fields = vec![];
                    for f in n.fields {
                        let f = f.borrow();
                        let field_ident = f.field_ident;
                        let value_ident = f.value.id().ident();
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
                Node_::SerialRange(n) => {
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
                Node_::SerialRange(n) => { },
                Node_::Int(n) => {
                    code.push(n.borrow().generate_write());
                },
                Node_::String(n) => {
                    let n = n.borrow();
                    let source_ident = n.rust.expect("").id().ident();
                    let dest_ident = n.id.ident();
                    let dest_len_ident = n.serial_len.borrow().id.ident();
                    let dest_len_type = n.serial_len.borrow().rust_type;
                    code.push(quote!{
                        let #dest_len_ident = #source_ident.len() as #dest_len_type;
                        let #dest_ident = #source_ident.as_bytes();
                    });
                },
                Node_::Array(n) => {
                    let n = n.borrow();
                    let source_ident = n.id.ident();
                    let dest_ident = n.rust.expect("").id().ident();
                    let dest_len_ident = n.serial_len.borrow().id.ident();
                    let dest_len_type = n.serial_len.borrow().rust_type;
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
                    let enum_name = n.type_ident;
                    let source_ident = n.rust.expect("").id().ident();
                    let dest_ident = n.id.ident();
                    let dest_tag_ident = n.serial_tag.id().ident();
                    let mut var_code = vec![];
                    for v in n.variants {
                        let tag = hex::encode(v.tag);
                        let variant_name = v.var_ident;
                        let element = v.element.0.borrow();
                        let elem_source_ident = element.rust_root.borrow().id.ident();
                        let elem_dest_ident = element.serial_root.borrow().id.ident();
                        let elem_code;
                        if !element.updeps.is_empty() {
                            elem_code = quote!{
                                let mut #elem_dest_ident = vec ![];
                                #elem_source_ident.write(& mut #elem_dest_ident);
                            };
                        } else {
                            elem_code = generate_write(&*element);
                        }
                        var_code.push(quote!{
                            #enum_name:: #variant_name(#elem_source_ident) => {
                                #dest_tag_ident = hex_literal:: hex !(#tag);
                                #elem_code 
                                //. .
                                #dest_ident.extend(#elem_dest_ident);
                            },
                        });
                    }
                    code.push(quote!{
                        let #dest_tag_ident;
                        let mut #dest_ident = vec ![];
                        match #source_ident {
                            #(#var_code) *
                        };
                    });
                },
                Node_::Option(n) => {
                    let n = n.borrow();
                    let source_ident = n.rust.expect("Option unused").id().ident();
                    let dest_ident = n.id.ident();
                    let dest_switch_ident = n.serial_switch.id().ident();
                    let element = n.element.0.borrow();
                    let elem_source_ident = element.rust_root.borrow().id.ident();
                    let elem_dest_ident = element.serial_root.borrow().id.ident();
                    let elem_code;
                    if !element.updeps.is_empty() {
                        elem_code = quote!{
                            let mut #elem_dest_ident = vec ![];
                            #elem_source_ident.write(& mut #elem_dest_ident);
                        };
                    } else {
                        elem_code = generate_write(&*element);
                    }
                    code.push(quote!{
                        let #dest_switch_ident;
                        let mut #dest_ident = vec ![];
                        if let Some(#elem_source_ident) = #source_ident {
                            #dest_switch_ident = true;
                            #elem_code 
                            //. .
                            #dest_ident.extend(#elem_dest_ident);
                        }
                        else {
                            #dest_switch_ident = false;
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
                    let field_ident = n.field_ident;
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
