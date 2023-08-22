use std::{
    cell::{
        Cell,
        RefCell,
    },
    rc::Rc,
    collections::HashMap,
    fmt::Display,
};
use proc_macro2::{
    TokenStream,
    Ident,
};
use quote::{
    quote,
    format_ident,
    IdentFragment,
};

fn reader_ident() -> Ident {
    return format_ident!("reader");
}

fn writer_ident() -> Ident {
    return format_ident!("writer");
}

struct RustField {
    name: String,
    value: Node,
}

struct Builder_ {
    parent: Option<Builder>,
    id: String,
    type_ident: Ident,
    next_id: Rc<Cell<usize>>,
    serial_root: Option<Node>,
    rust_fields: Vec<RustField>,
    rust_consts: Vec<S<NodeRustConst>>,
}

struct Builder(Rc<RefCell<Builder_>>);

enum Endian {
    Big,
    Little,
}

struct EnumVariant {
    name: Ident,
    tag: Vec<u8>,
    element: Builder,
}

trait ToIdent {
    fn ident(&self) -> Ident;
}

impl<T: IdentFragment> ToIdent for T {
    fn ident(&self) -> Ident {
        return format_ident!("{}", self);
    }
}

type S<T> = &'static RefCell<T>;

struct NodeSerial {
    id: String,
}

struct NodeRange {
    id: String,
    serial_before: Option<Node>,
    // NodeSerial or NodeRange
    serial: Node,
    bytes: usize,
    bits: usize,
    sub_ranges: Vec<S<NodeRange>>,
    value: Option<Node>,
}

struct NodeInt {
    id: String,
    serial: S<NodeRange>,
    signed: bool,
    endian: Endian,
}

struct NodeString {
    id: String,
    serial_before: Option<Node>,
    serial: Node,
    serial_len: S<NodeInt>,
}

struct NodeArray {
    id: String,
    serial_before: Option<Node>,
    serial_len: Node,
    element: Builder,
    rust: Option<Node>,
}

struct NodeEnum {
    id: String,
    name: Ident,
    serial_before: Option<Node>,
    serial_tag: Node,
    variants: Vec<EnumVariant>,
    rust: Option<Node>,
}

struct NodeOption {
    id: String,
    serial_before: Option<Node>,
    serial_switch: Node,
    element: Builder,
    rust: Option<Node>,
}

struct NodeRustConst {
    id: String,
    value: Node,
    expect: TokenStream,
}

struct NodeRustField {
    id: String,
    field_name: String,
    value: Node,
    obj: S<NodeRustObj>,
}

struct NodeRustObj {
    id: String,
    obj_ident: Ident,
    fields: Vec<S<NodeRustField>>,
}

enum Node_ {
    Serial(S<NodeSerial>),
    SerialRange(S<NodeRange>),
    Int(S<NodeInt>),
    String(S<NodeString>),
    Array(S<NodeArray>),
    Enum(S<NodeEnum>),
    Option(S<NodeOption>),
    Const(S<NodeRustConst>),
    RustField(S<NodeRustField>),
    RustObj(S<NodeRustObj>),
}

type Node = S<Node_>;

fn new() -> Builder {
    return Builder(Rc::new(RefCell::new(Builder_ {
        next_id: Rc::new(Cell::new(0usize)),
        serial_root: None,
        serial_last: None,
        rust_fields: vec![],
    })));
}

fn node_read_deps(n: Node) -> Vec<Node> { }

fn node_write_deps(n: Node) -> Vec<Node> { }

fn node_id(n: Node) -> String { }

