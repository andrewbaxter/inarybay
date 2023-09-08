This is a graph-based binary format serializer/deserializer generator for use in `build.rs`!

This is for interfacing with existing, externally developed binary format specifications. If you want to automatically serialize/deserialize your own data, use Protobuf or Bincode or something similar instead of this.

By avoiding a struct macro based interface I think this is more flexible and simpler to use than alternatives.

# Goals

Goals

1. Flexibility, supporting all the craziness people do with binary format specs

   I'm not sure if it's possible to bound all things formats can do. This won't support everything from the get-go, but it's supposed to be structured in such a way that individual new features don't require major rewrites.

2. Easy to understand, composable elements

   The API is simple and flexible by constructing a graph of reversible processing nodes.

3. Precision

   Specifications are unambiguous, so binary representations with the same spec never change in future versions due to underspecification.

4. Safety
   - Type, overlap, bounds checking
   - Rust to Rust round-tripping is lossless

Non-goals

- Convenient binary format development

  If you want convenience just use ProtoBuf or Bincode or some other general purpose, battle-tested generator which handles low level details for you.

  This is primarily for interfacing with existing, externally developed specifications.

- Extreme optimization

  This is rust, and I don't do crazy things, so it won't be slow. But optimization is lower priority than the above.

# Features

Current features

- Basic schema - primitive types, integers, arrays, enums
- Serial bit fields
- Alignment
- Out of order/split deserialization
- Custom types (serde, exotic string encodings)
- Sync and async
- ✨Macro and generic free✨

Want features

- Reading from/writing to the end of file (reverse direction)
- Rust bitfields
- Fixed-length arrays
- Delimited arrays
- Unwrap single-field objects

Not-in-the-short-run features

- Zero-alloc reading/writing
- Global byte alignment (alignment is relative to the current object, but alignments in an unaligned object will be unaligned)

# Example

A minimal versioned data container.

**build.rs**:

```rust
use std::{
   path::PathBuf,
   env,
   str::FromStr,
   fs::{
      self,
   },
};
use inarybay::scope::Endian;
use quote::quote;

pub fn main() {
   println!("cargo:rerun-if-changed=build.rs");
   let root = PathBuf::from_str(&env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
   let schema = inarybay::schema::Schema::new();
   {
      let scope = schema.scope("scope", inarybay::schema::GenerateConfig {
            read: true,
            write: true,
            ..Default::default()
      });
      let version = scope.int("version_int", scope.fixed_range("version_bytes", 2), Endian::Big, false);
      let body = scope.remaining_bytes("data_bytes");
      let object = scope.object("obj", "Versioned");
      {
            object.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
            object.field("version", version);
            object.field("body", body);
      }
      scope.rust_root(object);
   }
   fs::write(root.join("src/versioned.rs"), schema.generate().as_bytes()).unwrap();
}
```

Then use it like:

**main.rs**

```rust
use std::fs::File;
use crate::versioned::versioned_read;

pub fn main() {
   let mut f = File::open("x.jpg.ver").unwrap();
   let v = versioned_read(&mut f).unwrap();
   println!("x.jpg.ver is {}", v.version);
   println!("x.jpg.ver length is {}", v.body.len());
}
```

# Guide

## Terminology: The serial-rust axis

The words "serial" and "rust" are scattered here and there.

Deserialization takes data from the "serial" side and moves towards the "rust" side, ending at the "rust root" which is the object to deserialize.

Serialization takes "rust" side data (the object) and transforms it through multiple nodes until it reaches the "serial root" - the file, or socket, or whatever.

Each link between nodes has a "serial" and a "rust" side.

## Setup

To use Inarybay you define a schema in `build.rs`, which generates normal rust code.

You need two dependencies:

```
cargo add --build inarybay
cargo add inarybay-runtime
```

The latter has a few helper types and republished dependencies.

## Defining a schema

Some of the descriptions here are deserialization-oriented, but everything is naturally bidirectional, it's just easier to describe with a reference direction.

1. Create a schema `let schema = Schema::new()`
2. Create a root scope `let scope = schema.scope(...)`
3. Define nodes on `scope` like `scope.fixed_range`, `scope.int`, etc., connecting them as you go
4. Define rust root using `scope.rust_root`
5. Generate the code with `schema.generate` and write it to the file you want

## A note on arguments

- **`Node`** - all `Node*` types have an `.into()` which will convert them to `Node`. `Node` is basically a big enum of all the node types.
- **`id`** - these are used for variable names in the generated de/serialize code, as well as uniquely identifying nodes for error messages and loop-identification during graph traversal
- **`TokenStream`** - if an argument has this type, it means it wants some code that will be injected into the generated code. You can generate it with `quote!()` or `quote!{}` (equivalent, use whichever bracket you prefer) from the [quote](https://github.com/dtolnay/quote) crate. The code could be something as simple as a type (like `quote!(my::special::Type)`), an expression (`quote!(#source * 33)`), or multiple statements, depending on what the function requires.

## Troubleshooting

- **Error line numbers**

  The build script uses panics to communicate build errors. By default in `build.rs` these are missing line numbers - to get line numbers do

  ```
  CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=true RUST_BACKTRACE=1 cargo build
  ```

- **Mismatched types**

  When specifying constants using `quote!` using a variable, like `quote!(#i)`, `quote!` appends the literal type suffix. You may need to cast to the right number type before passing it to `quote!` to match the data type of whatever data it's being used with.

  It may be possible to make this more type safe in a future update.

# Design and implementation

De/serialization is done as a dependency graph traversal - all dependencies are walked before any dependents.

For serialization, the root of the graph is the file/socket/whatever - it depends on each serial segment, which depends on individual data serializations from fields. Later segments depend on earlier segments, so the earlier segments get written first.

For deserialization, the root of the graph is the root rust type, like a rust struct - the root depends on each field to be read, which depends on data transformations, which depend on segments being read. Again, like for serialization, each serial segment depends on the previous segment so that segment is read before the next.

## Nesting

Nested objects, like arrays and enums, are treated as single nodes with their own graphs internally. I feel like it should be possible to unify this in a single graph but I haven't come up with a way to do it yet.

## Writing and memory use

Right now, each node allocates memory for its output when writing. I would like to write directly to the output stream without allocating extra buffers, but at the moment that would make the following scenario difficult. Consider the graph:

- Serial enum tag
- Serial `X`
- Serial `Y`
- Serial enum
  - Variant
    - Serial integer `Z`
    - Custom node, uses `Z` and `X`

The custom node does some complex processing during transformation.

During serialization, the variant node produces three outputs: the enum tag, the enum, and `X`. `Y` needs to be written between `X` and the enum, so there are two options:

1. Serialize to variables, then write the variables in order
2. Descend into the enum multiple times

2 would allow direct writing to the stream with no temporary buffers, but both `X` and `Z` depend on the custom node, so each descent would need to redo the custom node's transformations, or else somehow per-variant temporary values would need to be stored between descents into the enum.

It may be possible to do that, but it would be significantly more complicated and I don't think the memory use is excessive at the moment.
