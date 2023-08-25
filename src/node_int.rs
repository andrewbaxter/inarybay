use proc_macro2::{
    TokenStream,
    Ident,
};
use crate::{
    util::{
        S,
        ToIdent,
        Coord,
        RedirectRef,
    },
    node_fixed_range::NodeSerialFixedRange,
    node::Node,
    object::WeakObj,
};
use quote::{
    quote,
    format_ident,
};

#[derive(PartialEq)]
pub(crate) enum Endian {
    Big,
    Little,
}

pub(crate) struct NodeIntArgs {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial: RedirectRef<S<NodeSerialFixedRange>, Node>,
    pub(crate) start: Coord,
    pub(crate) len: Coord,
    pub(crate) signed: bool,
    pub(crate) endian: Endian,
}

pub(crate) struct NodeInt {
    pub(crate) scope: WeakObj,
    pub(crate) id: String,
    pub(crate) serial: RedirectRef<S<NodeSerialFixedRange>, Node>,
    pub(crate) start: Coord,
    pub(crate) len: Coord,
    pub(crate) signed: bool,
    pub(crate) endian: Endian,
    pub(crate) rust: Option<RedirectRef<Node, Node>>,
    // Computed
    pub(crate) rust_bits: usize,
    pub(crate) rust_type: Ident,
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
        return NodeInt {
            scope: args.scope,
            id: args.id,
            serial: args.serial,
            start: args.start,
            len: args.len,
            signed: args.signed,
            endian: args.endian,
            rust: None,
            rust_type: format_ident!("{}{}", sign_prefix, rust_bits),
            rust_bits: rust_bits,
        };
    }

    pub(crate) fn generate_read(&self) -> TokenStream {
        let dest_ident = self.id.ident();
        let source_ident = self.serial.primary.borrow().id.ident();
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
            let rust_type = self.rust_type;
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
            let rust_type = self.rust_type;
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

    pub(crate) fn generate_write(&self) -> TokenStream {
        let source_ident = self.rust.expect("").primary.id().ident();
        let dest_ident = self.serial.primary.borrow().id.ident();
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
            let rust_type = self.rust_type;
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
    pub(crate) fn read_deps(&self) -> Vec<Node> {
    }
    pub(crate) fn write_deps(&self) -> Vec<Node> {
    }
    pub(crate) fn write_default(&self) -> TokenStream {
    }
}
