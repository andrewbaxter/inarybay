use std::{
    path::PathBuf,
    env,
    fs,
    str::FromStr,
};
use inarybay::{
    schema::Schema,
    node_int::Endian,
};
use quote::quote;

pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let root = PathBuf::from_str(&env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    let src = root.join("src");
    let write = |name: &str, s: Schema| {
        let out_path = src.join(format!("gen_{}.rs", name));
        let ts = s.generate(true, true).to_string();
        match genemichaels::format_str(&ts, &genemichaels::FormatConfig::default()) {
            Err(e) => {
                eprintln!("{}: {}", out_path.to_string_lossy(), e);
                fs::write(out_path, ts.as_bytes()).unwrap();
            },
            Ok(formatted) => {
                fs::write(out_path, formatted.rendered.as_bytes()).unwrap();
            },
        };
    };

    // Fixed bytes
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        o.rust_field("z4RAFJ6AL", o.bytes("zSA2RONLC", o.fixed_range("z9K3TNON3", 4)), "f");
        write("fixed_bytes", s);
    }

    // Single byte int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        o.rust_field("z4RAFJ6AL", o.int("zSA2RONLC", o.fixed_range("z9K3TNON3", 1), Endian::Little, false), "f");
        write("int_singlebyte", s);
    }

    // Signed single-byte int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        o.rust_field("z4RAFJ6AL", o.int("zSA2RONLC", o.fixed_range("z9K3TNON3", 1), Endian::Little, true), "f");
        write("int_signed", s);
    }

    // Standard multi-byte int, LE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        o.rust_field("z4RAFJ6AL", o.int("zSA2RONLC", o.fixed_range("z9K3TNON3", 4), Endian::Little, true), "f");
        write("int_multibyte_le", s);
    }

    // Standard multi-byte int, BE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        o.rust_field("z4RAFJ6AL", o.int("zSA2RONLC", o.fixed_range("z9K3TNON3", 4), Endian::Big, true), "f");
        write("int_multibyte_be", s);
    }

    // Standard multi-byte int, LE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        o.rust_field("z4RAFJ6AL", o.int("zSA2RONLC", o.fixed_range("z9K3TNON3", 3), Endian::Little, true), "f");
        write("int_multibyte_npo2_le", s);
    }

    // Standard multi-byte int, BE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        o.rust_field("z4RAFJ6AL", o.int("zSA2RONLC", o.fixed_range("z9K3TNON3", 3), Endian::Big, true), "f");
        write("int_multibyte_npo2_be", s);
    }

    // Bitfields, single int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        let bitfield = o.fixed_range("z9K3TNON3", 1);
        o.rust_field("z4RAFJ6AL", o.int("zSA2RONLC", o.subrange(&bitfield, 0, 3), Endian::Little, false), "f");
        write("bitfield_single", s);
    }

    // Bitfields, multiple ints
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        let bitfield = o.fixed_range("z9K3TNON3", 1);
        o.rust_field("z4RAFJ6AL", o.int("zSA2RONLC", o.subrange(&bitfield, 0, 3), Endian::Little, false), "f");
        o.rust_field("zI7IWWPAA", o.int("zS1GUG7KF", o.subrange(&bitfield, 0, 5), Endian::Little, false), "g");
        write("bitfield_multiple", s);
    }

    // Const int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        o.rust_const(
            "zHCUNMDQQ",
            o.int("zSA2RONLC", o.fixed_range("z9K3TNON3", 4), Endian::Little, true),
            quote!(33),
        );
        write("const_int", s);
    }

    // Dynamic bytes
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("zJDI9H4JF", "T1");
        let len = o.int("zSA2RONLC", o.fixed_range("z9K3TNON3", 1), Endian::Little, false);
        o.rust_field("z4RAFJ6AL", o.dynamic_bytes("zFG9IBUAC", len), "f");
        write("dynamic_bytes", s);
    }

    // Dynamic array
    {
        let s = inarybay::schema::Schema::new();
        let top = s.object("zJDI9H4JF", "T1");
        let len = top.int("zSA2RONLC", top.fixed_range("z9K3TNON3", 1), Endian::Little, false);
        let (arr, arr_elem) = top.dynamic_array("zXZTIT4K3", len, "Thrusters");
        arr_elem.rust_field(
            "zFXRLSAHE",
            arr_elem.int("z4QM2N96U", arr_elem.fixed_range("zZTXFJH14", 1), Endian::Little, true),
            "f",
        );
        top.rust_field("z9F5CKEP5", arr, "thrusters");
        write("dynamic_array", s);
    }

    // Enum
    {
        let s = inarybay::schema::Schema::new();
        let top = s.object("zJDI9H4JF", "T1");
        let tag = top.int("zSA2RONLC", top.fixed_range("z9K3TNON3", 1), Endian::Little, false);
        let (enum_, enum_builder) = top.enum_("zR6V6CZJQ", tag, "August");
        let november = enum_builder.variant("zGRTFX9I3", "November", "November", quote!(0));
        november.rust_field(
            "zKEPRZWGF",
            november.int("zFCS161O2", november.fixed_range("zAZ2G3MG1", 1), Endian::Little, true),
            "f",
        );
        let december = enum_builder.variant("zQG5ZEV9Z", "December", "December", quote!(1));
        december.rust_field(
            "zFXRLSAHE",
            december.int("z4QM2N96U", december.fixed_range("zZTXFJH14", 1), Endian::Little, true),
            "f",
        );
        top.rust_field("z9F5CKEP5", enum_, "august");
        write("enum", s);
    }

    // Enum with external deps
    {
        let s = inarybay::schema::Schema::new();
        let top = s.object("zJDI9H4JF", "T1");
        let tag = top.int("zSA2RONLC", top.fixed_range("z9K3TNON3", 1), Endian::Little, false);
        let external = top.fixed_range("zZTXFJH14", 1);
        let (enum_, enum_builder) = top.enum_("zR6V6CZJQ", tag, "August");
        let november = enum_builder.variant("z71ZZPEGR", "November", "November", quote!(0));
        november.rust_field("zVMIA1US6", top.int("zLS23TK2G", external.clone(), Endian::Little, true), "f");
        let december = enum_builder.variant("zQG5ZEV9Z", "December", "December", quote!(1));
        december.rust_field("zFXRLSAHE", top.int("z4QM2N96U", external.clone(), Endian::Little, true), "f");
        top.rust_field("z9F5CKEP5", enum_, "august");
        write("enum_external_deps", s);
    }
}
