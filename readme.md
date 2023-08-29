This is a graph-based binary format serializer/deserializer generator for use in `build.rs`, for interfacing with existing, externally developed binary format specifications. If you want to automatically serialize/deserialize your own data, use Protobuf or Bincode or something similar instead of this.

Many existing de/serializer code generators use derive macros, but this puts restrictions on what sort of data can be described (it needs to roughly match a rust struct) and the limited workarounds available make those tools harder to learn and work with.

# Goals

Goals

1. Flexibility, supporting all the craziness people do with binary format specs

   The API is simple and flexible by chaining reversible processing steps.

   I'm not sure if it's possible to bound all things formats can do. This won't support everything from the get-go, but it's supposed to be structured in such a way that individual new features don't require major rewrites.

2. Unambiguity - specifications are unambiguous, so binary representations with the same spec never change in future versions due to underspecification

3. Safety
   - Type, overlap, bounds checking
   - Rust to Rust round-tripping is lossless

Non-goals

- Convenient binary format development

  If you want convenience just use ProtoBuf or Bincode or some other general purpose, battle-tested generator which handles low level details for you.

  This is primarily for interfacing with existing, externally developed specifications.

# Features

Current features

- Basic schema - primitive types, integers, arrays, enums
- Serial bit fields
- Alignment
- Out of order/split deserialization
- Custom types (serde, exotic string encodings)
- Sync and async

To-do features

- More string encodings
- Reading from/writing to the end of file (reverse direction)
- Rust bitfields
- Fixed-length arrays
- Delimited arrays

Not-in-the-short-run features

- Zero-alloc reading/writing
- Global byte alignment (alignment is relative to the current object, but alignments in an unaligned object will be unaligned)

# How to use it
