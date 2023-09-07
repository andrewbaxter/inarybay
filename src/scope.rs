use std::{
    collections::{
        HashMap,
        BTreeMap,
    },
    rc::Rc,
    cell::RefCell,
};

use gc::{
    GcCell,
    Finalize,
    Trace,
    Gc,
};
use proc_macro2::{
    TokenStream,
    Ident,
};
use quote::{
    quote,
    format_ident,
    ToTokens,
};
use syn::{
    Path,
};
use crate::{
    node::{
        node_serial::{
            NodeSerial,
            NodeSerialSegment,
            NodeSerialSegment_,
            NodeSerialSegmentMut_,
            NodeSerial_,
            NodeSerialMut_,
        },
        node::{
            Node,
            NodeMethods,
            RedirectRef,
        },
        node_const::{
            NodeConst_,
            NodeConst,
            NodeConstMut_,
        },
        node_custom::{
            NodeCustom,
            NodeCustom_,
            NodeCustomMut_,
        },
        node_dynamic_bytes::{
            NodeDynamicBytes,
            NodeDynamicBytes_,
            NodeDynamicBytesMut_,
        },
        node_delimited_bytes::{
            NodeDelimitedBytes,
            NodeDelimitedBytes_,
            NodeDelimitedBytesMut_,
        },
        node_remaining_bytes::{
            NodeRemainingBytes,
            NodeRemainingBytes_,
            NodeSuffixBytesMut_,
        },
        node_enum::{
            NodeEnum,
            NodeEnum_,
            NodeEnumMut_,
        },
        node_fixed_range::{
            NodeFixedRange,
            NodeFixedRange_,
            NodeFixedRangeMut_,
        },
        node_fixed_bytes::{
            NodeFixedBytes,
            NodeFixedBytes_,
            NodeFixedBytesMut_,
        },
        node_int::{
            NodeInt,
            NodeInt_,
            NodeIntMut_,
        },
        node_align::{
            NodeAlign,
            NodeAlign_,
        },
        node_dynamic_array::{
            NodeDynamicArray,
            NodeDynamicArray_,
            NodeDynamicArrayMut_,
        },
        node_object::{
            NodeObj,
            NodeObj_,
            NodeObjMut_,
        },
    },
    util::{
        BVec,
        LateInit,
        ToIdent,
    },
    schema::{
        ReaderBounds,
        Schema,
    },
};

#[derive(PartialEq, Trace, Finalize)]
pub enum Endian {
    Big,
    Little,
}

pub trait IntoByteVec {
    fn get(&self) -> Node;
}

impl IntoByteVec for NodeDynamicBytes {
    fn get(&self) -> Node {
        return self.clone().into();
    }
}

impl IntoByteVec for NodeDelimitedBytes {
    fn get(&self) -> Node {
        return self.clone().into();
    }
}

