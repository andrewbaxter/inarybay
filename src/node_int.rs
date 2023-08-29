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
use crate::{
    util::{
        ToIdent,
        BVec,
        LateInit,
    },
    node_fixed_range::NodeFixedRange,
    node::{
        Node,
        RedirectRef,
        NodeMethods,
        ToDep,
    },
    object::{
        Object,
        Endian,
    },
    derive_forward_node_methods,
    schema::GenerateContext,
};
use quote::{
    quote,
    format_ident,
    ToTokens,
};

pub(crate) struct NodeIntArgs {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) start: BVec,
    pub(crate) len: BVec,
    pub(crate) signed: bool,
    pub(crate) endian: Endian,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeIntMut_ {
    pub(crate) serial: LateInit<RedirectRef<NodeFixedRange, Node>>,
    pub(crate) rust: Option<Node>,
}

#[derive(Trace, Finalize)]
pub(crate) struct NodeInt_ {
    pub(crate) scope: Object,
    pub(crate) id: String,
    pub(crate) start: BVec,
    pub(crate) len: BVec,
    pub(crate) signed: bool,
    pub(crate) endian: Endian,
    pub(crate) mut_: GcCell<NodeIntMut_>,
    // Computed
    pub(crate) rust_bytes: usize,
    #[unsafe_ignore_trace]
    pub(crate) rust_type: Ident,
}

impl NodeMethods for NodeInt_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().serial.dep();
    }

    fn generate_read(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let dest_ident = self.id.ident();
        let source_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.0.id.ident();
        if self.len.bytes == 0 {
            if self.start.bits + self.len.bits > 8 {
                panic!();
            }
            let serial_start = self.start.bytes;
            let mut out = quote!(#source_ident[#serial_start]);
            let serial_offset = self.start.bits;
            if serial_offset > 0 {
                out = quote!((#out >> #serial_offset));
            }
            let mut serial_mask = 0u8;
            for _ in 0 .. self.len.bits {
                serial_mask = serial_mask * 2 + 1;
            }
            out = quote!((#out & #serial_mask));
            let rust_type = &self.rust_type;
            return quote!{
                let #dest_ident = #rust_type:: from_ne_bytes([#out]);
            };
        } else {
            if self.start.bits != 0 {
                panic!();
            }
            if self.len.bits % 8 != 0 {
                panic!();
            }
            let serial_start = self.start.bytes;
            let serial_bytes = self.len.bytes;
            let rust_type = &self.rust_type;
            let method;
            match self.endian {
                Endian::Big => method = format_ident!("from_be_bytes"),
                Endian::Little => method = format_ident!("from_le_bytes"),
            };
            let mut out = quote!(#source_ident[#serial_start..#serial_start + #serial_bytes]);
            if self.len.bytes != self.rust_bytes {
                let rust_bytes = self.rust_bytes;
                match self.endian {
                    Endian::Big => {
                        let endian_pad_offset = rust_bytes - serial_bytes;
                        out = quote!({
                            let #source_ident =& #out;
                            let mut temp = if #source_ident[0] &(1u8 << 7) > 0 {
                                [
                                    255u8;
                                    #rust_bytes
                                ]
                            }
                            else {
                                [
                                    0u8;
                                    #rust_bytes
                                ]
                            };
                            temp[#endian_pad_offset..#rust_bytes].copy_from_slice(& #source_ident);
                            temp
                        });
                    },
                    Endian::Little => {
                        out = quote!({
                            let #source_ident =& #out;
                            let mut temp = if #source_ident[#serial_bytes - 1] &(1u8 << 7) > 0 {
                                [
                                    255u8;
                                    #rust_bytes
                                ]
                            }
                            else {
                                [
                                    0u8;
                                    #rust_bytes
                                ]
                            };
                            temp[0..#serial_bytes].copy_from_slice(& #source_ident);
                            temp
                        });
                    },
                }
            } else {
                out = quote!(#out.try_into().unwrap());
            }
            return quote!{
                let #dest_ident = #rust_type:: #method(#out);
            };
        }
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self, _gen_ctx: &GenerateContext) -> TokenStream {
        let source_ident = self.id.ident();
        let dest_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.0.id.ident();
        if self.len.bytes == 0 {
            if self.start.bits + self.len.bits > 8 {
                panic!();
            }
            let mut serial_mask = 0u8;
            for _ in 0 .. self.len.bits {
                serial_mask = serial_mask * 2 + 1;
            }
            let mut out = quote!((u8:: from_ne_bytes([* #source_ident]) & #serial_mask));
            let serial_offset = self.start.bits;
            if serial_offset > 0 {
                out = quote!((#out << #serial_offset));
            }
            let serial_start = self.start.bytes;
            return quote!{
                #dest_ident[#serial_start] |= #out;
            };
        } else {
            if self.start.bits != 0 {
                panic!();
            }
            if self.len.bits % 8 != 0 {
                panic!();
            }
            let serial_start = self.start.bytes;
            let serial_bytes = self.len.bytes;
            let method;
            match self.endian {
                Endian::Big => method = format_ident!("to_be_bytes"),
                Endian::Little => method = format_ident!("to_le_bytes"),
            };
            let mut out = quote!(#source_ident.#method());
            if self.len.bytes != self.rust_bytes {
                let rust_bytes = self.rust_bytes;
                match self.endian {
                    Endian::Big => {
                        let endian_pad_offset = rust_bytes - serial_bytes;
                        out = quote!(#out[#endian_pad_offset..#rust_bytes]);
                    },
                    Endian::Little => {
                        out = quote!(#out[0..#serial_bytes]);
                    },
                }
            }
            return quote!{
                #dest_ident[#serial_start..#serial_start + #serial_bytes].copy_from_slice(& #out);
            };
        }
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

    fn scope(&self) -> Object {
        return self.scope.clone();
    }

    fn id(&self) -> String {
        return self.id.clone();
    }

    fn rust_type(&self) -> TokenStream {
        return self.rust_type.clone().into_token_stream();
    }
}

impl NodeInt {
    pub(crate) fn new(args: NodeIntArgs) -> NodeInt {
        let mut rust_bits = (args.len.bytes * 8 + args.len.bits).next_power_of_two();
        if rust_bits < 8 {
            rust_bits = 8;
        }
        if rust_bits > 64 {
            panic!("Rust doesn't support ints with >64b width");
        }
        let sign_prefix;
        if args.signed {
            sign_prefix = "i";
        } else {
            sign_prefix = "u";
        }
        return NodeInt(Gc::new(NodeInt_ {
            scope: args.scope,
            id: args.id,
            start: args.start,
            len: args.len,
            signed: args.signed,
            endian: args.endian,
            rust_type: format_ident!("{}{}", sign_prefix, rust_bits),
            rust_bytes: rust_bits / 8,
            mut_: GcCell::new(NodeIntMut_ {
                serial: None,
                rust: None,
            }),
        }));
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct NodeInt(pub(crate) Gc<NodeInt_>);

impl Into<Node> for NodeInt {
    fn into(self) -> Node {
        return Node(crate::node::Node_::Int(self));
    }
}

derive_forward_node_methods!(NodeInt);
