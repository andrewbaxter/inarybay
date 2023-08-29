This is a graph-based binary format serializer/deserializer generator for use in `build.rs`, for interfacing with existing, externally developed binary format specifications. If you want to automatically serialize/deserialize your own data, use Protobuf or Bincode or something similar instead of this.

Many existing de/serializer code generators use derive macros, but this puts restrictions on what sort of data can be described (it needs to roughly match a rust struct) and the limited workarounds available make those tools harder to learn and work with.

# Goals

Goals

1. Flexibility, supporting all the craziness people do with binary format specs

   I'm not sure if it's possible to bound all things formats can do. This won't support everything from the get-go, but it's supposed to be structured in such a way that individual new features don't require major rewrites.

2. Easy to understand, composable elements

   The API is simple and flexible by chaining reversible processing steps with easily understood nodes.

3. Unambiguity - specifications are unambiguous, so binary representations with the same spec never change in future versions due to underspecification

4. Safety
   - Type, overlap, bounds checking
   - Rust to Rust round-tripping is lossless

Non-goals

- Convenient binary format development

  If you want convenience just use ProtoBuf or Bincode or some other general purpose, battle-tested generator which handles low level details for you.

  This is primarily for interfacing with existing, externally developed specifications.

- Extreme optimization

  This is rust, and I don't do crazy things, so it won't be slow. But I'm not going to compromise on the goals to increase speed.

# Features

Current features

- Basic schema - primitive types, integers, arrays, enums
- Serial bit fields
- Alignment
- Out of order/split deserialization
- Custom types (serde, exotic string encodings)
- Sync and async

Want features

- Reading from/writing to the end of file (reverse direction)
- Rust bitfields
- Fixed-length arrays
- Delimited arrays

Not-in-the-short-run features

- Zero-alloc reading/writing
- Global byte alignment (alignment is relative to the current object, but alignments in an unaligned object will be unaligned)

# Guide

## Terminology: The serial-rust axis

Things like links between nodes have two sides, and for various operations it's important to be able to refer to these separately. Since this is bi-directional, words like "source" "destination" "root" "leaf" etc. are ambiguous.

I use the worlds "serial" and "rust". Deserialization takes data from the "serial" side and moves towards the "rust" side, ending at the "rust root" which is the object to deserialize. Serialization takes "rust" side data (the object) and transforms it through multiple nodes until it reaches the "serial root" - the file, or socket, or whatever. Each link between nodes has a "serial" and a "rust" side.

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

1. Create a schema `s`
2. Create a root object `o` (or more than one)
3. Define serial segment nodes

   These describe the serial-side of the data - which chunks of bytes there are, how long they are, etc. They interact directly with the stream.

   The basic serial-side nodes are:

   - **`o.fixed_range`**, **`o.subrange`** - describes a fixed range of bytes or bits
   - **`o.dynamic_bytes`** - a range of bytes, the length of which is specified by an integer node
   - **`o.remaining_bytes`** - a range of bytes to the end of the file
   - **`o.delimited_bytes`** - a range of bytes terminated by a magic string of bytes (could be just a null byte)
   - **`o.dynamic_array`**, **`o.enum`** - these read a dynamic range based on more complex rules
   - **`o.align`** - this reads and discards data to a specified alignment. I.e. if 3 bytes had been read from the start of the file and the alignment is 4 bytes, this node will read one byte so the next serial segment is aligned. This works with both fixed and dynamic offsets.

4. Define transformation nodes

   These take the serial data and transform it, like converting a run of bytes into a string.

   These nodes are:

   - **`o.int`**, **`o.bool`**, **`o.float`** - take a fixed range of bytes and produce an integer/bool/float value
   - **`o.bytes`** - this is a bit special. Fixed ranges/subranges don't have a well defined rust type. This turns the range into a sized Rust `u8` array.
   - **`o.string_utf8`** - takes a dynamic range of bytes and parse it as a utf8 string
   - **`o.custom`** - inject custom transformation code. As arguments, it takes the rust-side type name, and two functions to output code for consuming the serial-side sources and producing the rust object and vice-versa. The final argument, a list of nodes, are the serial-side nodes that will be the _source_ argument to the deserialization closure and the _destination_ argument to the serialization closure. `bool` and `string_utf8` are built on this.

5. Define rust nodes

   There's two of these

   - **`o.rust_field`** - takes a value and uses it as a field in the output object
   - **`o.rust_const`** - the value isn't available to the caller, but instead confirms that it matches an expected constant/magic value

6. Generate the code with `s.generate` on the schema

   This will produce files, ready to be used by your project.

## A note on arguments

- **`id`** - these are used for variable names in the generated de/serialize code, as well as uniquely identifying nodes for error messages and loop-identification during graph traversal
- **`TokenStream`** - if an argument has this type, it means it wants some code that will be injected into the generated code. You can generate it with `quote!()` or `quote!{}` (equivalent, use whichever bracket you prefer) from the [quote](x) crate. The code could be something as simple as a type (like `quote!(my::special::Type)`), an expression (`quote!(#source * 33)`), or multiple statements, depending on what the function requires.

# Design and implementation

De/serialization is done as a dependency graph traversal - all dependencies are walked before any dependents.

For serialization, the root of the graph is the serial side - it pulls in each segment, which pulls in individual data serializations from fields. Later segments depend on earlier segments, so the earlier segments get written first.

For deserialization, the root of the graph is the rust object - it pulls in each field, which depends on data transformations, which depends on segments being read. Again, like for serialization, each segment depends on the previous segment so that segment is read before the next.

## Nesting

Nested objects, like arrays and enums, are treated as single nodes with their own graphs internally. I feel like it should be possible to unify this in a single graph but I haven't come up with a way to do it yet.

### Enums

For enums alone, already-read data outside the enum (from a higher scope) can be interpreted differently in different variants. In MQTT for instance, the remaining bitfields from the main packet enum tag bytes are used by some variants and are expected to have magic numbers or be all zero in others.

If the external serial node was linked to the node within the nested enum variant grpah 1. the serial field could have multiple dependencies, one per variant and 2. the discriminated field of the variant is only accessible within the context of the `match` which starts and ends during the processing of the enum node itself.

To handle this, links crossing scope boundaries are split. The external node is linked to the containing enum node, and the field in the variant is linked to the serial root of the subgraph. This solves ordering issues and avoids conditional graph links that aren't known at generation time.

## Dynamic elements

There are several dynamic elements, like arrays, dynamic bytes, enums, etc. which have a tag or length segment and a separate body segment. These segments may be separated by other segments, or even occur at a higher scope.

Since these elements are represented by a single graph node, during serialization both segments must be serialized at the same time. However if another element comes in between, then they can't be serialized as a unit. To work around this, these nodes serialize to memory, then the memory chunks are written to the stream on the serial-side (outside of the node), properly interleaved. For things like enums, this means that there's only one `match` block, rather than needing to inspect the enum twice.

An alternative would be to represent these as multiple nodes so the data can be interleaved while writing to the stream directly. For an enum, it means when writing the tag there would be a `match`, and another `match` while writing the body later. The complication with this approach is the handling of cross-scope dependencies (mentioned above) - if each external dep could be ordered independently, each one could require a `match` (and if multiple-levels are traversed multiple nested `match` blocks). These are rare (I think?) though and it would remove the need for a separate "segment" layer of nodes.
