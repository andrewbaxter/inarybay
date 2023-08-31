use std::{
    collections::{
        HashMap,
        BTreeMap,
        HashSet,
    },
    cell::{
        RefCell,
    },
    rc::Rc,
    vec,
};
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
use quote::{
    quote,
};
use crate::{
    util::{
        BVec,
        LateInit,
    },
    node_serial::{
        NodeSerial,
        NodeSerialSegment,
        NodeSerial_,
        NodeSerialSegment_,
        NodeSerialMut_,
        NodeSerialSegmentMut_,
    },
    node_rust::{
        NodeRustObj,
        NodeRustField,
        NodeRustObj_,
        NodeRustField_,
        NodeRustObjMut_,
    },
    node_const::{
        NodeConst,
        NodeConst_,
        NodeConstMut_,
    },
    schema::{
        Schema,
        ReaderBounds,
    },
    node::{
        Node,
        RedirectRef,
        NodeMethods,
    },
    node_fixed_range::{
        NodeFixedRange,
        NodeFixedRange_,
        NodeFixedRangeMut_,
    },
    node_int::{
        NodeInt,
        NodeIntArgs,
    },
    node_dynamic_bytes::{
        NodeDynamicBytes,
        NodeDynamicBytes_,
        NodeDynamicBytesMut_,
    },
    node_dynamic_array::{
        NodeDynamicArray,
        NodeDynamicArray_,
        NodeDynamicArrayMut_,
    },
    node_enum::{
        NodeEnum,
        EnumVariant,
        NodeEnum_,
        EnumDefaultVariant,
        NodeEnumDummy,
        NodeEnumDummy_,
        NodeEnumDummyMut_,
    },
    node_fixed_bytes::{
        NodeFixedBytes,
        NodeFixedBytesArgs,
    },
    node_delimited_bytes::{
        NodeDelimitedBytes,
        NodeDelimitedBytesMut_,
        NodeDelimitedBytes_,
    },
    node_remaining_bytes::{
        NodeRemainingBytes,
        NodeRemainingBytes_,
        NodeSuffixBytesMut_,
    },
    node_custom::{
        NodeCustom,
        NodeCustom_,
        NodeCustomMut_,
    },
    node_align::{
        NodeAlign,
        NodeAlign_,
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
    enum_: NodeEnum,
    variant_name: String,
    parent: Object,
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

#[derive(Trace, Finalize)]
pub(crate) struct ObjectMut_ {
    pub(crate) escapable_parent: EscapableParent,
    pub(crate) rust_extra_roots: Vec<NodeConst>,
    pub(crate) serial_extra_roots: Vec<Node>,
    pub(crate) has_external_deps: bool,
    #[unsafe_ignore_trace]
    pub(crate) type_attrs: Vec<TokenStream>,
    seen_ids: HashSet<String>,
}

#[derive(Trace, Finalize)]
pub(crate) struct Object_ {
    pub(crate) schema: Schema,
    pub(crate) id: String,
    pub(crate) serial_root: NodeSerial,
    pub(crate) rust_root: NodeRustObj,
    pub(crate) mut_: GcCell<ObjectMut_>,
}

/// This is a builder for an object that can be (de)serialized.  You can create
/// these via Schema.
#[derive(Clone, Trace, Finalize)]
pub struct Object(pub(crate) Gc<Object_>);

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
    Local(Object, RangeAllocSingle),
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

/// This is a builder for an enum-like parse of a binary structure (outputs an enum
/// in Rust).
pub struct Enum {
    schema: Schema,
    obj: Object,
    enum_: NodeEnum,
}

impl Object {
    // # Object creation + setup
    pub(crate) fn new(id: impl Into<String>, schema: &Schema, name: String) -> Object {
        let id = id.into();
        let serial_id = format!("{}__serial", id);
        let rust_id = format!("{}__rust", id);
        let serial_root = NodeSerial(Gc::new(NodeSerial_ {
            id: serial_id.clone(),
            mut_: GcCell::new(NodeSerialMut_ {
                segments: vec![],
                sub_segments: vec![],
                lifted_serial_deps: BTreeMap::new(),
            }),
        }));
        let rust_root = NodeRustObj(Gc::new(NodeRustObj_ {
            id: rust_id.clone(),
            type_name: name.clone(),
            mut_: GcCell::new(NodeRustObjMut_ { fields: vec![] }),
        }));
        let out = Object(Gc::new(Object_ {
            schema: schema.clone(),
            id: id.clone(),
            serial_root: serial_root,
            rust_root: rust_root,
            mut_: GcCell::new(ObjectMut_ {
                escapable_parent: EscapableParent::None,
                rust_extra_roots: vec![],
                serial_extra_roots: vec![],
                has_external_deps: false,
                type_attrs: vec![],
                seen_ids: HashSet::new(),
            }),
        }));
        out.take_id(id);
        out.take_id(serial_id);
        out.take_id(rust_id);
        schema.0.as_ref().borrow_mut().objects.entry(name).or_insert_with(Vec::new).push(out.clone());
        return out;
    }

    /// Add a structure prefix line like `#[...]` to the object definition.  Call like
    /// `o.add_type_attrs(quote!(#[Derive(x,y,z)]))`.
    pub fn add_type_attrs(&self, attrs: TokenStream) {
        self.0.mut_.borrow_mut().type_attrs.push(attrs);
    }

    // # Node generation
    /// Read/write a sequence of bytes with a fixed length from the stream next. See
    /// `Range` for more information on how this can be used.
    pub fn fixed_range(&self, id: impl Into<String>, bytes: usize) -> Range {
        let id = id.into();
        let node_id = self.take_id(id.clone());
        let seg = self.seg(&id);
        let serial = NodeFixedRange(Gc::new(NodeFixedRange_ {
            scope: self.clone(),
            id: node_id,
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: seg.clone(),
            len_bytes: bytes,
            mut_: GcCell::new(NodeFixedRangeMut_ { rust: BTreeMap::new() }),
        }));
        seg.0.mut_.borrow_mut().rust = Some(serial.clone().into());
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(serial.clone().into());
        return Range(Gc::new(GcCell::new(Range_ {
            serial: serial,
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
        let rust = NodeFixedBytes::new(NodeFixedBytesArgs {
            scope: self.clone(),
            id: self.take_id(id),
            start: using.start.bytes,
            len: using.avail.bytes,
        });
        self.lift_connect(&ancestry, &range2.serial, rust.clone().into(), &mut rust.0.mut_.borrow_mut().serial);
        return rust;
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
        let rust = NodeInt::new(NodeIntArgs {
            scope: self.clone(),
            id: self.take_id(id),
            start: using.start,
            len: using.avail,
            signed: signed,
            endian: endian,
        });
        self.lift_connect(&ancestry, &range2.serial, rust.clone().into(), &mut rust.0.mut_.borrow_mut().serial);
        return rust;
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
            id: self.take_id(id),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: seg.clone(),
            shift: shift,
            alignment: alignment,
        }));
        seg.0.mut_.borrow_mut().rust = Some(serial.clone().into());
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(serial.clone().into());
    }

    /// Read/write a sequence of bytes whose length is determined dynamically by a
    /// previous integer.
    pub fn dynamic_bytes(&self, id: impl Into<String>, len: NodeInt) -> NodeDynamicBytes {
        let id = id.into();
        let serial = self.seg(&id);
        let rust = NodeDynamicBytes(Gc::new(NodeDynamicBytes_ {
            scope: self.clone(),
            id: self.take_id(id),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial,
            mut_: GcCell::new(NodeDynamicBytesMut_ {
                serial_len: None,
                rust: None,
            }),
        }));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(rust.clone().into());
        self.lift_connect(
            &self.get_ancestry_to(&len),
            &len,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial_len,
        );
        return rust;
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
        let rust = NodeDelimitedBytes(Gc::new(NodeDelimitedBytes_ {
            scope: self.clone(),
            id: self.take_id(id),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial.clone(),
            delim_len: delimiter.len(),
            delim_bytes: quote!(&[#(#delim_els,) *]),
            mut_: GcCell::new(NodeDelimitedBytesMut_ { rust: None }),
        }));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(rust.clone().into());
        serial.0.mut_.borrow_mut().rust = Some(rust.clone().into());
        return rust;
    }

    /// Read/write until the end of the serial data.
    pub fn remaining_bytes(&self, id: impl Into<String>) -> NodeRemainingBytes {
        let id = id.into();
        let serial = self.seg(&id);
        let rust = NodeRemainingBytes(Gc::new(NodeRemainingBytes_ {
            scope: self.clone(),
            id: self.take_id(id),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial.clone(),
            mut_: GcCell::new(NodeSuffixBytesMut_ { rust: None }),
        }));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(rust.clone().into());
        serial.0.mut_.borrow_mut().rust = Some(rust.clone().into());
        return rust;
    }

    /// Read/write an array of objects, with the length (number of objects) specified
    /// by a previous integer value.
    pub fn dynamic_array(
        &self,
        id: impl Into<String>,
        len: NodeInt,
        obj_name: impl Into<String>,
    ) -> (NodeDynamicArray, Object) {
        let id = id.into();
        let serial = self.seg(&id);
        let obj_name = obj_name.into();
        let element = Object::new(&format!("{}__elem", id), &self.0.schema, obj_name);
        let rust = NodeDynamicArray(Gc::new(NodeDynamicArray_ {
            scope: self.clone(),
            id: self.take_id(id),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial,
            element: element.clone(),
            mut_: GcCell::new(NodeDynamicArrayMut_ {
                serial_len: None,
                rust: None,
            }),
        }));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(rust.clone().into());
        self.lift_connect(
            &self.get_ancestry_to(&len),
            &len,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial_len,
        );
        return (rust, element);
    }

    /// Read/write an enumeration, where the parsing variation is determined by the
    /// previously defined tag value.  The tag can be any `match`-able data type (int,
    /// bool, float, string, byte array).
    pub fn enum_(
        &self,
        id: impl Into<String>,
        tag: impl Into<Node>,
        enum_name: impl Into<String>,
    ) -> (NodeEnum, Enum) {
        let id = id.into();
        let tag = tag.into();
        let serial = self.seg(&id);
        let enum_name = enum_name.into();
        let rust = NodeEnum(Gc::new(NodeEnum_ {
            scope: self.clone(),
            id: self.take_id(id),
            type_name: enum_name.clone(),
            serial_before: self.0.serial_root.0.mut_.borrow().sub_segments.last().cloned(),
            serial: serial,
            mut_: GcCell::new(crate::node_enum::NodeEnumMut_ {
                serial_tag: None,
                variants: vec![],
                default_variant: None,
                rust: None,
                external_deps: BTreeMap::new(),
                type_attrs: vec![],
            }),
        }));
        self.0.serial_root.0.mut_.borrow_mut().sub_segments.push(rust.clone().into());
        self.0.schema.0.as_ref().borrow_mut().enums.entry(enum_name).or_insert_with(Vec::new).push(rust.clone());
        self.lift_connect(
            &self.get_ancestry_to(&tag),
            &tag,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial_tag,
        );
        return (rust.clone(), Enum {
            schema: self.0.schema.clone(),
            obj: self.clone(),
            enum_: rust,
        });
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
        let rust = NodeCustom(Gc::new(NodeCustom_ {
            scope: self.clone(),
            id: self.take_id(id),
            rust_type: rust_type,
            read_code: Box::new(read_code),
            write_code: Box::new(write_code),
            mut_: GcCell::new(NodeCustomMut_ {
                serial: vec![],
                rust: None,
            }),
        }));
        for arg in serial {
            rust.0.mut_.borrow_mut().serial.push(None);
            self.lift_connect(
                &self.get_ancestry_to(&arg),
                &arg,
                rust.clone().into(),
                rust.0.mut_.borrow_mut().serial.last_mut().unwrap(),
            );
        }
        return rust;
    }

    /// Turn an integer into a boolean value.  0 is false, all other values are true.
    /// When writing, 1 will be written for true values.
    pub fn bool(&self, id: impl Into<String>, serial: NodeInt) -> NodeCustom {
        let rust_type = serial.0.rust_type.clone();
        return self.custom(
            //. .
            id,
            quote!(bool),
            |s, d| quote!{
                let #d = #(#s) *!= 0;
            },
            move |s, d| quote!{
                let #(#d) *: #rust_type 
                //. .
                = match * #s {
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
                    let #d = #rust_type:: #read_method(#(#s) *);
                }
            },
            move |s, d| quote!{
                let #(#d) *
                //. .
                =& #s.#write_method();
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
                let #d = String:: from_utf8(#(#s) *).errorize(#err) ?;
            },
            move |s, d| quote!{
                let #(#d) *
                //. .
                = #s.as_bytes();
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
    pub fn rust_const(&self, id: impl Into<String>, serial: impl Into<Node>, value: TokenStream) {
        let id = id.into();
        let serial: Node = serial.into();
        let rust = NodeConst(Gc::new(NodeConst_ {
            id: self.take_id(id),
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

    /// Store the read `serial` value in the object with the given `name`.  The type
    /// will match whatever type `serial` is.
    pub fn rust_field(&self, name: impl Into<String>, serial: impl Into<Node>) {
        let name = name.into();
        let serial = serial.into();
        let rust = NodeRustField(Gc::new(NodeRustField_ {
            scope: self.clone(),
            id: self.take_id(name.clone()),
            field_name: name,
            obj: self.0.rust_root.clone(),
            mut_: GcCell::new(crate::node_rust::NodeRustFieldMut_ { serial: None }),
        }));
        self.0.rust_root.0.mut_.borrow_mut().fields.push(rust.clone());
        self.lift_connect(
            &self.get_ancestry_to(&serial),
            &serial,
            rust.clone().into(),
            &mut rust.0.mut_.borrow_mut().serial,
        );
    }

    // # Internal
    fn take_id(&self, id: String) -> String {
        let mut at = self.clone();
        loop {
            if at.0.mut_.borrow().seen_ids.contains(&id) {
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
        self.0.mut_.borrow_mut().seen_ids.insert(id.clone());
        return id;
    }

    fn id(&self) -> String {
        return self.0.id.clone();
    }

    fn seg(&self, id: &str) -> NodeSerialSegment {
        let out = NodeSerialSegment(Gc::new(NodeSerialSegment_ {
            scope: self.clone(),
            id: self.take_id(format!("{}__serial_seg", id)),
            serial_root: self.0.serial_root.clone().into(),
            serial_before: self.0.serial_root.0.mut_.borrow().segments.last().cloned(),
            mut_: GcCell::new(NodeSerialSegmentMut_ { rust: None }),
        }));
        self.0.serial_root.0.mut_.borrow_mut().segments.push(out.clone());
        return out;
    }

    fn get_ancestry_to(&self, serial: &dyn NodeMethods) -> Vec<SomeEscapableParent> {
        let mut ancestry = vec![];
        let mut at = self.clone();
        loop {
            if at.id() == serial.scope().id() {
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

    fn lift_connect<
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

impl Enum {
    /// See `add_type_attrs` in `Object`.
    pub fn add_type_attrs(&self, attrs: TokenStream) {
        self.enum_.0.mut_.borrow_mut().type_attrs.push(attrs);
    }

    /// Define a new variant in the enum.  `tag` is a literal that will be used in the
    /// match case for the tag value the enum reads.
    pub fn variant(
        &self,
        id: impl Into<String>,
        variant_name: impl Into<String>,
        obj_name: impl Into<String>,
        tag: TokenStream,
    ) -> Object {
        let id = id.into();
        let variant_name = variant_name.into();
        let element = Object::new(id, &self.schema, obj_name.into());
        self.enum_.0.mut_.borrow_mut().variants.push(EnumVariant {
            var_name: variant_name.clone(),
            tag: tag,
            element: element.clone(),
        });
        element.0.mut_.borrow_mut().escapable_parent = EscapableParent::Enum(EscapableParentEnum {
            enum_: self.enum_.clone(),
            variant_name: variant_name,
            parent: self.obj.clone(),
        });
        return element;
    }

    /// Define the default variant.  This returns the `Object` for defining the variant
    /// type, as well as a `Node` that represents the unmatched tag value which can be
    /// used by nodes within the variant type.  If you only need to read the value,
    /// this can be ignored.  If you need to round-trip, the value needs to be consumed
    /// so it can be output again upon serialization.
    pub fn default(
        &self,
        id: impl Into<String>,
        variant_name: impl Into<String>,
        obj_name: impl Into<String>,
    ) -> (Object, Node) {
        let id = id.into();
        let variant_name = variant_name.into();
        let element = Object::new(id.clone(), &self.schema, obj_name.into());
        let dummy: Node = NodeEnumDummy(Gc::new(NodeEnumDummy_ {
            scope: element.clone(),
            id: format!("{}__tag", id),
            rust_type: self.enum_.0.mut_.borrow().serial_tag.as_ref().unwrap().primary.rust_type(),
            mut_: GcCell::new(NodeEnumDummyMut_ { rust: None }),
        })).into();
        let old = self.enum_.0.mut_.borrow_mut().default_variant.replace(EnumDefaultVariant {
            var_name: variant_name.clone(),
            tag: dummy.clone(),
            element: element.clone(),
        });
        if old.is_some() {
            panic!("Default variant already set.");
        }
        {
            let mut el_mut = element.0.mut_.borrow_mut();
            el_mut.serial_extra_roots.push(dummy.clone());
            el_mut.escapable_parent = EscapableParent::Enum(EscapableParentEnum {
                enum_: self.enum_.clone(),
                variant_name: variant_name,
                parent: self.obj.clone(),
            });
        }
        return (element, dummy);
    }
}
