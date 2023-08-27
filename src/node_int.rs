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
    object::Object,
    derive_forward_node_methods,
};
use quote::{
    quote,
    format_ident,
};

#[derive(PartialEq, Trace, Finalize)]
pub enum Endian {
    Big,
    Little,
}

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
    pub(crate) rust_bits: usize,
    #[unsafe_ignore_trace]
    pub(crate) rust_type: Ident,
}

impl NodeMethods for NodeInt_ {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().serial.dep();
    }

    fn generate_read(&self) -> TokenStream {
        let dest_ident = self.id.ident();
        let source_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.0.id.ident();
        if self.len.bits <= 8 {
            if self.start.bits + self.len.bits > 8 {
                panic!();
            }
            let serial_start = self.start.bytes;
            let serial_offset = self.start.bits;
            let mut serial_mask = 0u8;
            for _ in 0 .. self.len.bits {
                serial_mask = serial_mask * 2 + 1;
            }
            let rust_type = &self.rust_type;
            return quote!{
                let #dest_ident = #rust_type:: from_ne_bytes(
                    [((#source_ident[#serial_start] >> #serial_offset) & #serial_mask)]
                );
            };
        } else {
            if self.start.bits != 0 {
                panic!();
            }
            if self.len.bits % 8 != 0 {
                panic!();
            }
            let serial_start = self.start.bytes;
            let serial_bytes = self.len.bits / 8;
            let rust_type = &self.rust_type;
            let method;
            match self.endian {
                Endian::Big => method = format_ident!("from_be_bytes"),
                Endian::Little => method = format_ident!("from_le_bytes"),
            };
            let mut out = quote!(#source_ident[#serial_start..#serial_start + #serial_bytes]);
            if self.len.bits != self.rust_bits {
                let rust_bytes = self.rust_bits / 8;
                match self.endian {
                    Endian::Big => {
                        let endian_pad_offset = rust_bytes - serial_bytes;
                        out = quote!({
                            let mut temp =[
                                0u8;
                                #rust_bytes
                            ];
                            temp[#endian_pad_offset..#rust_bytes].copy_from_slice(& #out);
                            temp
                        });
                    },
                    Endian::Little => {
                        out = quote!({
                            let mut temp =[
                                0u8;
                                #rust_bytes
                            ];
                            temp.copy_from_slice(& #out);
                            temp
                        });
                    },
                }
            } else {
                out = quote!(#out.try_into().unwrap());
            }
            return quote!{
                let dest_ident = #rust_type:: #method(#out);
            };
        }
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.mut_.borrow().rust.dep();
    }

    fn generate_write(&self) -> TokenStream {
        let source_ident = self.id.ident();
        let dest_ident = self.mut_.borrow().serial.as_ref().unwrap().primary.0.id.ident();
        if self.len.bits <= 8 {
            if self.start.bits + self.len.bits > 8 {
                panic!();
            }
            let serial_start = self.start.bytes;
            let serial_offset = self.start.bits;
            let mut serial_mask = 0u8;
            for _ in 0 .. self.len.bits {
                serial_mask = serial_mask * 2 + 1;
            }
            let rust_type = &self.rust_type;
            return quote!{
                #dest_ident[
                    #serial_start
                ] |=(#rust_type:: from_ne_bytes([#source_ident]) & #serial_mask) << #serial_offset;
            };
        } else {
            if self.start.bits != 0 {
                panic!();
            }
            if self.len.bits % 8 != 0 {
                panic!();
            }
            let serial_start = self.start.bytes;
            let serial_bytes = self.len.bits / 8;
            let method;
            match self.endian {
                Endian::Big => method = format_ident!("to_be_bytes"),
                Endian::Little => method = format_ident!("to_le_bytes"),
            };
            let mut out = quote!(#source_ident.#method());
            if self.len.bits != self.rust_bits {
                let rust_bytes = self.rust_bits / 8;
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
}

impl NodeInt {
    pub(crate) fn new(args: NodeIntArgs) -> NodeInt {
        let mut rust_bits = args.len.bits.next_power_of_two();
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
            rust_bits: rust_bits,
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
