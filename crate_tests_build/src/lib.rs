use std::{
    path::PathBuf,
    fs::{
        self,
    },
};
use inarybay::{
    schema::Schema,
    object::Endian,
};
use quote::quote;

pub fn generate(root: PathBuf) {
    let src = root.join("src");
    let write = |name: &str, s: Schema| {
        let out_path = src.join(format!("gen_{}.rs", name));
        let ts = s.generate(inarybay::schema::GenerateConfig {
            read: true,
            write: true,
            sync_: true,
            async_: true,
            low_heap: false,
        }).to_string();
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
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.bytes("f_val", o.fixed_range("range0", 4)));
        write("fixed_bytes", s);
    }

    // Single byte int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 1), Endian::Little, false));
        write("int_singlebyte", s);
    }

    // Signed single-byte int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 1), Endian::Little, true));
        write("int_signed", s);
    }

    // Standard multi-byte int, LE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 4), Endian::Little, true));
        write("int_multibyte_le", s);
    }

    // Standard multi-byte int, BE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 4), Endian::Big, true));
        write("int_multibyte_be", s);
    }

    // Standard multi-byte int, LE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 3), Endian::Little, true));
        write("int_multibyte_npo2_le", s);
    }

    // Standard multi-byte int, BE
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.int("f_val", o.fixed_range("range0", 3), Endian::Big, true));
        write("int_multibyte_npo2_be", s);
    }

    // Bitfields, single int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        let bitfield = o.fixed_range("range0", 1);
        o.rust_field("f", o.int("f_val", o.subrange(&bitfield, 0, 3), Endian::Little, false));
        write("bitfield_single", s);
    }

    // Bitfields, multiple ints
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        let bitfield = o.fixed_range("range0", 1);
        o.rust_field("f", o.int("f_val", o.subrange(&bitfield, 0, 3), Endian::Little, false));
        o.rust_field("g", o.int("g_val", o.subrange(&bitfield, 0, 5), Endian::Little, false));
        write("bitfield_multiple", s);
    }

    // Const int
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_const(
            "range0_magic",
            o.int("range0_int", o.fixed_range("range0", 4), Endian::Little, true),
            quote!(33i32),
        );
        write("const_int", s);
    }

    // Bool
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.bool("f_val_bool", o.int("f_val", o.fixed_range("range0", 1), Endian::Little, false)));
        write("bool_", s);
    }

    // Float
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.float("f_val", o.fixed_range("range0", 8), Endian::Little));
        write("float_", s);
    }

    // Alignment
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.int("g_val", o.fixed_range("range0", 1), Endian::Little, false));
        o.align("align", 4);
        o.rust_field("g", o.int("f_val", o.fixed_range("range1", 1), Endian::Little, false));
        write("align", s);
    }

    // Delimited bytes
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("g", o.delimited_bytes("g_val", b"\n\n"));
        o.rust_field("f", o.bytes("f_val", o.fixed_range("range0", 4)));
        write("delimited_bytes", s);
    }

    // Dynamic bytes
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        let len = o.int("f_len", o.fixed_range("range0", 1), Endian::Little, false);
        o.rust_field("f", o.dynamic_bytes("f_val", len));
        write("dynamic_bytes", s);
    }

    // Remaining bytes
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("f", o.bytes("f_val", o.fixed_range("range0", 4)));
        o.rust_field("g", o.remaining_bytes("g_val"));
        write("remaining_bytes", s);
    }

    // Dynamic array
    {
        let s = inarybay::schema::Schema::new();
        let top = s.object("root", "T1");
        top.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        let len = top.int("thrusters_len", top.fixed_range("range0", 1), Endian::Little, false);
        {
            let (arr, arr_elem) = top.dynamic_array("thrusters_val", len, "Thrusters");
            arr_elem.rust_field(
                "f",
                arr_elem.int("f_val", arr_elem.fixed_range("range1", 1), Endian::Little, true),
            );
            arr_elem.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
            top.rust_field("thrusters", arr);
        }
        write("dynamic_array", s);
    }

    // Enum
    {
        let s = inarybay::schema::Schema::new();
        let top = s.object("root", "T1");
        top.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        let tag = top.int("august_tag", top.fixed_range("range0", 1), Endian::Little, false);
        {
            let (enum_, enum_builder) = top.enum_("august_val", tag, "August");
            enum_builder.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
            {
                let november = enum_builder.variant("var_nov", "November", "November", quote!(0u8));
                november.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
                november.rust_field(
                    "f",
                    november.int("f_val", november.fixed_range("range1", 1), Endian::Little, true),
                );
            }
            {
                let december = enum_builder.variant("var_dec", "December", "December", quote!(1u8));
                december.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
                december.rust_field(
                    "f",
                    december.int("f_val", december.fixed_range("range1", 1), Endian::Little, true),
                );
            }
            top.rust_field("august", enum_);
        }
        write("enum", s);
    }

    // Enum with external deps
    {
        let s = inarybay::schema::Schema::new();
        let top = s.object("root", "T1");
        top.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        let tag = top.int("august_tag", top.fixed_range("range0", 1), Endian::Little, false);
        let external = top.fixed_range("shared0", 1);
        {
            let (enum_, enum_builder) = top.enum_("august_val", tag, "August");
            enum_builder.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
            {
                let november = enum_builder.variant("var_nov", "November", "November", quote!(0u8));
                november.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
                november.rust_field("f", november.int("f_val1", external.clone(), Endian::Little, true));
            }
            {
                let december = enum_builder.variant("var_dec", "December", "December", quote!(1u8));
                december.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
                december.rust_field("f", december.int("f_val2", external.clone(), Endian::Little, true));
            }
            top.rust_field("august", enum_);
        }
        write("enum_external_deps", s);
    }

    // String
    {
        let s = inarybay::schema::Schema::new();
        let o = s.object("root", "T1");
        o.add_type_attrs(quote!(#[derive(Debug, PartialEq)]));
        o.rust_field("g", o.string_utf8("g_str", o.remaining_bytes("g_val")));
        write("string", s);
    }
}