impl IntoByteVec for NodeRemainingBytes {
    fn get(&self) -> Node {
        return self.clone().into();
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct EscapableParentEnum {
    pub(crate) enum_: NodeEnum,
    pub(crate) variant_name: String,
    pub(crate) parent: Scope,
}

#[derive(Clone, Trace, Finalize)]
pub(crate) enum EscapableParent {
    None,
    Enum(EscapableParentEnum),
}

#[derive(Clone, Trace, Finalize)]
pub(crate) enum SomeEscapableParent {
    Enum(EscapableParentEnum),
}

#[derive(Clone, Copy)]
struct RangeAllocSingle {
    /// Relative to root serial
    start: BVec,
    avail: BVec,
}

struct RangeAllocEnum {
    enum_id: String,
    template_alloc: RangeAllocSingle,
    variants: HashMap<String, Rc<RefCell<RangeAlloc>>>,
}

enum RangeAlloc {
    Unset(RangeAllocSingle),
    Local(Scope, RangeAllocSingle),
    Enum(RangeAllocEnum),
}

#[derive(Trace, Finalize)]
struct Range_ {
    serial: NodeFixedRange,
    #[unsafe_ignore_trace]
    alloc: Rc<RefCell<RangeAlloc>>,
}

/// This represents a sequence of read/written bytes of fixed length.  It can be
/// turned into fixed-width data types (int, bool, float, byte array) via Object
/// methods or further subdivided.
#[derive(Clone)]
pub struct Range(Gc<GcCell<Range_>>);

#[derive(Trace, Finalize)]
pub(crate) struct ScopeMut_ {
    pub(crate) rust_root: Option<Node>,
    pub(crate) escapable_parent: EscapableParent,
    pub(crate) rust_extra_roots: Vec<NodeConst>,
    pub(crate) serial_extra_roots: Vec<Node>,
    pub(crate) has_external_deps: bool,
    #[unsafe_ignore_trace]
    pub(crate) level_ids: BTreeMap<String, Option<Node>>,
}

#[derive(Trace, Finalize)]
pub(crate) struct Scope_ {
    pub(crate) schema: Schema,
    pub(crate) id: String,
    pub(crate) serial_root: NodeSerial,
    pub(crate) mut_: GcCell<ScopeMut_>,
}

/// During (de)serialization there is a tree-based accessibility of certain values.
///  For example, when serializing a Vec, each element is only in scope during that
/// iteration of the loop, so any nodes depending on it must be evaluated within
/// that loop iteration.
///
/// A `Scope` is a collection representing all the nodes in that _scope_.  In the
/// above example, there would be a top level `Scope`, then a `Scope` for the
/// processing within the loop.
///
/// `Scope` has methods for creating nodes that will be evaulated within the scope.
///  In some nesting contexts (enum, object, _not_ array) nodes in an inner scope
/// can refer to nodes in an outer scope.
///
/// The `Scope` also has a defined serial-side and rust-side root which are
/// starting points for the serialization and deserialization graphs respectively.
/// The serial root is automatically defined, but you must call `rust_root` to set
/// the rust-side root with whatever element represents the output of the scope.
#[derive(Clone, Trace, Finalize)]
pub struct Scope(pub(crate) Gc<Scope_>);

impl Scope {
    // # Object creation + setup
    pub(crate) fn new(id: impl Into<String>, schema: &Schema) -> Scope {
        let id = id.into();
        let serial_id = format!("{}__serial", id);
        let serial_root = NodeSerial(Gc::new(NodeSerial_ {
            id: serial_id.clone(),
            id_ident: serial_id.ident().expect("Couldn't convert id into a rust identifier"),
            mut_: GcCell::new(NodeSerialMut_ {
                segments: vec![],
                sub_segments: vec![],
                lifted_serial_deps: BTreeMap::new(),
            }),
        }));
        let out = Scope(Gc::new(Scope_ {
            schema: schema.clone(),
            id: id.clone(),
            serial_root: serial_root,
            mut_: GcCell::new(ScopeMut_ {
                escapable_parent: EscapableParent::None,
                rust_root: None,
                rust_extra_roots: vec![],
                serial_extra_roots: vec![],
                has_external_deps: false,
                level_ids: BTreeMap::new(),
            }),
        }));
        out.take_id(&id, None);
        out.take_id(&serial_id, None);
        return out;
    }

    // # Node generation
    /// Read/write a sequence of bytes with a fixed length from the stream next. See
    /// `Range` for more information on how this can be used.
    pub fn fixed_range(&self, id: impl Into<String>, bytes: usize) -> Range {
        let id = id.into();
        let seg = self.seg(&id);
        let node = NodeFixedRange(Gc::new(NodeFixedRange_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: seg.clone(),
            len_bytes: bytes,
            mut_: GcCell::new(NodeFixedRangeMut_ { rust: BTreeMap::new() }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        seg.0.mut_.borrow_mut().rust = Some(node.clone().into());
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(node.clone().into());
        return Range(Gc::new(GcCell::new(Range_ {
            serial: node,
            alloc: Rc::new(RefCell::new(RangeAlloc::Unset(RangeAllocSingle {
                start: BVec::zero(),
                avail: BVec::bytes(bytes),
            }))),
        })));
    }

    /// Select a subrange of a `Range`.
    pub fn subrange(&self, range: &Range, bytes: usize, bits: usize) -> Range {
        let mut range2 = range.0.as_ref().borrow_mut();
        let using = BVec {
            bytes: bytes,
            bits: bits,
        };
        if using.bits > 8 {
            panic!("Bits should be in excess of bytes (i.e. < 8), but got argument of {}b", using.bits);
        }
        let start = {
            let ancestry = self.get_ancestry_to(&range2.serial);
            self.modify_range(&mut range2, &ancestry, |alloc| {
                if using > alloc.avail {
                    panic!("Range has {} available, but subrange consumes {}", alloc.avail, using);
                }
                let out = alloc.start;
                alloc.start += using;
                alloc.avail -= using;
                return out;
            })
        };
        let out = Range(Gc::new(GcCell::new(Range_ {
            serial: range2.serial.clone(),
            alloc: Rc::new(RefCell::new(RangeAlloc::Unset(RangeAllocSingle {
                start: start,
                avail: using,
            }))),
        })));
        return out;
    }

    /// Treat a fixed range as a fixed byte array in Rust (like `[u8;4]`).  The range
    /// must be byte-aligned and of integer byte length.
    pub fn bytes(&self, id: impl Into<String>, range: Range) -> NodeFixedBytes {
        let id = id.into();
        let mut range2 = range.0.as_ref().borrow_mut();
        let ancestry = self.get_ancestry_to(&range2.serial);
        let using = self.modify_range(&mut range2, &ancestry, |alloc| {
            if alloc.avail == BVec::zero() {
                panic!("Range has no space available");
            }
            let out = alloc.clone();
            alloc.avail = BVec::zero();
            return out;
        });
        if using.start.bits != 0 {
            panic!("Must be byte-aligned but has non-zero bit offset");
        }
        if using.avail.bits != 0 {
            panic!("Must be whole-byte-sized but has non-zero bit length");
        }
        let node = NodeFixedBytes(Gc::new(NodeFixedBytes_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Id is not a valid rust identifier"),
            start: using.start.bytes,
            len: using.avail.bytes,
            rust_type: {
                let len = using.avail.bytes;
                quote!([
                    u8;
                    #len
                ])
            },
            mut_: GcCell::new(NodeFixedBytesMut_ {
                serial: None,
                rust: None,
            }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        self.lift_connect(&ancestry, &range2.serial, node.clone().into(), &mut node.0.mut_.borrow_mut().serial);
        return node;
    }

    /// Treat a fixed range as a an integer.  Currently the range must be 0 bytes and
    /// 0-8 bits, or >= 1 bytes and 0 bits long (only integer-width integers and
    /// <integer width bit fields are supported).  The smallest Rust type that is large
    /// enough to handle all values of the field is selected.
    pub fn int(&self, id: impl Into<String>, range: Range, endian: Endian, signed: bool) -> NodeInt {
        let id = id.into();
        let mut range2 = range.0.as_ref().borrow_mut();
        let ancestry = self.get_ancestry_to(&range2.serial);
        let using = self.modify_range(&mut range2, &ancestry, |alloc| {
            if alloc.avail == BVec::zero() {
                panic!("Range has no space available");
            }
            let out = alloc.clone();
            alloc.avail = BVec::zero();
            return out;
        });
        let mut rust_bits = (using.avail.bytes * 8 + using.avail.bits).next_power_of_two();
        if rust_bits < 8 {
            rust_bits = 8;
        }
        if rust_bits > 64 {
            panic!("Rust doesn't support ints with >64b width");
        }
        let sign_prefix;
        if signed {
            sign_prefix = "i";
        } else {
            sign_prefix = "u";
        }
        let rust_type = format_ident!("{}{}", sign_prefix, rust_bits).into_token_stream();
        let node = NodeInt(Gc::new(NodeInt_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Id could not be turned into an identifier"),
            start: using.start,
            len: using.avail,
            signed: signed,
            endian: endian,
            rust_type: rust_type,
            rust_bytes: rust_bits / 8,
            mut_: GcCell::new(NodeIntMut_ {
                serial: None,
                rust: None,
            }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        self.lift_connect(&ancestry, &range2.serial, node.clone().into(), &mut node.0.mut_.borrow_mut().serial);
        return node;
    }

    /// Align the next serial data read/written.  For example, if a dynamic bytes
    /// segment reads 5 bytes, then you align to 4 bytes, the next read will be at
    /// offset 8. This is local to the current object - if a nested object starts
    /// reading at offset 1, align to 4 will make the subsequent read at global offset
    /// 5.
    ///
    /// `shift` is added to the offset (shifting input phase) before doing alignment
    /// calculation (effectively a negative shift).  For example, a shift of 1 and
    /// alignment of 4 will make the next read/write at 3, then 7, etc.
    ///
    /// The minimum alignment is 1, minimum shift is 0.  A 0 alignment will result in
    /// divide by zero.
    pub fn align(&self, id: impl Into<String>, shift: usize, alignment: usize) {
        let id = id.into();
        let seg = self.seg(&id);
        let serial = NodeAlign(Gc::new(NodeAlign_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: seg.clone(),
            shift: shift,
            alignment: alignment,
        }));
        self.take_id(&id, None);
        seg.0.mut_.borrow_mut().rust = Some(serial.clone().into());
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(serial.clone().into());
    }

    /// Read/write a sequence of bytes whose length is determined dynamically by a
    /// previous integer.
    pub fn dynamic_bytes(&self, id: impl Into<String>, len: NodeInt) -> NodeDynamicBytes {
        let id = id.into();
        let serial = self.seg(&id);
        let node = NodeDynamicBytes(Gc::new(NodeDynamicBytes_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial,
            mut_: GcCell::new(NodeDynamicBytesMut_ {
                serial_len: None,
                rust: None,
            }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(node.clone().into());
        self.lift_connect(
            &self.get_ancestry_to(&len),
            &len,
            node.clone().into(),
            &mut node.0.mut_.borrow_mut().serial_len,
        );
        return node;
    }

    /// Read/write a sequence of bytes until the specified delimiter sequence of bytes.
    pub fn delimited_bytes(&self, id: impl Into<String>, delimiter: &[u8]) -> NodeDelimitedBytes {
        self.0.schema.0.borrow_mut().reader_bounds = ReaderBounds::Buffered;
        let id = id.into();
        let serial = self.seg(&id);
        let mut delim_els = vec![];
        for b in delimiter {
            delim_els.push(quote!(#b));
        }
        let node = NodeDelimitedBytes(Gc::new(NodeDelimitedBytes_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial.clone(),
            delim_len: delimiter.len(),
            delim_bytes: quote!(&[#(#delim_els,) *]),
            mut_: GcCell::new(NodeDelimitedBytesMut_ { rust: None }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(node.clone().into());
        serial.0.mut_.borrow_mut().rust = Some(node.clone().into());
        return node;
    }

    /// Read/write until the end of the serial data.
    pub fn remaining_bytes(&self, id: impl Into<String>) -> NodeRemainingBytes {
        let id = id.into();
        let serial = self.seg(&id);
        let node = NodeRemainingBytes(Gc::new(NodeRemainingBytes_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial.clone(),
            mut_: GcCell::new(NodeSuffixBytesMut_ { rust: None }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(node.clone().into());
        serial.0.mut_.borrow_mut().rust = Some(node.clone().into());
        return node;
    }

    /// Read/write an array of objects, with the length (number of objects) specified
    /// by a previous integer value.
    pub fn dynamic_array(&self, id: impl Into<String>, len: NodeInt) -> (NodeDynamicArray, Scope) {
        let id = id.into();
        let serial = self.seg(&id);
        let scope = Scope::new(&format!("{}__scope", id), &self.0.schema);
        let node = NodeDynamicArray(Gc::new(NodeDynamicArray_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial,
            element: scope.clone(),
            mut_: GcCell::new(NodeDynamicArrayMut_ {
                serial_len: None,
                rust: None,
            }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(node.clone().into());
        self.lift_connect(
            &self.get_ancestry_to(&len),
            &len,
            node.clone().into(),
            &mut node.0.mut_.borrow_mut().serial_len,
        );
        return (node, scope);
    }

    /// Inject a custom node.
    ///
    /// * `rust_type` is the end result of the read.
    ///
    /// * `serial` is the list of input nodes.
    ///
    /// * `read_code` is a function that takes the input node identifiers and an output
    ///   node identifier and returns statements that produces the Rust type from the
    ///   inputs and stores it in the output node identifier.
    ///
    /// * `write_code` does the opposite, taking the Rust value and producing values for
    ///   each input node.
    ///
    /// See `string_utf8` for an example.
    pub fn custom(
        &self,
        id: impl Into<String>,
        rust_type: TokenStream,
        read_code: impl Fn(&Vec<Ident>, &TokenStream) -> TokenStream + 'static,
        write_code: impl Fn(&TokenStream, &Vec<Ident>) -> TokenStream + 'static,
        serial: Vec<Node>,
    ) -> NodeCustom {
        let id = id.into();
        syn::parse2::<Path>(rust_type.clone()).expect("Rust type isn't a valid type");
        let node = NodeCustom(Gc::new(NodeCustom_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            rust_type: rust_type,
            read_code: Box::new(read_code),
            write_code: Box::new(write_code),
            mut_: GcCell::new(NodeCustomMut_ {
                serial: vec![],
                rust: None,
            }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        for arg in serial {
            node.0.mut_.borrow_mut().serial.push(None);
            self.lift_connect(
                &self.get_ancestry_to(&arg),
                &arg,
                node.clone().into(),
                node.0.mut_.borrow_mut().serial.last_mut().unwrap(),
            );
        }
        return node;
    }

    /// Turn an integer into a boolean value.  0 is false, all other values are true.
    /// When writing, 1 will be written for true values.
    pub fn bool(&self, id: impl Into<String>, serial: NodeInt) -> NodeCustom {
        return self.custom(
            //. .
            id,
            quote!(bool),
            |s, d| quote!{
                #d = #(#s) *!= 0;
            },
            move |s, d| quote!{
                #(#d) *
                //. .
                = match #s {
                    true => 1,
                    false => 0,
                };
            },
            vec![serial.into()],
        );
    }

    /// Turn a fixed-length sequence of bytes into a floating point number. Follows
    /// IEEE754 per Rust's f32/f64 conversion methods.
    pub fn float(&self, id: impl Into<String>, range: Range, endian: Endian) -> NodeCustom {
        let id = id.into();
        let bytes = self.bytes(format!("{}__bytes", id), range);
        let rust_type;
        match bytes.0.len {
            4 => {
                rust_type = quote!(f32);
            },
            8 => {
                rust_type = quote!(f64);
            },
            b => {
                panic!("Unsupported float width {}, must be 4 or 8", b);
            },
        }
        let read_method;
        let write_method;
        match endian {
            Endian::Big => {
                read_method = quote!(from_be_bytes);
                write_method = quote!(to_be_bytes);
            },
            Endian::Little => {
                read_method = quote!(from_le_bytes);
                write_method = quote!(to_le_bytes);
            },
        };
        return self.custom(
            //. .
            id,
            rust_type.clone(),
            {
                let rust_type = rust_type.clone();
                move |s, d| quote!{
                    #d = #rust_type:: #read_method(#(#s) *);
                }
            },
            move |s, d| quote!{
                #(#d) *
                //. .
                = #s.#write_method();
            },
            vec![bytes.into()],
        );
    }

    /// Treat a dynamic-length byte sequence as a UTF-8 string (`String` in Rust).
    pub fn string_utf8(&self, id: impl Into<String>, serial: impl IntoByteVec) -> NodeCustom {
        let id = id.into();
        let err = format!("Error parsing utf8 string in node {}", id);
        return self.custom(
            //. .
            id,
            quote!(String),
            move |s, d| quote!{
                #d = String:: from_utf8(#(#s) *).errorize(#err) ?;
            },
            move |s, d| quote!{
                #(#d) *
                //. .
                = #s.into_bytes();
            },
            vec![serial.get()],
        );
    }

    /// On deserialization, confirm that the read `serial` value equals `value`.  On
    /// serialization, feed `value` into the pipeline.  This value is not available
    /// post-deserialization, it is only involved in parsing mechanics and checking.
    /// `value` should be a Rust expression that is compatible with the value of
    /// `serial`, ex: if `serial` is a bool, value could be `quote!(true)` or
    /// `quote!(false)`.
    pub fn const_(&self, id: impl Into<String>, serial: impl Into<Node>, value: TokenStream) {
        let id = id.into();
        self.take_id(&id, None);
        let serial: Node = serial.into();
        let rust = NodeConst(Gc::new(NodeConst_ {
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            expect: value,
            mut_: GcCell::new(NodeConstMut_ { serial: None }),
        }));
        self.0.mut_.borrow_mut().rust_extra_roots.push(rust.clone());
        self.lift_connect(
            &self.get_ancestry_to(&serial),
            &serial,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial,
        );
    }

    /// Read/write a rust struct.
    pub fn object(&self, id: impl Into<String>, obj_name: impl Into<String>) -> NodeObj {
        let id = id.into();
        let name = obj_name.into();
        let type_name_ident = name.ident().expect("Couldn't convert name into a rust identifier");
        let node = NodeObj(Gc::new(NodeObj_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            type_name: name.clone(),
            type_name_ident: type_name_ident.clone(),
            mut_: GcCell::new(NodeObjMut_ {
                fields: vec![],
                type_attrs: vec![],
            }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        self.0.schema.0.as_ref().borrow_mut().objects.entry(name).or_insert_with(Vec::new).push(node.clone());
        return node;
    }

    /// Read/write an enumeration, where the parsing variation is determined by the
    /// previously defined tag value.  The tag can be any `match`-able data type (int,
    /// bool, float, string, byte array).
    pub fn enum_(&self, id: impl Into<String>, tag: impl Into<Node>, enum_name: impl Into<String>) -> NodeEnum {
        let id = id.into();
        let tag = tag.into();
        let serial = self.seg(&id);
        let enum_name = enum_name.into();
        let enum_name_ident = enum_name.ident().expect("Couldn't convert enum name into a rust identifier");
        let node = NodeEnum(Gc::new(NodeEnum_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            type_name: enum_name.clone(),
            type_name_ident: enum_name_ident,
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial,
            mut_: GcCell::new(NodeEnumMut_ {
                serial_tag: None,
                variants: vec![],
                default_variant: None,
                rust: None,
                external_deps: BTreeMap::new(),
                type_attrs: vec![],
            }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(node.clone().into());
        self.0.schema.0.as_ref().borrow_mut().enums.entry(enum_name).or_insert_with(Vec::new).push(node.clone());
        self.lift_connect(
            &self.get_ancestry_to(&tag),
            &tag,
            node.clone().into(),
            &mut node.0.mut_.borrow_mut().serial_tag,
        );
        return node;
    }

    /// Set the rust-side root for (de)serialization.  During serialization, this will
    /// be the argument to the `write` function.  During deserialization, this will be
    /// the return value.
    pub fn rust_root(&self, rust: impl Into<Node>) {
        let mut self2 = self.0.mut_.borrow_mut();
        match &self2.rust_root {
            Some(o) => {
                panic!("A rust root has already been defined for this scope: {}", o.id());
            },
            None => { },
        }
        self2.rust_root = Some(rust.into());
    }

    // # Internal
    pub(crate) fn take_id(&self, id: &String, node: Option<Node>) {
        let mut at = self.clone();
        loop {
            if at.0.mut_.borrow().level_ids.contains_key(id) {
                panic!("Id {} already used in scope {}", id, at.0.id);
            }
            let escapable_parent = at.0.mut_.borrow().escapable_parent.clone();
            match &escapable_parent {
                EscapableParent::None => {
                    break;
                },
                EscapableParent::Enum(e) => {
                    at = e.parent.clone();
                },
            };
        }
        self.0.mut_.borrow_mut().level_ids.insert(id.clone(), node);
    }

    pub(crate) fn get_rust_root(&self) -> Node {
        return self
            .0
            .mut_
            .borrow()
            .rust_root
            .as_ref()
            .expect(&format!("Scope {} has no defined rust root", self.0.id))
            .clone();
    }

    fn seg(&self, id: &str) -> NodeSerialSegment {
        let id = format!("{}__serial_seg", id);
        let node = NodeSerialSegment(Gc::new(NodeSerialSegment_ {
            scope: self.clone(),
            id: id.clone(),
            id_ident: id.ident().expect("Couldn't convert id into a rust identifier"),
            serial_root: self.0.serial_root.clone().into(),
            serial_before: self.0.serial_root.0.mut_.borrow().segments.last().cloned(),
            mut_: GcCell::new(NodeSerialSegmentMut_ { rust: None }),
        }));
        self.take_id(&id, Some(node.clone().into()));
        self.0.serial_root.0.mut_.borrow_mut().segments.push(node.clone());
        return node;
    }

    pub(crate) fn get_ancestry_to(&self, serial: &dyn NodeMethods) -> Vec<SomeEscapableParent> {
        let mut ancestry = vec![];
        let mut at = self.clone();
        loop {
            if at.0.id == serial.scope().0.id {
                break;
            }
            let nesting_parent = at.0.mut_.borrow().escapable_parent.clone();
            match &nesting_parent {
                EscapableParent::None => {
                    panic!(
                        "Serial-side dependency {} is not from any containing scope; maybe this is within an array context and so can't depend on higher scopes",
                        serial.id()
                    );
                },
                EscapableParent::Enum(e) => {
                    ancestry.push(SomeEscapableParent::Enum(e.clone()));
                    at = e.parent.clone();
                },
            };
        }
        ancestry.reverse();
        return ancestry;
    }

    fn modify_range<
        T,
    >(
        &self,
        range: &mut Range_,
        ancestry: &Vec<SomeEscapableParent>,
        f: impl FnOnce(&mut RangeAllocSingle) -> T,
    ) -> T {
        let mut range_level = range.alloc.clone();

        // Trace ancestry from shared root up to current level
        for ancestor_level in ancestry {
            match ancestor_level {
                SomeEscapableParent::Enum(ancestor_enum) => {
                    let range_level1 = range_level.clone();
                    let mut range_level2 = range_level1.borrow_mut();
                    let range_level3 = &mut *range_level2;
                    match range_level3 {
                        RangeAlloc::Unset(alloc) => {
                            let alloc = alloc.clone();
                            *range_level3 = RangeAlloc::Enum(RangeAllocEnum {
                                enum_id: ancestor_enum.enum_.0.type_name.clone(),
                                template_alloc: alloc,
                                variants: HashMap::new(),
                            });
                            match range_level3 {
                                RangeAlloc::Enum(e) => {
                                    e
                                        .variants
                                        .insert(
                                            ancestor_enum.variant_name.clone(),
                                            Rc::new(RefCell::new(RangeAlloc::Unset(alloc))),
                                        );
                                    range_level = e.variants.get(&ancestor_enum.variant_name).unwrap().clone();
                                },
                                _ => unreachable!(),
                            };
                        },
                        RangeAlloc::Local(obj, _) => panic!(
                            "This range is already used by something else in this scope: {}",
                            obj.0.id
                        ),
                        RangeAlloc::Enum(range_enum) => {
                            if range_enum.enum_id != ancestor_enum.enum_.0.type_name {
                                panic!(
                                    "This range is already used another enum that's not an ancestor: {} (ancestor at that level is {})",
                                    range_enum.enum_id,
                                    ancestor_enum.enum_.0.type_name
                                );
                            }
                            let old =
                                range_enum
                                    .variants
                                    .insert(
                                        ancestor_enum.variant_name.clone(),
                                        Rc::new(RefCell::new(RangeAlloc::Unset(range_enum.template_alloc))),
                                    );
                            if old.is_some() {
                                // Different variant with same name, should be caught elsewhere?
                                unreachable!();
                            }
                            range_level = range_enum.variants.get_mut(&ancestor_enum.variant_name).unwrap().clone();
                        },
                    }
                },
            }
        }
        let mut range_level = range_level.borrow_mut();
        let range_level = &mut *range_level;
        match range_level {
            RangeAlloc::Unset(alloc) => {
                *range_level = RangeAlloc::Local(self.clone(), *alloc);
                match range_level {
                    RangeAlloc::Local(_, alloc) => {
                        return f(alloc);
                    },
                    _ => unreachable!(),
                };
            },
            RangeAlloc::Local(_, alloc) => {
                return f(alloc);
            },
            RangeAlloc::Enum(enum_) => {
                panic!("This range is already used by something else in this scope: enum {}", enum_.enum_id);
            },
        }
    }

    pub(crate) fn lift_connect<
        T: Clone + NodeMethods + Into<Node> + Trace + Finalize,
    >(
        &self,
        ancestry: &Vec<SomeEscapableParent>,
        serial: &T,
        rust: Node,
        rust_field: &mut LateInit<RedirectRef<T, Node>>,
    ) {
        if ancestry.is_empty() {
            serial.set_rust(rust.clone());
            *rust_field = Some(RedirectRef::new(serial.clone()));
        } else {
            for (i, level) in ancestry.iter().enumerate() {
                match level {
                    SomeEscapableParent::Enum(level) => {
                        if i == 0 {
                            // 1A
                            serial.set_rust(level.enum_.clone().into());

                            // 1B
                            level.enum_.0.mut_.borrow_mut().external_deps.insert(serial.id(), serial.clone().into());
                        } else {
                            let level_serial_root = &level.parent.0.serial_root;

                            // 2A
                            level_serial_root.0.mut_.borrow_mut().lifted_serial_deps.insert(level.enum_.0.id.clone(), level.enum_.clone().into());

                            // 2B
                            level
                                .enum_
                                .0
                                .mut_
                                .borrow_mut()
                                .external_deps
                                .insert(level_serial_root.0.id.clone(), level_serial_root.clone().into());

                            // 4
                            level.parent.0.mut_.borrow_mut().has_external_deps = true;
                        }
                    },
                }
            }

            // 4
            rust.0.scope().0.mut_.borrow_mut().has_external_deps = true;

            // 3A
            self.0.serial_root.0.mut_.borrow_mut().lifted_serial_deps.insert(rust.id(), rust.clone());

            // 3B
            *rust_field = Some(RedirectRef {
                primary: serial.clone(),
                redirect: Some(self.0.serial_root.clone().into()),
            });
        }
    }
}