fn builder_read(b: &Builder_) -> TokenStream {
    let obj_ident = b.type_ident;
    let ident = b.id.ident();
    let mut seen_count = HashMap::new();
    let mut stack = vec![];
    for f in b.rust_fields {
        stack.push((f.value, false));
    }
    for c in b.rust_consts {
        stack.push((new_node(Node_::Const(c)), false));
    }
    let mut code = vec![];
    while let Some((node, first_visit)) = stack.pop() {
        if first_visit {
            stack.push((node, true));
            for dep in node_read_deps(node) {
                stack.push((dep, false));
            }
        } else if *seen_count
            .entry(node_id(node).clone())
            .or_insert_with(|| node_read_deps(node).len().saturating_sub(1)) ==
            0 {
            match &*node.borrow() {
                Node_::Serial(_) => { },
                Node_::SerialRange(n) => {
                    let n = n.borrow();
                    let serial_ident = node_id(n.serial).ident();
                    let ident = n.id.ident();
                    let bytes = n.bytes;
                    let bits = n.bits;
                    code.push(quote!{
                        let mut #ident = #serial_ident.read_len(#bytes, #bits) ?;
                    });
                },
                Node_::Int(n) => {
                    let n = n.borrow();
                    let ident = n.id.ident();
                    code.push(quote!{
                        let #ident = todo !();
                    });
                },
                Node_::String(n) => {
                    let n = n.borrow();
                    let ident = n.id.ident();
                    let serial_ident = node_id(n.serial).ident();
                    let len_ident = n.serial_len.borrow().id.ident();
                    code.push(quote!{
                        let mut #ident = #serial_ident.read_len(#len_ident) ?;
                    });
                },
                Node_::Array(n) => {
                    let n = n.borrow();
                    let ident = n.id.ident();
                    let len_ident = node_id(n.serial_len).ident();
                    let element = n.element.0.borrow();
                    let elem_ident = element.id.ident();
                    let elem_code = builder_read(&*element);
                    code.push(quote!{
                        let mut #ident = vec ![];
                        for _ in 0..len_ident {
                            #elem_code 
                            //. .
                            #ident.push(#elem_ident);
                        }
                    });
                },
                Node_::Enum(n) => {
                    let n = n.borrow();
                    let ident = n.id.ident();
                    let enum_name = n.name;
                    let tag_ident = node_id(n.serial_tag).ident();
                    let mut var_code = vec![];
                    for v in n.variants {
                        let tag = hex::encode(v.tag);
                        let elem_name = v.name;
                        let elem = v.element.0.borrow();
                        let elem_ident = elem.id.ident();
                        let elem_code = builder_read(&*elem);
                        var_code.push(quote!{
                            hex_literal:: hex !(#tag) => {
                                #elem_code #ident = #enum_name:: #elem_name(#elem_ident);
                            },
                        });
                    }
                    code.push(quote!{
                        let #ident;
                        match #tag_ident {
                            #(#var_code) * _ => {
                                return Err("Unknown variant with tag {}", #tag_ident);
                            }
                        };
                    });
                },
                Node_::Option(n) => {
                    let n = n.borrow();
                    let ident = n.id.ident();
                    let switch_ident = node_id(n.serial_switch).ident();
                    let element = n.element.0.borrow();
                    let elem_ident = element.id.ident();
                    let elem_code = builder_read(&element);
                    code.push(quote!{
                        let #ident;
                        if #switch_ident {
                            #elem_code #ident = Some(#elem_ident);
                        }
                        else {
                            #ident = None;
                        };
                    });
                },
                Node_::Const(n) => {
                    let n = n.borrow();
                    let ident = n.id.ident();
                    let value_ident = node_id(n.value).ident();
                    let expect = n.expect;
                    code.push(quote!{
                        if #value_ident != #expect {
                            return Err("Magic mismatch at TODO");
                        }
                    });
                },
                Node_::RustField(n) => { },
                Node_::RustObj(n) => {
                    let n = n.borrow();
                    let obj_ident = n.obj_ident;
                    let ident = n.id.ident();
                    let mut fields = vec![];
                    for f in n.fields {
                        let f = f.borrow();
                        let field_ident = f.field_name.ident();
                        let value_ident = node_id(f.value).ident();
                        fields.push(quote!{
                            #field_ident: #value_ident,
                        });
                    }
                    code.push(quote!{
                        #ident = #obj_ident {
                            #(#fields) *
                        };
                    });
                },
            };
        }
    }
    return quote!(#(#code) *);
}

