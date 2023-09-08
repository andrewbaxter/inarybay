use std::{
    path::PathBuf,
    fs::{
        self,
    },
};
use inarybay::{
    schema::{
        Schema,
        GenerateConfig,
    },
    scope::Endian,
};
use quote::quote;

mod example {
    #![allow(dead_code)]

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
}

pub fn generate(root: PathBuf) {
    let src = root.join("src");
    let write = |name: &str, s: Schema| {
        let out_path = src.join(format!("gen_{}.rs", name));
        fs::write(out_path, s.generate().as_bytes()).unwrap();
    };
    let config = GenerateConfig {
        prefix: None,
        read: true,
        write: true,
        sync_: true,
        async_: true,
        simple_errors: false,
    };

    // Fixed bytes
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.bytes("f_val", scope.fixed_range("range0", 4)));
        write("fixed_bytes", schema);
    }

    // Single byte int
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("f_val", scope.fixed_range("range0", 1), Endian::Little, false));
        write("int_singlebyte", schema);
    }

    // Signed single-byte int
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("f_val", scope.fixed_range("range0", 1), Endian::Little, true));
        write("int_signed", schema);
    }

    // Standard multi-byte int, LE
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("f_val", scope.fixed_range("range0", 4), Endian::Little, true));
        write("int_multibyte_le", schema);
    }

    // Standard multi-byte int, BE
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("f_val", scope.fixed_range("range0", 4), Endian::Big, true));
        write("int_multibyte_be", schema);
    }

    // Standard multi-byte int, LE
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("f_val", scope.fixed_range("range0", 3), Endian::Little, true));
        write("int_multibyte_npo2_le", schema);
    }

    // Standard multi-byte int, BE
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("f_val", scope.fixed_range("range0", 3), Endian::Big, true));
        write("int_multibyte_npo2_be", schema);
    }

    // Bitfields, single int
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let bitfield = scope.fixed_range("range0", 1);
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("f_val", scope.subrange(&bitfield, 0, 3), Endian::Little, false));
        write("bitfield_single", schema);
    }

    // Bitfields, multiple ints
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let bitfield = scope.fixed_range("range0", 1);
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("f_val", scope.subrange(&bitfield, 0, 3), Endian::Little, false));
        obj.field("g", scope.int("g_val", scope.subrange(&bitfield, 0, 5), Endian::Little, false));
        write("bitfield_multiple", schema);
    }

    // Const int
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        scope.const_(
            "range0_magic",
            scope.int("range0_int", scope.fixed_range("range0", 4), Endian::Little, true),
            quote!(33i32),
        );
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        write("const_int", schema);
    }

    // Bool
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field(
            "f",
            scope.bool("f_val_bool", scope.int("f_val", scope.fixed_range("range0", 1), Endian::Little, false)),
        );
        write("bool_", schema);
    }

    // Float
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.float("f_val", scope.fixed_range("range0", 8), Endian::Little));
        write("float_", schema);
    }

    // Alignment
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("g_val", scope.fixed_range("range0", 1), Endian::Little, false));
        scope.align("align", 0, 4);
        obj.field("g", scope.int("f_val", scope.fixed_range("range1", 1), Endian::Little, false));
        write("align", schema);
    }

    // Alignment with shift
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.int("g_val", scope.fixed_range("range0", 1), Endian::Little, false));
        scope.align("align", 1, 4);
        obj.field("g", scope.int("f_val", scope.fixed_range("range1", 1), Endian::Little, false));
        write("align_shift", schema);
    }

    // Delimited bytes
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("g", scope.delimited_bytes("g_val", b"\n\n"));
        obj.field("f", scope.bytes("f_val", scope.fixed_range("range0", 4)));
        write("delimited_bytes", schema);
    }

    // Dynamic bytes
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let len = scope.int("f_len", scope.fixed_range("range0", 1), Endian::Little, false);
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.dynamic_bytes("f_val", len));
        write("dynamic_bytes", schema);
    }

    // Remaining bytes
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("f", scope.bytes("f_val", scope.fixed_range("range0", 4)));
        obj.field("g", scope.remaining_bytes("g_val"));
        write("remaining_bytes", schema);
    }

    // Dynamic array
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        let len = scope.int("thrusters_len", scope.fixed_range("range0", 1), Endian::Little, false);
        {
            let (arr, arr_scope) = scope.dynamic_array("thrusters_val", len);
            let arr_obj = arr_scope.object("thrusters_obj", "Thrusters");
            arr_scope.rust_root(arr_obj.clone());
            arr_obj.field("f", arr_scope.int("f_val", arr_scope.fixed_range("range1", 1), Endian::Little, true));
            arr_obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
            obj.field("thrusters", arr);
        }
        write("dynamic_array", schema);
    }

    // Enum
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let tag = scope.int("august_tag", scope.fixed_range("range0", 1), Endian::Little, false);
        let enum_ = {
            let enum_ = scope.enum_("august_val", tag, "August");
            enum_.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
            {
                let november = enum_.variant("var_nov", "November", quote!(0u8));
                let obj = november.object("nov_obj", "November");
                obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
                obj.field("f", november.int("f_val", november.fixed_range("range1", 1), Endian::Little, true));
                november.rust_root(obj);
            }
            {
                let december = enum_.variant("var_dec", "December", quote!(1u8));
                let obj = december.object("dec_obj", "December");
                obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
                obj.field("f", december.int("f_val", december.fixed_range("range1", 1), Endian::Little, true));
                december.rust_root(obj);
            }
            enum_
        };
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("august", enum_);
        write("enum", schema);
    }

    // Enum default variant
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let tag = scope.int("august_tag", scope.fixed_range("range0", 1), Endian::Little, false);
        let enum_ = {
            let enum_ = scope.enum_("august_val", tag, "August");
            enum_.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
            {
                let november = enum_.variant("var_nov", "November", quote!(0u8));
                let obj = november.object("nov_obj", "November");
                obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
                obj.field("f", november.int("f_val", november.fixed_range("range1", 1), Endian::Little, true));
                november.rust_root(obj);
            }
            {
                let (december, tag) = enum_.default("var_dec", "December");
                let obj = december.object("dec_obj", "December");
                obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
                obj.field("what", tag);
                december.rust_root(obj);
            }
            enum_
        };
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("august", enum_);
        write("enum_default", schema);
    }

    // Enum with external deps
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let tag = scope.int("august_tag", scope.fixed_range("range0", 1), Endian::Little, false);
        let external = scope.fixed_range("shared0", 1);
        let enum_ = {
            let enum_ = scope.enum_("august_val", tag, "August");
            enum_.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
            {
                let november = enum_.variant("var_nov", "November", quote!(0u8));
                let obj = november.object("nov_obj", "November");
                obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
                obj.field("f", november.int("f_val1", external.clone(), Endian::Little, true));
                november.rust_root(obj);
            }
            {
                let december = enum_.variant("var_dec", "December", quote!(1u8));
                let obj = december.object("dec_obj", "December");
                obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
                obj.field("f", december.int("f_val2", external.clone(), Endian::Little, true));
                december.rust_root(obj);
            }
            enum_
        };
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("august", enum_);
        write("enum_external_deps", schema);
    }

    // String
    {
        let schema = inarybay::schema::Schema::new();
        let scope = schema.scope("root", config.clone());
        let obj = scope.object("obj", "T1");
        scope.rust_root(obj.clone());
        obj.add_type_attrs(quote!(#[derive(Clone, Debug, PartialEq)]));
        obj.field("g", scope.string_utf8("g_str", scope.remaining_bytes("g_val")));
        write("string", schema);
    }
}
