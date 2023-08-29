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
        offset_ident,
    },
    node_enum::NodeEnum,
};

#[derive(Trace, Finalize)]
pub(crate) enum ReaderBounds {
    None,
    Buffered,
}

#[derive(Trace, Finalize)]
pub struct Schema_ {
    #[unsafe_ignore_trace]
    pub(crate) imports: BTreeMap<String, TokenStream>,
    pub(crate) reader_bounds: ReaderBounds,
    pub(crate) objects: BTreeMap<String, Vec<Object>>,
    pub(crate) enums: BTreeMap<String, Vec<NodeEnum>>,
}

impl Schema_ { }

#[derive(Clone, Trace, Finalize)]
pub struct Schema(pub(crate) Gc<GcCell<Schema_>>);

pub struct GenerateConfig {
    /// Generate read methods (async or otherwise)
    pub read: bool,
    /// Generate write methods (async or otherwise)
    pub write: bool,
    /// Generate sync (blocking, non-async) methods
    pub sync_: bool,
    /// Generate async (non-blocking) methods
    pub async_: bool,
    /// Avoid excessive heap allocation... mostly just errors, not quite heap-free at
    /// this time I'm afraid
    pub low_heap: bool,
}

pub(crate) struct GenerateContext {
    pub(crate) low_heap: bool,
    pub(crate) async_: bool,
}

