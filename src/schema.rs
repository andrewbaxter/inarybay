use std::{
    collections::{
        HashMap,
        BTreeMap,
        HashSet,
        btree_map::{
            Entry,
        },
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
    format_ident,
};
use crate::{
    util::{
        offset_ident,
        ToIdent,
    },
    node::{
        node::{
            Node,
            Node_,
            NodeMethods,
        },
        node_object::NodeObj,
        node_enum::NodeEnum,
    },
    scope::Scope,
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
    #[unsafe_ignore_trace]
    pub(crate) mods: Vec<TokenStream>,
    pub(crate) reader_bounds: ReaderBounds,
    pub(crate) top_scopes: BTreeMap<String, Scope>,
    pub(crate) objects: BTreeMap<String, Vec<NodeObj>>,
    pub(crate) enums: BTreeMap<String, Vec<NodeEnum>>,
}

impl Schema_ { }

/// A schema is the entrypoint for generating de/serializers.  A schema currently
/// generates one module with all the types/functions.  Types will be
/// deduplicated/reused within a schema.
#[derive(Clone, Trace, Finalize)]
pub struct Schema(pub(crate) Gc<GcCell<Schema_>>);

#[derive(Default)]
pub struct GenerateConfig {
    /// Generate read methods (async or otherwise)
    pub read: bool,
    /// Generate write methods (async or otherwise)
    pub write: bool,
    /// Generate sync (blocking, non-async) methods
    pub sync_: bool,
    /// Generate async (non-blocking) methods
    pub async_: bool,
    /// If true, errors are a single pointer with no allocations. (This may be used for
    /// other memory optimizations in the future.)
    pub low_heap: bool,
}

pub(crate) struct GenerateContext {
    pub(crate) low_heap: bool,
    pub(crate) async_: bool,
}

impl GenerateContext {
    pub(crate) fn read_err_type(&self) -> TokenStream {
        match self.low_heap {
            true => return quote!(inarybay_runtime::lowheap_error::ReadError),
            false => return quote!(inarybay_runtime::error::ReadError),
        }
    }

    pub(crate) fn wrap_read(&self, node: &str, mut read: TokenStream) -> TokenStream {
        read = self.wrap_async(read);
        read = quote!(#read.errorize_io(#node) ?);
        return read;
    }

    pub(crate) fn wrap_async(&self, mut read: TokenStream) -> TokenStream {
        if self.async_ {
            read = quote!(#read.await);
        }
        return read;
    }

    pub(crate) fn wrap_write(&self, mut write: TokenStream) -> TokenStream {
        write = self.wrap_async(write);
        return quote!(#write ?);
    }

    pub(crate) fn new_read_err(&self, node: &str, lowheap_text: &str, text: TokenStream) -> TokenStream {
        match self.low_heap {
            true => {
                let err_type = self.read_err_type();
                return quote!(#err_type:: new(#node, #lowheap_text));
            },
            false => {
                let err_type = self.read_err_type();
                return quote!(#err_type:: new(#node, #text));
            },
        }
    }
}

impl Schema {
    pub fn new() -> Schema {
        return Schema(Gc::new(GcCell::new(Schema_ {
            imports: BTreeMap::new(),
            mods: vec![],
            reader_bounds: ReaderBounds::None,
            top_scopes: BTreeMap::new(),
            objects: BTreeMap::new(),
            enums: BTreeMap::new(),
        })));
    }

    /// See `Scope` for more details. Scopes created directly in the Schema will have
    /// (de)serialization methods generated. The method names will be prefixed with
    /// `prefix + "_"` if `prefix` is non-empty.
    pub fn scope(&self, id: impl Into<String>, prefix: impl Into<String>) -> Scope {
        let out = Scope::new(id, self);
        match self.0.borrow_mut().top_scopes.entry(prefix.into()) {
            Entry::Vacant(e) => {
                e.insert(out.clone());
            },
            Entry::Occupied(_) => {
                panic!("The prefix for top level scope {} is not unique", out.0.id);
            },
        };
        return out;
    }

    /// Generate code for the schema.
    pub fn generate(&self, config: GenerateConfig) -> String {
        let self2 = self.0.borrow();

        // Generate types
        let mut code = vec![quote!{
            #![allow(warnings, unused)]
        }];
        match config.low_heap {
            true => {
                code.push(quote!{
                    use inarybay_runtime::lowheap_error::ReadErrCtx;
                    use inarybay_runtime::lowheap_error::ReadErrCtxIo;
                });
            },
            false => {
                code.push(quote!{
                    use inarybay_runtime::error::ReadErrCtx;
                    use inarybay_runtime::error::ReadErrCtxIo;
                });
            },
        }
        code.extend(self.0.borrow().imports.values().cloned());
        for mod_ in &self.0.borrow().mods {
            code.push(quote!(pub mod #mod_;));
        }
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
                        let first_type = &first_v.element.get_rust_root().rust_type().to_string();
                        let other_type = &other_v.get_rust_root().rust_type().to_string();
                        if first_type != other_type {
                            panic!(
                                "Definitions of enum {} variant {} have mismatched types: {} and {}",
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
                match (&first.0.mut_.borrow().default_variant, &other.0.mut_.borrow().default_variant) {
                    (None, None) => { },
                    (Some(first_v), Some(other_v)) => {
                        let first_type = &first_v.element.get_rust_root().rust_type().to_string();
                        let other_type = &other_v.element.get_rust_root().rust_type().to_string();
                        if first_type != other_type {
                            panic!(
                                "Definitions of enum {} default variant {} have mismatched types: {} and {}",
                                name,
                                first_v.var_name,
                                first_type,
                                other_type
                            );
                        }
                    },
                    _ => {
                        panic!("Some definitions of {} are missing a default variant", name);
                    },
                }
            }

            // Generate code
            let type_ident = &first.0.type_name_ident;
            let mut variants = vec![];
            for v in &first.0.mut_.borrow().variants {
                let var_ident = &v.var_name_ident;
                let var_type_ident = &v.element.get_rust_root().rust_type();
                variants.push(quote!{
                    #var_ident(#var_type_ident),
                });
            }
            if let Some(default_v) = &first.0.mut_.borrow().default_variant {
                let var_ident = &default_v.var_name_ident;
                let var_type_ident = &default_v.element.get_rust_root().rust_type();
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
                for f in &other.0.mut_.borrow().fields {
                    let f_mut = f.0.mut_.borrow();
                    let f_serial = f_mut.serial.as_ref().unwrap();
                    other_fields.insert(
                        f.0.field_name.clone(),
                        f_serial.redirect.clone().unwrap_or_else(|| f_serial.primary.clone()),
                    );
                }
                for first_f in &first.0.mut_.borrow().fields {
                    if let Some(other_f_value) = other_fields.remove(&first_f.0.field_name) {
                        let first_f_mut = first_f.0.mut_.borrow();
                        let first_f_serial = first_f_mut.serial.as_ref().unwrap();
                        let l_type = other_f_value.0.rust_type().to_string();
                        let r_type = first_f_serial.primary.0.rust_type().to_string();
                        if l_type != r_type {
                            panic!("Some definitions of {} have value type {}, others {}", name, l_type, r_type);
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
            let type_ident = &first.0.type_name_ident;
            let mut fields = vec![];
            for f in &first.0.mut_.borrow().fields {
                let field_ident = &f.0.field_name_ident;
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
        }
        for (prefix, scope) in &self.0.borrow().top_scopes {
            let prefix = if prefix.is_empty() {
                "".to_string()
            } else {
                format!("{}_", prefix)
            };
            let offset_ident = offset_ident();
            let rust = scope.get_rust_root();
            let rust_ident = rust.id_ident();
            let rust_type_ident = rust.rust_type();
            let serial_ident = &scope.0.serial_root.0.id_ident;
            if config.read {
                if config.sync_ {
                    let gen_ctx = GenerateContext {
                        low_heap: config.low_heap,
                        async_: false,
                    };
                    let reader = match &self.0.borrow().reader_bounds {
                        ReaderBounds::None => quote!(std::io::Read),
                        ReaderBounds::Buffered => quote!(std::io::BufRead),
                    };
                    let method_ident = format_ident!("{}read", prefix);
                    let method_code = generate_read(&gen_ctx, scope);
                    let err_ident = gen_ctx.read_err_type();
                    code.push(quote!{
                        pub fn #method_ident < R: #reader >(#serial_ident:& mut R) -> Result < #rust_type_ident,
                        #err_ident > {
                            let mut #offset_ident = 0usize;
                            #method_code 
                            //. .
                            return Ok(#rust_ident);
                        }
                    });
                }
                if config.async_ {
                    let gen_ctx = GenerateContext {
                        low_heap: config.low_heap,
                        async_: true,
                    };
                    let reader = match &self.0.borrow().reader_bounds {
                        ReaderBounds::None => quote!(inarybay_runtime::async_::AsyncReadExt),
                        ReaderBounds::Buffered => quote!(inarybay_runtime::async_::AsyncBufReadExt),
                    };
                    let method_ident = format_ident!("{}read_async", prefix);
                    let method_code = generate_read(&gen_ctx, scope);
                    let err_ident = gen_ctx.read_err_type();
                    code.push(quote!{
                        pub async fn #method_ident < R: #reader + std:: marker:: Unpin >(
                            #serial_ident:& mut R
                        ) -> Result < #rust_type_ident,
                        #err_ident > {
                            let mut #offset_ident = 0usize;
                            #method_code 
                            //. .
                            return Ok(#rust_ident);
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
                    let method_ident = format_ident!("{}write", prefix);
                    let method_code = generate_write(&gen_ctx, scope);
                    code.push(quote!{
                        pub fn #method_ident < W: std:: io:: Write >(
                            #rust_ident: #rust_type_ident,
                            #serial_ident:& mut W
                        ) -> std:: io:: Result <() > {
                            use std::io::Write;
                            let mut #offset_ident = 0usize;
                            //. .
                            #method_code 
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
                    let method_ident = format_ident!("{}write_async", prefix);
                    let method_code = generate_write(&gen_ctx, scope);
                    code.push(quote!{
                        pub async fn #method_ident < W: inarybay_runtime:: async_:: AsyncWriteExt + std:: marker:: Unpin >(
                            #rust_ident: #rust_type_ident,
                            #serial_ident:& mut W
                        ) -> std:: io:: Result <() > {
                            use inarybay_runtime::async_::AsyncWriteExt;
                            let mut #offset_ident = 0usize;
                            //. .
                            #method_code 
                            //. .
                            return Ok(());
                        }
                    });
                }
            }
        }
        let stream = quote!{
            #(#code) *
        };
        return genemichaels::format_ast(
            syn::parse2::<syn::File>(
                stream.clone(),
            ).expect(
                &format!(
                    "Failed to generate module code; check that any attributes or imports you added have valid Rust syntax\nRaw source: {}",
                    stream
                ),
            ),
            &genemichaels::FormatConfig::default(),
            HashMap::new(),
        )
            .unwrap()
            .rendered;
    }

    /// Add an import line to the generated module. Deduplicated by naive
    /// stringification.
    pub fn add_import(&self, import: TokenStream) {
        self.0.borrow_mut().imports.insert(import.to_string(), import);
    }

    /// Add a `pub mod ...` line to the generated module, for if you want to add custom
    /// implementations or methods to the generated types.
    pub fn add_mod(&self, name: impl Into<String>) {
        let ident = name.into().ident().expect("Invalid mod name");
        self.0.borrow_mut().mods.push(ident.into_token_stream());
    }
}

pub(crate) fn generate_read(gen_ctx: &GenerateContext, scope: &Scope) -> TokenStream {
    let mut seen = HashSet::new();
    let mut stack: Vec<(Node, bool)> = vec![];
    stack.push((scope.0.mut_.borrow().rust_root.as_ref().unwrap().clone(), true));
    for c in &scope.0.mut_.borrow().rust_extra_roots {
        stack.push((c.clone().into(), true));
    }
    for c in &scope.0.serial_root.0.mut_.borrow().sub_segments {
        // Make sure to read everything, even if padding, to leave stream in expected state
        stack.push((c.clone().into(), true));
    }
    let mut code = vec![];
    for (id, info) in &scope.0.mut_.borrow().level_ids {
        let Some(node) = info else {
            continue;
        };
        let id = id.ident().unwrap();
        let rust_type = node.rust_type();
        code.push(quote!{
            let mut #id: #rust_type;
        });
    }
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

pub(crate) fn generate_write(gen_ctx: &GenerateContext, scope: &Scope) -> TokenStream {
    // Read from own id ident, write to serial-side id idents (except serial
    // segment/serial root)
    let mut seen = HashSet::new();
    let mut stack: Vec<(Node, bool)> = vec![];
    for c in &scope.0.mut_.borrow().serial_extra_roots {
        stack.push((c.clone().into(), true));
    }
    stack.push((scope.0.serial_root.clone().into(), true));
    let mut code = vec![];
    let rust_root_id = scope.get_rust_root().id();
    for (id, info) in &scope.0.mut_.borrow().level_ids {
        let Some(node) = info else {
            continue;
        };
        if *id == rust_root_id {
            continue;
        }
        let id = id.ident().unwrap();
        let rust_type = node.rust_type();
        code.push(quote!{
            let mut #id: #rust_type;
        });
    }
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
