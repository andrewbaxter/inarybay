use std::{
    io,
    collections::{
        HashSet,
        HashMap,
    },
};
use proc_macro2::{
    Ident,
    TokenStream,
};
use quote::{
    quote,
    format_ident,
};

// Unsize + CoerceUnsize are unstable so no smart pointer libraries can use.  Give
// up and just never garbage collect for now.
type S<X> = &'static X;

#[derive(Clone, Copy)]
struct Pos {
    bytes: usize,
    bits: usize,
}

enum Endianness {
    Big,
    Little,
}

struct IntArgs {
    signed: bool,
    endian: Endianness,
    bits: usize,
}

fn obj_ident() -> Ident {
    return format_ident!("rust");
}

fn write_buf_ident() -> Ident {
    return format_ident!("write_buf");
}

fn reader_ident() -> Ident {
    return format_ident!("reader");
}

fn read_int(serial: &Ident, pos: Pos, int_args: &IntArgs) -> TokenStream {
    if int_args.bits <= 8 {
        if pos.bits + int_args.bits > 8 {
            panic!();
        }
        let serial_start = pos.bytes;
        let serial_offset = pos.bits;
        let mut serial_mask = 0u8;
        for _ in 0 .. int_args.bits {
            serial_mask = serial_mask * 2 + 1;
        }
        let rust_type;
        if int_args.signed {
            rust_type = format_ident!("i8");
        } else {
            rust_type = format_ident!("u8");
        }
        return quote!(#rust_type:: from_ne_bytes([((#serial[#serial_start] >> #serial_offset) & #serial_mask)]));
    } else {
        if pos.bits != 0 {
            panic!();
        }
        if int_args.bits % 8 != 0 {
            panic!();
        }
        let serial_start = pos.bytes;
        let serial_bytes = int_args.bits / 8;
        let mut rust_bits = int_args.bits.next_power_of_two();
        if rust_bits > 64 {
            panic!();
        }
        if rust_bits < 8 {
            rust_bits = 8;
        }
        let rust_type;
        if int_args.signed {
            rust_type = format_ident!("s{}", rust_bits);
        } else {
            rust_type = format_ident!("u{}", rust_bits);
        }
        let method;
        match int_args.endian {
            Endianness::Big => method = format_ident!("from_be_bytes"),
            Endianness::Little => method = format_ident!("from_le_bytes"),
        };
        let mut out = quote!(#serial[#serial_start..#serial_start + #serial_bytes]);
        if int_args.bits != rust_bits {
            let rust_bytes = rust_bits / 8;
            match int_args.endian {
                Endianness::Big => {
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
                Endianness::Little => {
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
        return quote!(#rust_type:: #method(#out));
    }
}

fn write_int(rust: &Ident, serial: &Ident, pos: Pos, int_args: &IntArgs) -> TokenStream {
    if int_args.bits <= 8 {
        if pos.bits + int_args.bits > 8 {
            panic!();
        }
        let serial_start = pos.bytes;
        let serial_offset = pos.bits;
        let mut serial_mask = 0u8;
        for _ in 0 .. int_args.bits {
            serial_mask = serial_mask * 2 + 1;
        }
        let rust_type;
        if int_args.signed {
            rust_type = format_ident!("i8");
        } else {
            rust_type = format_ident!("u8");
        }
        return quote!{
            #serial[#serial_start] |=(#rust_type:: from_ne_bytes([#rust]) & #serial_mask) << #serial_offset;
        };
    } else {
        if pos.bits != 0 {
            panic!();
        }
        if int_args.bits % 8 != 0 {
            panic!();
        }
        let serial_start = pos.bytes;
        let serial_bytes = int_args.bits / 8;
        let mut rust_bits = int_args.bits.next_power_of_two();
        if rust_bits > 64 {
            panic!();
        }
        if rust_bits < 8 {
            rust_bits = 8;
        }
        let method;
        match int_args.endian {
            Endianness::Big => method = format_ident!("to_be_bytes"),
            Endianness::Little => method = format_ident!("to_le_bytes"),
        };
        let mut out = quote!(#rust.#method());
        if int_args.bits != rust_bits {
            let rust_bytes = rust_bits / 8;
            match int_args.endian {
                Endianness::Big => {
                    let endian_pad_offset = rust_bytes - serial_bytes;
                    out = quote!(#out[#endian_pad_offset..#rust_bytes]);
                },
                Endianness::Little => {
                    out = quote!(#out[0..#serial_bytes]);
                },
            }
        }
        return quote!{
            #serial[#serial_start..#serial_start + #serial_bytes].copy_from_slice(& #out);
        };
    }
}

trait Node {
    fn read_deps(&self) -> Vec<S<dyn Node>>;
    fn gen_read(&self, code: &mut Vec<TokenStream>);
    fn write_deps(&self) -> Vec<S<dyn Node>>;
    fn gen_write(&self, code: &mut Vec<TokenStream>);

    // Only for values... put in this trait since trait->trait upcasting doesn't work
    // yet...
    fn value_get_ident(&self) -> Ident;
}

struct File_ {
    buf_ident: Ident,
    chunks: Vec<Chunk>,
}

impl Node for File_ {
    fn read_deps(&self) -> Vec<S<dyn Node>> {
        // Used as seed, this should never be called
        panic!();
    }

    fn gen_read(&self, _code: &mut Vec<TokenStream>) { }

    fn write_deps(&self) -> Vec<S<dyn Node>> {
        return self.chunks.iter().map(|c| c.0 as S<dyn Node>).collect();
    }

    fn gen_write(&self, _code: &mut Vec<TokenStream>) { }

    fn value_get_ident(&self) -> Ident {
        panic!();
    }
}

struct File(S<File_>);

struct Chunk_ {
    file: File,
    before: Option<Chunk>,
    after: Option<Chunk>,
    ident: Ident,
    bytes: usize,
    bits: usize,
}

impl Node for Chunk_ {
    fn read_deps(&self) -> Vec<S<dyn Node>> {
        let mut out = vec![self.file.0 as S<dyn Node>];
        if let Some(x) = &self.before {
            out.push(x.0);
        }
        return out;
    }

    fn gen_read(&self, code: &mut Vec<TokenStream>) {
        let buf_ident = &self.ident;
        let bytes = self.bytes;
        let reader_ident = reader_ident();
        code.push(quote!{
            let mut #buf_ident = std:: vec:: Vec:: new();
            #buf_ident.resize(#bytes);
            #reader_ident.read_exact(& mut #buf_ident);
        })
    }

    fn write_deps(&self) -> Vec<S<dyn Node>> {
        let mut out = vec![];
        if let Some(x) = &self.before {
            out.push(x.0 as S<dyn Node>);
        }
        return out;
    }

    fn gen_write(&self, code: &mut Vec<TokenStream>) {
        let buf_ident = &self.file.0.buf_ident;
        let bytes = self.bytes;
        code.push(quote!{
            #buf_ident.resize(#buf_ident.len() + #bytes);
        });
    }

    fn value_get_ident(&self) -> Ident {
        panic!();
    }
}

struct Chunk(S<Chunk_>);

struct Integer_ {
    chunk: Chunk,
    chunk_pos: Pos,
    rust: Option<S<dyn Node>>,
    ident: Ident,
    args: IntArgs,
}

impl Node for Integer_ {
    fn read_deps(&self) -> Vec<S<dyn Node>> {
        return vec![self.chunk.0];
    }

    fn gen_read(&self, code: &mut Vec<TokenStream>) {
        let ident = &self.ident;
        let value = read_int(&self.chunk.0.ident, self.chunk_pos, &self.args);
        code.push(quote!{
            let #ident = #value;
        });
    }

    fn write_deps(&self) -> Vec<S<dyn Node>> {
        return vec![self.rust.0];
    }

    fn gen_write(&self, code: &mut Vec<TokenStream>) {
        code.push(write_int(&self.ident, &self.chunk.0.ident, self.chunk_pos, &self.args));
    }

    fn value_get_ident(&self) -> Ident {
        return self.ident.clone();
    }
}

struct Integer(S<Integer_>);

struct Field {
    ident: Ident,
    value: S<dyn Node>,
}

struct Obj_ {
    fields: Vec<Field>,
    type_ident: Ident,
}

impl Node for Obj_ {
    fn read_deps(&self) -> Vec<S<dyn Node>> {
        todo!()
    }

    fn gen_read(&self, code: &mut Vec<TokenStream>) {
        let type_ident = &self.type_ident;
        let mut field = vec![];
        for f in &self.fields {
            let field_ident = &f.ident;
            let value_ident = f.value.value_get_ident();
            field.push(quote!(#field_ident: #value_ident,));
        }
        code.push(quote!(#type_ident {
            #(#field) *
        }));
    }

    fn write_deps(&self) -> Vec<S<dyn Node>> {
        return self.fields.iter().map(|e| e.value as S<dyn Node>).collect();
    }

    fn gen_write(&self, code: &mut Vec<TokenStream>) {
        for f in &self.fields {
            let field = &f.ident;
            let ident = f.value.value_get_ident();
            let obj_ident = format!("rust");
            code.push(quote!{
                let #ident = #obj_ident.#field;
            });
        }
    }

    fn value_get_ident(&self) -> Ident {
        panic!();
    }
}

struct Obj(S<Obj_>);

//. struct X {
//.     x: i32,
//.     y: i64,
//.     z: bool,
//.     a: String,
//.     b: i32,
//. }
//. 
//. fn x(r: impl io::Read) -> Result<X, String> {
//.     let mut b0 = [0u8; 123];
//.     r.read_exact(&mut b0)?;
//.     let x = i32::from_be_bytes(b0[0 .. 4].try_into().unwrap());
//.     let y = i64::from_be_bytes(b0[4 .. 12].try_into().unwrap());
//.     let z = b0[12] != 0u8;
//.     let l0 = u16::from_be_bytes(b0[13 .. 15].try_into().unwrap());
//.     let mut b1 = Vec::new();
//.     b1.resize(l0 as usize, 0u8);
//.     r.read_exact(&mut b1)?;
//.     let a = String::from_utf8(b1)?;
//.     let mut b2 = Vec::new();
//.     b2.resize(24, 0u8);
//.     r.read_exact(&mut b2)?;
//.     let b = i32::from_be_bytes(b2[0 .. 4].try_into().unwrap());
//.     return Ok(X {
//.         x: x,
//.         y: y,
//.         z: z,
//.         a: a,
//.         b: b,
//.     });
//. }
//. 
//. fn x2(w: impl io::Write, x: X) {
//.     let mut b0 = Vec::new();
//.     b0.resize(b0.len() + 123, 0u8);
//.     b0[0 .. 4].copy_from_slice(&x.x.to_be_bytes());
//.     b0[4 .. 12].copy_from_slice(&x.y.to_be_bytes());
//.     b0[12] = if x.z {
//.         1u8
//.     } else {
//.         0u8
//.     };
//.     b0[13 .. 15].copy_from_slice(&(x.a.len() as u16).to_be_bytes());
//.     b0.resize(b0.len() + x.a.len(), 0u8);
//.     b0[123 .. x.a.len()].copy_from_slice(x.a.as_bytes());
//.     let at = 123 + x.a.len();
//.     b0.resize(b0.len() + 24, 0u8);
//.     b0[at .. at + 4].copy_from_slice(&x.b.to_be_bytes());
//. }
//. 
struct Context {
    serial_root: File,
    rust_roots: Vec<S<dyn Node>>,
    last_chunk: Option<Chunk>,
}

struct RangeBuilder<'a> {
    context: &'a mut Context,
    chunk: Chunk,
    pos: Pos,
    bytes: usize,
    bits: usize,
}

impl<'a> RangeBuilder<'a> {
    fn int(&self, signed: bool, endianness: Endianness) -> Integer {
        // TODO branching
        let i = Integer(Box::leak(Box::new(Integer_ {
            chunk: self.chunk,
            chunk_pos: self.pos,
            rust: None,
            ident: todo,
            args: IntArgs {
                signed: signed,
                endian: endianness,
                bits: self.bytes * 8 + self.bits,
            },
        })));
        self.chunk.0.fields.push(i);
        return i;
    }
}

impl Context {
    fn range(&mut self, bytes: usize, bits: usize) -> RangeBuilder {
        let chunk;
        match &mut self.last_chunk {
            Some(c) => {
                chunk = c;
            },
            None => {
                let mut c = Chunk(Box::leak(Box::new(Chunk_ {
                    file: self.serial_root,
                    after: None,
                    before: None,
                    ident: todo,
                    bytes: 0,
                    bits: 0,
                })));
                self.last_chunk = Some(c);
                self.serial_root.0.chunks.push(c);
                chunk = c;
            },
        };
        let bits = chunk.0.bits + bits;
        chunk.0.bytes += bits / 8;
        chunk.0.bits = bits % 8;
        return RangeBuilder {
            context: self,
            bytes: bytes,
            bits: bits,
        };
    }

    fn generate_type(&self) -> TokenStream { }

    fn generate_write(&self) -> TokenStream {
        let mut seen_count = HashMap::new();
        let mut stack = vec![(f.0 as S<dyn Node>, true)];
        let mut code = vec![];
        while let Some((node, first_visit)) = stack.pop() {
            if first_visit {
                stack.push((node, true));
                for dep in node.write_deps() {
                    stack.push((dep, false));
                }
            } else if *seen_count
                .entry(node as *const dyn Node)
                .or_insert_with(|| node.read_deps().len().saturating_sub(1)) ==
                0 {
                node.gen_write(&mut code);
            }
        }
        return quote!(#(#code) *);
    }

    fn generate_read(&self) -> TokenStream {
        let mut seen_count = HashMap::new();
        let mut stack = vec![(o.0 as S<dyn Node>, true)];
        let mut code = vec![];
        while let Some((node, first_visit)) = stack.pop() {
            if first_visit {
                stack.push((node, true));
                for dep in node.read_deps() {
                    stack.push((dep, false));
                }
            } else if *seen_count
                .entry(node as *const dyn Node)
                .or_insert_with(|| node.write_deps().len().saturating_sub(1)) ==
                0 {
                node.gen_read(&mut code);
            }
        }
        return quote!(#(#code) *);
    }
}

#[test]
fn x() {
    println!("{}", genemichaels::format_str(generate_type(o).to_string()).unwrap().rendered);
    println!("{}", genemichaels::format_str(generate_read(o).to_string()).unwrap().rendered);
    println!("{}", genemichaels::format_str(generate_write(f).to_string()).unwrap().rendered);
}
