use enum_dispatch::enum_dispatch;
use gc::{
    Trace,
    Finalize,
};
use proc_macro2::TokenStream;
use crate::{
    node_serial::{
        NodeSerial,
        NodeSerialSegment,
    },
    node_fixed_range::NodeFixedRange,
    node_int::NodeInt,
    node_dynamic_bytes::NodeDynamicBytes,
    node_dynamic_array::NodeDynamicArray,
    node_const::NodeConst,
    node_rust::{
        NodeRustField,
        NodeRustObj,
    },
    node_enum::NodeEnum,
    object::{
        Object,
    },
    node_fixed_bytes::NodeFixedBytes,
};

#[enum_dispatch]
pub(crate) trait NodeMethods {
    fn gather_read_deps(&self) -> Vec<Node>;
    fn generate_read(&self) -> TokenStream;
    fn gather_write_deps(&self) -> Vec<Node>;
    fn generate_write(&self) -> TokenStream;
    fn set_rust(&self, rust: Node);
    fn scope(&self) -> Object;
    fn id(&self) -> String;
}

#[macro_export]
macro_rules! derive_forward_node_methods{
    ($x: ty) => {
        impl NodeMethods for $x {
            fn gather_read_deps(&self) -> Vec<Node> {
                return self.0.gather_read_deps();
            }

            fn generate_read(&self) -> TokenStream {
                return self.0.generate_read();
            }

            fn gather_write_deps(&self) -> Vec<Node> {
                return self.0.gather_write_deps();
            }

            fn generate_write(&self) -> TokenStream {
                return self.0.generate_write();
            }

            fn set_rust(&self, rust: Node) {
                return self.0.set_rust(rust);
            }

            fn scope(&self) -> Object {
                return self.0.scope();
            }

            fn id(&self) -> String {
                return self.0.id();
            }
        }
    };
}

#[derive(Clone, Trace, Finalize)]
#[samevariant::samevariant(NodeSameVariant)]
//. #[enum_dispatch(NodeMethods)]
pub(crate) enum Node_ {
    Serial(NodeSerial),
    SerialSegment(NodeSerialSegment),
    FixedRange(NodeFixedRange),
    FixedBytes(NodeFixedBytes),
    Int(NodeInt),
    DynamicBytes(NodeDynamicBytes),
    DynamicArray(NodeDynamicArray),
    Enum(NodeEnum),
    Const(NodeConst),
    RustField(NodeRustField),
    RustObj(NodeRustObj),
}

impl NodeMethods for Node_ {
    #[inline]
    fn gather_read_deps(&self) -> Vec<Node> {
        match self {
            Node_::Serial(inner) => NodeMethods::gather_read_deps(inner),
            Node_::SerialSegment(inner) => NodeMethods::gather_read_deps(inner),
            Node_::FixedRange(inner) => NodeMethods::gather_read_deps(inner),
            Node_::FixedBytes(inner) => NodeMethods::gather_read_deps(inner),
            Node_::Int(inner) => NodeMethods::gather_read_deps(inner),
            Node_::DynamicBytes(inner) => NodeMethods::gather_read_deps(inner),
            Node_::DynamicArray(inner) => NodeMethods::gather_read_deps(inner),
            Node_::Enum(inner) => NodeMethods::gather_read_deps(inner),
            Node_::Const(inner) => NodeMethods::gather_read_deps(inner),
            Node_::RustField(inner) => NodeMethods::gather_read_deps(inner),
            Node_::RustObj(inner) => NodeMethods::gather_read_deps(inner),
        }
    }

    #[inline]
    fn generate_read(&self) -> TokenStream {
        match self {
            Node_::Serial(inner) => NodeMethods::generate_read(inner),
            Node_::SerialSegment(inner) => NodeMethods::generate_read(inner),
            Node_::FixedRange(inner) => NodeMethods::generate_read(inner),
            Node_::FixedBytes(inner) => NodeMethods::generate_read(inner),
            Node_::Int(inner) => NodeMethods::generate_read(inner),
            Node_::DynamicBytes(inner) => NodeMethods::generate_read(inner),
            Node_::DynamicArray(inner) => NodeMethods::generate_read(inner),
            Node_::Enum(inner) => NodeMethods::generate_read(inner),
            Node_::Const(inner) => NodeMethods::generate_read(inner),
            Node_::RustField(inner) => NodeMethods::generate_read(inner),
            Node_::RustObj(inner) => NodeMethods::generate_read(inner),
        }
    }