impl GenerateContext {
    pub(crate) fn read_err_type(&self) -> TokenStream {
        match self.low_heap {
            true => return quote!(& 'static str),
            false => return quote!(inarybay_runtime::error::ReadError),
        }
    }

    pub(crate) fn wrap_read_err(&self, node: &str, mut read: TokenStream) -> TokenStream {
        let text = format!("Error parsing, in node {}", node);
        read = quote!(#read.errorize(#text) ?);
        return read;
    }

    pub(crate) fn wrap_read(&self, node: &str, mut read: TokenStream) -> TokenStream {
        if self.async_ {
            read = quote!(#read.await);
        }
        read = self.wrap_read_err(node, read);
        return read;
    }

    pub(crate) fn wrap_write(&self, mut write: TokenStream) -> TokenStream {
        if self.async_ {
            write = quote!(#write.await);
        }
        return quote!(#write ?);
    }

    pub(crate) fn new_read_err(&self, node: &str, text: TokenStream) -> TokenStream {
        match self.low_heap {
            true => {
                let text = format!("Error parsing, in node {}", node);
                return quote!(#text);
            },
            false => {
                let err_type = self.read_err_type();
                return quote!(#err_type {
                    node: #node,
                    inner: #text
                });
            },
        }
    }
}

impl Schema {
    pub fn new() -> Schema {
        return Schema(Gc::new(GcCell::new(Schema_ {
            imports: BTreeMap::new(),
            reader_bounds: ReaderBounds::None,
            objects: BTreeMap::new(),
            enums: BTreeMap::new(),
        })));
    }

    /// Define a new de/serializable object.
    pub fn object(&self, id: impl Into<String>, name: impl Into<String>) -> Object {
        return Object::new(id, &self, name.into());
    }

    /// Generate code for the described schema.
    pub fn generate(&self, config: GenerateConfig) -> TokenStream {
        let self2 = self.0.borrow();

        // Generate types
        let mut code = vec![];
        for (name, enums) in &self2.enums {
            let first = enums.first().unwrap();

            // Make sure all definitions are consistent
            for other in &enums[1..] {
                let mut other_variants = HashMap::new();
                for v in &other.0.mut_.borrow().variants {
                    other_variants.insert(v.var_name.to_string(), v.element.clone());
                }
                for first_v in &first.0.mut_.borrow().variants {
                    if let Some(other_v) = other_variants.remove(&first_v.var_name) {
                        let first_type = &first_v.element.0.rust_root.0.type_name;
                        let other_type = &other_v.0.rust_root.0.type_name;
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
            let type_ident = &first.0.type_name.ident();
            let mut variants = vec![];
            for v in &first.0.mut_.borrow().variants {
                let var_ident = &v.var_name.ident();
                let var_type_ident = &v.element.0.rust_root.0.type_name.ident();
                variants.push(quote!{
                    #var_ident(#var_type_ident),
                });
            }
            let attrs = &first.0.mut_.borrow().type_attrs;
            code.push(quote!{
                #(#attrs) * 
                //. .
                pub enum #type_ident {
                    #(#variants) *
                }
            });
        }
        for (name, objs) in &self2.objects {
            let first = objs.first().unwrap();

            // Make sure all definitions are consistent
            for other in &objs[1..] {
                let mut other_fields = HashMap::new();
                for f in &other.0.rust_root.0.mut_.borrow().fields {
                    let f_mut = f.0.mut_.borrow();
                    let f_serial = f_mut.serial.as_ref().unwrap();
                    other_fields.insert(
                        f.0.field_name.clone(),
                        f_serial.redirect.clone().unwrap_or_else(|| f_serial.primary.clone()),
                    );
                }
                for first_f in &first.0.rust_root.0.mut_.borrow().fields {
                    if let Some(other_f_value) = other_fields.remove(&first_f.0.field_name) {
                        let first_f_mut = first_f.0.mut_.borrow();
                        let first_f_serial = first_f_mut.serial.as_ref().unwrap();
                        match NodeSameVariant::pairs(&other_f_value.0, &first_f_serial.primary.0) {
                            NodeSameVariant::Enum(l, r) => {
                                let l_type = &l.0.type_name;
                                let r_type = &r.0.type_name;
                                if l_type != r_type {
                                    panic!(
                                        "Definitions of {} field {} have mismatched enum inner types {} and {}",
                                        name,
                                        first_f.0.field_name,
                                        l_type,
                                        r_type
                                    );
                                }
                                let mut l_variants = HashMap::new();
                                for v in &l.0.mut_.borrow().variants {
                                    l_variants.insert(v.var_name.to_string(), v.element.clone());
                                }
                                for v in &r.0.mut_.borrow().variants {
                                    if let Some(l_v) = l_variants.remove(&v.var_name.to_string()) {
                                        let l_type = &l_v.0.rust_root.0.type_name;
                                        let r_type = &v.element.0.rust_root.0.type_name;
                                        if l_type != r_type {
                                            panic!(
                                                "Definitions of {} enum field {} variant {} have mismatched inner types {} and {}",
                                                name,
                                                first_f.0.field_name,
                                                v.var_name,
                                                l_type,
                                                r_type
                                            );
                                        }
                                    } else {
                                        panic!(
                                            "Definitions of {} enum field {} are missing variant {}",
                                            name,
                                            first_f.0.field_name,
                                            v.var_name
                                        );
                                    }
                                }
                            },
                            NodeSameVariant::Nonmatching(l, r) => {
                                let l_type = l.rust_type().to_string();
                                let r_type = r.rust_type().to_string();
                                if l_type != r_type {
                                    panic!(
                                        "Some definitions of {} have value type {}, others {}",
                                        name,
                                        l_type,
                                        r_type
                                    );
                                }
                            },
                            _ => unreachable!(),
                        }
                    } else {
                        panic!("Some definitions of {} are missing field {}", name, first_f.0.field_name);
                    }
                }
                for f in other_fields.keys() {
                    panic!("Some definitions of {} are missing field {}", name, f);
                }
            }

            // Generate code
            let type_ident = &first.0.rust_root.0.type_name.ident();
            let mut fields = vec![];
            for f in &first.0.rust_root.0.mut_.borrow().fields {
                let field_ident = &f.0.field_name.ident();
                let field_type = f.0.mut_.borrow().serial.as_ref().unwrap().primary.rust_type();
                fields.push(quote!{
                    pub #field_ident: #field_type,
                });
            }
            let attrs = &first.0.mut_.borrow().type_attrs;
            code.push(quote!{
                #(#attrs) * 
                //. .
                pub struct #type_ident {
                    #(#fields) *
                }
            });

            // Generate de/serialization methods for any objs with self-contained
            // serialization. (TODO Should maybe check for exactly one serialization format?).
            let root = first;
            if !root.0.mut_.borrow().has_external_deps {
                let offset_ident = offset_ident();
                let root_obj = &root.0.rust_root.0;
                let obj_ident = root_obj.id.ident();
                let serial_ident = root.0.serial_root.0.id.ident();
                let mut methods = vec![];
                if config.read {
                    if config.sync_ {
                        let gen_ctx = GenerateContext {
                            low_heap: config.low_heap,
                            async_: false,
                        };
                        let reader = match self.0.borrow().reader_bounds {
                            ReaderBounds::None => quote!(std::io::Read),
                            ReaderBounds::Buffered => quote!(std::io::BufRead),
                        };
                        let code = generate_read(&gen_ctx, &root.0);
                        let err_ident = gen_ctx.read_err_type();
                        methods.push(quote!{
                            pub fn read < R: #reader >(#serial_ident:& mut R) -> Result < Self,
                            #err_ident > {
                                let mut #offset_ident = 0usize;
                                #code 
                                //. .
                                return Ok(#obj_ident);
                            }
                        });
                    }
                    if config.async_ {
                        let gen_ctx = GenerateContext {
                            low_heap: config.low_heap,
                            async_: true,
                        };
                        let reader = match self.0.borrow().reader_bounds {
                            ReaderBounds::None => quote!(inarybay_runtime::async_::AsyncReadExt),
                            ReaderBounds::Buffered => quote!(inarybay_runtime::async_::AsyncBufReadExt),
                        };
                        let code = generate_read(&gen_ctx, &root.0);
                        let err_ident = gen_ctx.read_err_type();
                        methods.push(quote!{
                            pub async fn read_async < R: #reader + std:: marker:: Unpin >(
                                #serial_ident:& mut R
                            ) -> Result < Self,
                            #err_ident > {
                                let mut #offset_ident = 0usize;
                                #code 
                                //. .
                                return Ok(#obj_ident);
                            }
                        });
                    }
                }
                if config.write {
                    if config.sync_ {
                        let gen_ctx = GenerateContext {
                            low_heap: config.low_heap,
                            async_: false,
                        };
                        let code = generate_write(&gen_ctx, &root.0);
                        methods.push(quote!{
                            pub fn write < W: std:: io:: Write >(
                                &self,
                                #serial_ident:& mut W
                            ) -> std:: io:: Result <() > {
                                let mut #offset_ident = 0usize;
                                let #obj_ident = self;
                                //. .
                                #code 
                                //. .
                                return Ok(());
                            }
                        });
                    }
                    if config.async_ {
                        let gen_ctx = GenerateContext {
                            low_heap: config.low_heap,
                            async_: true,
                        };
                        let code = generate_write(&gen_ctx, &root.0);
                        methods.push(quote!{
                            pub async fn write_async < W: inarybay_runtime:: async_:: AsyncWriteExt + std:: marker:: Unpin >(
                                &self,
                                #serial_ident:& mut W
                            ) -> std:: io:: Result <() > {
                                let mut #offset_ident = 0usize;
                                let #obj_ident = self;
                                //. .
                                #code 
                                //. .
                                return Ok(());
                            }
                        });
                    }
                }
                let type_ident = &root_obj.type_name.ident();
                code.push(quote!{
                    impl #type_ident {
                        #(#methods) *
                    }
                });
            }
        }
        let use_err;
        match config.low_heap {
            true => {
                use_err = quote!{
                    use inarybay_runtime::lowheap_error::ReadErrCtx;
                };
            },
            false => {
                use_err = quote!{
                    use inarybay_runtime::error::ReadErrCtx;
                };
            },
        }
        let imports: Vec<TokenStream> = self.0.borrow().imports.values().cloned().collect();
        return quote!{
            #![
                allow(
                    non_snake_case,
                    dropping_copy_types,
                    dropping_references,
                    unused_mut,
                    unused_variables,
                    unused_parens,
                )
            ] 
            //. .
            #(#imports) * 
            //. .
            #use_err 
            //. .
            #(#code) *
        };
    }

    /// Add an import line to the generated code. Deduplicated by naive stringification.
    pub fn add_import(&self, import: TokenStream) {
        self.0.borrow_mut().imports.insert(import.to_string(), import);
    }
}

pub(crate) fn generate_read(gen_ctx: &GenerateContext, obj: &Object_) -> TokenStream {
    let mut seen = HashSet::new();
    let mut stack: Vec<(Node, bool)> = vec![];
    stack.push((obj.rust_root.clone().into(), true));
    for c in &obj.mut_.borrow().rust_const_roots {
        stack.push((c.clone().into(), true));
    }
    for c in &obj.serial_root.0.mut_.borrow().sub_segments {
        // Make sure to read everything, even if padding, to leave stream in expected state
        stack.push((c.clone().into(), true));
    }
    let mut code = vec![];
    while let Some((node, first_visit)) = stack.pop() {
        if first_visit {
            if !seen.insert(node.id()) {
                continue;
            }

            // Visit deps, then visit this node again
            stack.push((node.clone(), false));
            for dep in node.0.gather_read_deps() {
                stack.push((dep, true));
            }
        } else {
            // Post-deps, now do main processing
            code.push(node.0.generate_read(gen_ctx));
        }
    }
    return quote!(#(#code) *);
}

pub(crate) fn generate_write(gen_ctx: &GenerateContext, obj: &Object_) -> TokenStream {
    // Read from own id ident, write to serial-side id idents (except serial
    // segment/serial root)
    let mut seen = HashSet::new();
    let mut stack: Vec<(Node, bool)> = vec![];
    stack.push((obj.serial_root.clone().into(), true));
    let mut code = vec![];
    while let Some((node, first_visit)) = stack.pop() {
        if first_visit {
            if !seen.insert(node.id()) {
                continue;
            }

            // Prep before visiting deps
            match &node.0 {
                Node_::FixedRange(n) => {
                    code.push(n.0.generate_pre_write());
                },
                _ => { },
            }

            // Visit deps, then visit this node again
            stack.push((node.clone(), false));
            for dep in node.0.gather_write_deps() {
                stack.push((dep, true));
            }
        } else {
            // Post-deps, now do main processing
            code.push(node.0.generate_write(gen_ctx));
        }
    }
    return quote!(#(#code) *);
}