fn builder_write(b: &Builder_) -> TokenStream {
    let mut seen_count = HashMap::new();
    let mut stack = vec![];
    if let Some(r) = b.serial_root {
        stack.push((r, false));
    }
    let mut code = vec![];
    while let Some((node, first_visit)) = stack.pop() {
        if first_visit {
            stack.push((node, true));
            for dep in node_write_deps(node) {
                stack.push((dep, false));
            }
        } else if *seen_count
            .entry(node_id(node).clone())
            .or_insert_with(|| node_write_deps(node).len().saturating_sub(1)) ==
            0 {
            match &*node.borrow() {
                Node_::Serial(n) => {
                    let n = n.borrow();
                    let serial_ident = n.id.ident();
                    for child in n.children {
                        let child_ident = node_id(child).ident();
                        code.push(quote!{
                            #serial_ident.write(& #child_ident) ?;
                        });
                    }
                },
                Node_::SerialRange(n) => {
                    let n = n.borrow();

                    // TODO branching children/sub-ranges
                    let source_ident = n.id.ident();
                    if let Some(child) = n.rust { } else {
                        for r in n.sub_ranges {
                            // TODO
                        }
                    }
                    code.push(quote!{
                        #dest_ident = #source_ident;
                    });
                },
                Node_::Int(n) => {
                    let n = n.borrow();
                    let dest_ident = n.id.ident();
                    let source_ident = node_id(n.rust).ident();
                    code.push(quote!{
                        let #dest_ident = todo !();
                    });
                },
                Node_::String(n) => {
                    let n = n.borrow();
                    let dest_ident = n.id.ident();
                    let source_ident = node_id(n.rust).ident();
                    let dest_len_ident = node_id(n.serial_len).ident();
                    code.push(quote!{
                        let #dest_len_ident = #source_ident.len();
                        let #dest_ident = #source_ident.as_bytes();
                    });
                },
                Node_::Array(n) => {
                    let n = n.borrow();
                    let source_ident = node_id(n.rust).ident();
                    let dest_ident = n.id.ident();
                    let dest_len_ident = node_id(n.serial_len).ident();
                    let element = n.element.0.borrow();
                    let elem_source_ident = builder_rust_ident;
                    let elem_code = builder_write(&*element);
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
                    let enum_name = n.name;
                    let source_ident = node_id(n.rust).ident();
                    let dest_ident = n.id.ident();
                    let dest_tag_ident = node_id(n.serial_tag).ident();
                    let mut var_code = vec![];
                    for v in n.variants {
                        let tag = hex::encode(v.tag);
                        let variant_name = v.name;
                        let element = v.element.0.borrow();
                        let elem_source_ident = builder_rust_ident;
                        let elem_dest_ident = builder_serial_ident;
                        let elem_code;
                        if !element.external_serial.is_empty() {
                            elem_code = quote!{
                                let mut #elem_dest_ident = vec ![];
                                #elem_source_ident.write(& mut #elem_dest_ident);
                            };
                        } else {
                            elem_code = builder_write(&*element);
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
                    let source_ident = node_id(n.rust.expect("Option unused")).ident();
                    let dest_ident = n.id.ident();
                    let dest_switch_ident = node_id(n.serial_switch).ident();
                    let element = n.element.0.borrow();
                    let elem_source_ident = builder_rust_ident;
                    let elem_dest_ident = builder_serial_ident;
                    let elem_code;
                    if !element.external_serial.is_empty() {
                        elem_code = quote!{
                            let mut #elem_dest_ident = vec ![];
                            #elem_source_ident.write(& mut #elem_dest_ident);
                        };
                    } else {
                        elem_code = builder_write(&*element);
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
                    let field_ident = n.field_name.ident();
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

struct Range {}

impl Builder {
    fn serial_range(&mut self, bytes: usize, bits: usize) -> Range { }

    fn serial_string(&mut self, len: Node) -> Node { }

    fn serial_array(&mut self, len: Node, obj_name: String) -> (Node, Builder) { }

    fn serial_enum_(&mut self, tag: Node, enum_name: String) -> (Node, EnumBuilder) { }

    fn serial_option(&mut self, switch: Node, obj_name: String) -> (Node, Builder) { }

    fn rust_magic(&mut self, serial: Node, value: &[u8]) { }

    fn rust_field(&mut self, serial: Node, name: String) { }

    fn generate(&mut self, write: bool, read: bool) -> TokenStream {
        let self2 = self.0.borrow();
        let type_ident = self2.type_ident;
        let mut fields = vec![];
        for f in &self2.rust_fields {
            let field_name = f.name;
            fields.push(quote!{
                #field_name: #type_,
            });
        }

        quote!{
            pub struct #type_ident {
                #(#fields) *
            }
        };

        drop(fields);
        let methods = vec![];
        if write {
            let code = builder_write(&self2);
            let ident = format_ident!("{}", self2.id);
            let writer_ident = writer_ident();
            methods.push(quote!{
                fn write(&self, #writer_ident: std:: io:: Write) {
                    #code
                }
            });
        }
        if read {
            let code = builder_read(&self2);
            let ident = format_ident!("{}", self2.id);
            let reader_ident = reader_ident();
            methods.push(quote!{
                fn read(#reader_ident: std:: io:: Read) -> #type_ident {
                    #code return #ident;
                }
            });
        }
        return quote!(pub struct #type_ident {
            #(#fields) *
        } impl #type_ident {
            #(#methods) *
        });
    }
}