    #[inline]
    fn gather_write_deps(&self) -> Vec<Node> {
        match self {
            Node_::Serial(inner) => NodeMethods::gather_write_deps(inner),
            Node_::SerialSegment(inner) => NodeMethods::gather_write_deps(inner),
            Node_::FixedRange(inner) => NodeMethods::gather_write_deps(inner),
            Node_::FixedBytes(inner) => NodeMethods::gather_write_deps(inner),
            Node_::Int(inner) => NodeMethods::gather_write_deps(inner),
            Node_::DynamicBytes(inner) => NodeMethods::gather_write_deps(inner),
            Node_::DynamicArray(inner) => NodeMethods::gather_write_deps(inner),
            Node_::Enum(inner) => NodeMethods::gather_write_deps(inner),
            Node_::Const(inner) => NodeMethods::gather_write_deps(inner),
            Node_::RustField(inner) => NodeMethods::gather_write_deps(inner),
            Node_::RustObj(inner) => NodeMethods::gather_write_deps(inner),
        }
    }

    #[inline]
    fn generate_write(&self) -> TokenStream {
        match self {
            Node_::Serial(inner) => NodeMethods::generate_write(inner),
            Node_::SerialSegment(inner) => NodeMethods::generate_write(inner),
            Node_::FixedRange(inner) => NodeMethods::generate_write(inner),
            Node_::FixedBytes(inner) => NodeMethods::generate_write(inner),
            Node_::Int(inner) => NodeMethods::generate_write(inner),
            Node_::DynamicBytes(inner) => NodeMethods::generate_write(inner),
            Node_::DynamicArray(inner) => NodeMethods::generate_write(inner),
            Node_::Enum(inner) => NodeMethods::generate_write(inner),
            Node_::Const(inner) => NodeMethods::generate_write(inner),
            Node_::RustField(inner) => NodeMethods::generate_write(inner),
            Node_::RustObj(inner) => NodeMethods::generate_write(inner),
        }
    }

    #[inline]
    fn set_rust(&self, __enum_dispatch_arg_0: Node) {
        match self {
            Node_::Serial(inner) => {
                NodeMethods::set_rust(inner, __enum_dispatch_arg_0)
            },
            Node_::SerialSegment(inner) => {
                NodeMethods::set_rust(inner, __enum_dispatch_arg_0)
            },
            Node_::FixedRange(inner) => {
                NodeMethods::set_rust(inner, __enum_dispatch_arg_0)
            },
            Node_::FixedBytes(inner) => {
                NodeMethods::set_rust(inner, __enum_dispatch_arg_0)
            },
            Node_::Int(inner) => NodeMethods::set_rust(inner, __enum_dispatch_arg_0),
            Node_::DynamicBytes(inner) => {
                NodeMethods::set_rust(inner, __enum_dispatch_arg_0)
            },
            Node_::DynamicArray(inner) => {
                NodeMethods::set_rust(inner, __enum_dispatch_arg_0)
            },
            Node_::Enum(inner) => NodeMethods::set_rust(inner, __enum_dispatch_arg_0),
            Node_::Const(inner) => {
                NodeMethods::set_rust(inner, __enum_dispatch_arg_0)
            },
            Node_::RustField(inner) => {
                NodeMethods::set_rust(inner, __enum_dispatch_arg_0)
            },
            Node_::RustObj(inner) => {
                NodeMethods::set_rust(inner, __enum_dispatch_arg_0)
            },
        }
    }

