use std::{
    path::PathBuf,
    env,
    fs::{
        self,
    },
    str::FromStr,
};
use inarybay::{
    schema::Schema,
    node_int::Endian,
};
use quote::quote;

pub fn generate() {
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
        let o = s.object("root", "T1");
        o.rust_field("f", o.bytes("f_val", o.fixed_range("range0", 4)));
        write("fixed_bytes", s);
    }

    // Single byte int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 1), Endian::Little, false));
        write("int_singlebyte", s);
    }

    // Signed single-byte int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 1), Endian::Little, true));
        write("int_signed", s);
    }

    // Standard multi-byte int, LE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 4), Endian::Little, true));
        write("int_multibyte_le", s);
    }

    // Standard multi-byte int, BE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 4), Endian::Big, true));
        write("int_multibyte_be", s);
    }

    // Standard multi-byte int, LE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 3), Endian::Little, true));
        write("int_multibyte_npo2_le", s);
    }

    // Standard multi-byte int, BE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 3), Endian::Big, true));
        write("int_multibyte_npo2_be", s);
    }

    // Bitfields, single int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        let bitfield = o.fixed_range("range0", 1);
        o.rust_field("f", o.int("f_val", o.subrange(&bitfield, 0, 3), Endian::Little, false));
        write("bitfield_single", s);
    }

    // Bitfields, multiple ints
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        let bitfield = o.fixed_range("range0", 1);
        o.rust_field("f", o.int("f_val", o.subrange(&bitfield, 0, 3), Endian::Little, false));
        o.rust_field("g", o.int("g_val", o.subrange(&bitfield, 0, 5), Endian::Little, false));
        write("bitfield_multiple", s);
    }

    // Const int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.rust_const(
            "range0_magic",
            o.int("range0_int", o.fixed_range("range0", 4), Endian::Little, true),
            quote!(33),
        );
        write("const_int", s);
    }

    // Dynamic bytes
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        let len = o.int("f_len", o.fixed_range("range0", 1), Endian::Little, false);
        o.rust_field("f", o.dynamic_bytes("f_val", len));
        write("dynamic_bytes", s);
    }

    // Dynamic array
    {
        let s = inarybay::schema::Schema::new();
        let top = s.object("root", "T1");
        let len = top.int("thrusters_len", top.fixed_range("range0", 1), Endian::Little, false);
        let (arr, arr_elem) = top.dynamic_array("thrusters_val", len, "Thrusters");
        arr_elem.rust_field("f", arr_elem.int("f_val", arr_elem.fixed_range("range1", 1), Endian::Little, true));
        top.rust_field("thrusters", arr);
        write("dynamic_array", s);
    }

    // Enum
    {
        let s = inarybay::schema::Schema::new();
        let top = s.object("root", "T1");
        let tag = top.int("august_tag", top.fixed_range("range0", 1), Endian::Little, false);
        let (enum_, enum_builder) = top.enum_("august_val", tag, "August");
        let november = enum_builder.variant("var_nov", "November", "November", quote!(0));
        november.rust_field("f", november.int("f_val", november.fixed_range("range1", 1), Endian::Little, true));
        let december = enum_builder.variant("var_dec", "December", "December", quote!(1));
        december.rust_field("f", december.int("f_val", december.fixed_range("range1", 1), Endian::Little, true));
        top.rust_field("august", enum_);
        write("enum", s);
    }

    // Enum with external deps
    {
        let s = inarybay::schema::Schema::new();
        let top = s.object("root", "T1");
        let tag = top.int("august_tag", top.fixed_range("range0", 1), Endian::Little, false);
        let external = top.fixed_range("shared0", 1);
        let (enum_, enum_builder) = top.enum_("august_val", tag, "August");
        let november = enum_builder.variant("var_nov", "November", "November", quote!(0));
        november.rust_field("f", november.int("f_val1", external.clone(), Endian::Little, true));
        let december = enum_builder.variant("var_dec", "December", "December", quote!(1));
        december.rust_field("f", december.int("f_val2", external.clone(), Endian::Little, true));
        top.rust_field("august", enum_);
        write("enum_external_deps", s);
    }
}