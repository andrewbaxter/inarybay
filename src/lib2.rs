use std::rc::Rc;
use gc::Gc;
use proc_macro2::TokenStream;
use quote::{
    quote,
    format_ident,
};

trait Bintery {
    fn render_read(&self, ctx: &mut Ctx) -> TokenStream;
    fn render_write(&self, ctx: &mut Ctx, dest: TokenStream, source: TokenStream) -> TokenStream;
}

pub struct RustField {
    pub parent: Opt<Rc<Object>>,
    pub name: &'static str,
}

pub enum Rust {
    Const(&'static [u8]),
    Field(Rc<RustField>),
}

pub struct NumType {
    pub alignment_bits: usize,
    pub le: bool,
    pub bits: usize,
    pub signed: bool,
}

pub struct NumVal {
    type_: NumType,
    rust: Rust,
}

pub struct VectVal {
    count: Rc<NumVal>,
    el_type: Type,
    rust: Rust,
}

pub struct Field {
    rust: RustField,
    type_: Type,
}

pub struct ObjType {
    fields: Vec<Field>,
}

impl Bintery for ObjType {
    fn render_read(&self, ctx: &mut Ctx) -> TokenStream {
        let mut out = vec![];
        let outvar = ctx.newvar();
        out.push(quote!{
            let mut $outvar: $rusttype = core:: mem:: uninitialized();
        });
        for f in self.fields {
            let f_body = f.type_.render_read(ctx);
            let f_name = f.rust.name;
            out.push(quote!{
                $outvar.$f_name = $f_body;
            });
        }
        out.push(ctx.read_flush());
        out.push(quote!{
            $outvar
        });
        return quote!{
            $($out) *
        };
    }

    fn render_write(&self, ctx: &mut Ctx, dest: TokenStream, source: TokenStream) -> TokenStream {
        let mut out = vec![];
        for f in self.fields {
            let body = f.type_.render_write(dest, quote!($source.$f_name));
            out.push(quote!{
                $body
            });
        }
        out.push(ctx.write_flush());
        return quote!{
            $($out) *
        };
    }
}

impl Bintery for NumVal {
    fn render_read(&self, ctx: &mut Ctx) -> TokenStream { }

    fn render_write(&self, ctx: &mut Ctx, dest: TokenStream, source: TokenStream) -> TokenStream {
        let val = self.rust.render_get(ctx, source);
        let mut out = vec![];
        if self.alignment_bits != 0 {
            let alignment_bits = self.alignment_bits;
            out.push(quote!{
                ctx.align($alignment_bits);
            });
        }
        if self.le {
            out.push(quote!{
                $dest.write_le($val);
            });
        } else {
            out.push(quote!{
                $dest.write_be($val);
            });
        }
        return quote!{
            $($out) *
        };
    }
}

struct Ctx {
    unique: usize,
}

impl Ctx {
    fn newvar(&mut self) -> TokenStream {
        let c = self.unique;
        self.unique += 1;
        return format_ident!("a{}", c);
    }
}

impl Bintery for VectVal {
    fn render_read(&self, ctx: &mut Ctx) {
        let loopvar = ctx.newvar();
        let outvar = ctx.newvar();
        let count = self.count.get_outvar(ctx);
        let body = self.body.render_write(ctx);
        return quote!{
            {
                let mut $outvar = vec ![];
                for $loopvar in 0..$count {
                    $outvar.push($body);
                }
                $outvar
            }
        };
    }

    fn render_write(&self, ctx: &mut Ctx, dest: TokenStream, source: TokenStream) -> TokenStream {
        let loopvar = ctx.newvar();
        let body = self.body.render_read(ctx, dest, loopvar);
        return quote!{
            for $loopvar in $rust {
                $body;
            }
        };
    }
}

fn test_mqtt() {
    let mut client_req = Object::new();
    let mut client_req_bits = client_req.jump(bits(4));
    let req_type_tag = NumType {
        alignment_bits: 0,
        le: false,
        bits: 4,
        signed: false,
    };
    let mut req_type_rust = RustObject::new();
    let mut req_type = TaggedEnumType { tag: req_type_tag, rust: req_type_rust };
    let req_type_connect = req_type.variant(req_type_tag.value(1));
    client_req_bits.add()
    client_req.add(req_type);
}