    #[inline]
    fn scope(&self) -> Object {
        match self {
            Node_::Serial(inner) => NodeMethods::scope(inner),
            Node_::SerialSegment(inner) => NodeMethods::scope(inner),
            Node_::FixedRange(inner) => NodeMethods::scope(inner),
            Node_::FixedBytes(inner) => NodeMethods::scope(inner),
            Node_::Int(inner) => NodeMethods::scope(inner),
            Node_::DynamicBytes(inner) => NodeMethods::scope(inner),
            Node_::DynamicArray(inner) => NodeMethods::scope(inner),
            Node_::Enum(inner) => NodeMethods::scope(inner),
            Node_::Const(inner) => NodeMethods::scope(inner),
            Node_::RustField(inner) => NodeMethods::scope(inner),
            Node_::RustObj(inner) => NodeMethods::scope(inner),
        }
    }

    #[inline]
    fn id(&self) -> String {
        match self {
            Node_::Serial(inner) => NodeMethods::id(inner),
            Node_::SerialSegment(inner) => NodeMethods::id(inner),
            Node_::FixedRange(inner) => NodeMethods::id(inner),
            Node_::FixedBytes(inner) => NodeMethods::id(inner),
            Node_::Int(inner) => NodeMethods::id(inner),
            Node_::DynamicBytes(inner) => NodeMethods::id(inner),
            Node_::DynamicArray(inner) => NodeMethods::id(inner),
            Node_::Enum(inner) => NodeMethods::id(inner),
            Node_::Const(inner) => NodeMethods::id(inner),
            Node_::RustField(inner) => NodeMethods::id(inner),
            Node_::RustObj(inner) => NodeMethods::id(inner),
        }
    }
}

impl Node_ {
    pub(crate) fn typestr(&self) -> &'static str {
        match self {
            Node_::Serial(_) => unreachable!(),
            Node_::SerialSegment(_) => unreachable!(),
            Node_::FixedRange(_) => unreachable!(),
            Node_::FixedBytes(_) => "fixed bytes",
            Node_::Int(_) => "int",
            Node_::DynamicBytes(_) => "dynamic bytes",
            Node_::DynamicArray(_) => "array",
            Node_::Enum(_) => "enum",
            Node_::Const(_) => "const",
            Node_::RustField(_) => unreachable!(),
            Node_::RustObj(_) => unreachable!(),
        }
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct Node(pub(crate) Node_);

impl NodeMethods for Node {
    fn gather_read_deps(&self) -> Vec<Node> {
        return self.0.gather_read_deps();
    }

    fn generate_read(&self) -> TokenStream {
        return self.0.generate_read();
    }

    fn gather_write_deps(&self) -> Vec<Node> {
        return self.0.gather_write_deps();
    }

    fn generate_write(&self) -> TokenStream {
        return self.0.generate_write();
    }

    fn set_rust(&self, rust: Node) {
        return self.0.set_rust(rust);
    }

    fn scope(&self) -> Object {
        return self.0.scope();
    }

    fn id(&self) -> String {
        return self.0.id();
    }
}

pub(crate) trait ToDep {
    fn dep(&self) -> Vec<Node>;
}

impl<T: Into<Node> + Clone> ToDep for T {
    fn dep(&self) -> Vec<Node> {
        return vec![self.clone().into()];
    }
}

impl<T: ToDep> ToDep for Option<T> {
    fn dep(&self) -> Vec<Node> {
        return self.iter().flat_map(|x| x.dep()).collect();
    }
}

impl<T: ToDep> ToDep for Vec<T> {
    fn dep(&self) -> Vec<Node> {
        return self.iter().flat_map(|x| x.dep()).collect();
    }
}

#[derive(Clone, Trace, Finalize)]
pub(crate) struct RedirectRef<T: Trace + Finalize, U: Trace + Finalize> {
    // Original ref
    pub(crate) primary: T,
    // Replacement for dep resolution
    pub(crate) redirect: Option<U>,
}

impl<
    T: Clone + Trace + Finalize + Into<Node>,
    U: Clone + Trace + Finalize + Into<Node>,
> ToDep for RedirectRef<T, U> {
    fn dep(&self) -> Vec<Node> {
        return vec![self.redirect.clone().map(|x| x.into()).unwrap_or_else(|| self.primary.clone().into())];
    }
}

impl<T: Trace + Finalize, U: Trace + Finalize> RedirectRef<T, U> {
    pub(crate) fn new(v: T) -> Self {
        Self {
            primary: v,
            redirect: None,
        }
    }
}
