mod gen_fixed_bytes;
mod gen_int_singlebyte;
mod gen_int_signed;
mod gen_int_multibyte_le;
mod gen_int_multibyte_be;
mod gen_int_multibyte_npo2_le;
mod gen_int_multibyte_npo2_be;
mod gen_bitfield_single;
mod gen_bitfield_multiple;
mod gen_const_int;
mod gen_bool_;
mod gen_float_;
mod gen_align;
mod gen_delimited_bytes;
mod gen_dynamic_bytes;
mod gen_remaining_bytes;
mod gen_dynamic_array;
mod gen_enum;
mod gen_enum_default;
mod gen_enum_external_deps;
mod gen_string;

macro_rules! round_trip{
    ($test: ident, $asynctest: ident; $t: ty, $e: expr) => {
        round_trip!($test, $asynctest; $t, $e, []);
    };
    ($test: ident, $asynctest: ident; $t: ty, $e: expr, $mid: expr) => {
        #[test] fn $test() {
            let start = $e;
            let mut bytes = vec![];
            start.write(&mut bytes).unwrap();
            let mid = $mid;
            if mid.len() > 0 {
                assert_eq!(bytes, mid, "Written data doesn't quite match the expected serialized bytes");
            }
            let end =< $t >:: read(&mut std::io::Cursor::new(&bytes)).unwrap();
            assert_eq!(start, end, "Round trip failed; intermediate: {:?}", bytes);
        }
        #[tokio::test] async fn $asynctest() {
            let start = $e;
            let mut bytes = vec![];
            start.write_async(&mut bytes).await.unwrap();
            let mid = $mid;
            if mid.len() > 0 {
                assert_eq!(bytes, mid);
            }
            let end =< $t >:: read_async(&mut futures::io::Cursor::new(&bytes)).await.unwrap();
            assert_eq!(start, end, "Round trip failed; intermediate: {:?}", bytes);
        }
    };
}

round_trip!(
    test_fixed_bytes,
    test_fixed_bytes_async;
    gen_fixed_bytes::T1,
    gen_fixed_bytes::T1 { f: [1u8, 12u8, 0u8, 3u8] }
);

round_trip!(
    test_int_singlebyte,
    test_int_singlebyte_async;
    gen_int_singlebyte::T1,
    gen_int_singlebyte::T1 { f: 29u8 }
);

round_trip!(
    test_int_signed,
    test_int_signed_async;
    gen_int_signed::T1,
    gen_int_signed::T1 { f: -122i8 }
);

round_trip!(
    test_int_multibyte_le,
    test_int_multibyte_le_async;
    gen_int_multibyte_le::T1,
    gen_int_multibyte_le::T1 { f: -3 }
);

round_trip!(
    test_int_multibyte_be,
    test_int_multibyte_be_async;
    gen_int_multibyte_be::T1,
    gen_int_multibyte_be::T1 { f: -3 }
);

round_trip!(
    test_int_multibyte_npo2_le,
    test_int_multibyte_npo2_le_async;
    gen_int_multibyte_npo2_le::T1,
    gen_int_multibyte_npo2_le::T1 { f: -3 }
);

round_trip!(
    test_int_multibyte_npo2_be,
    test_int_multibyte_npo2_be_async;
    gen_int_multibyte_npo2_be::T1,
    gen_int_multibyte_npo2_be::T1 { f: -3 }
);

round_trip!(
    test_bitfield_single,
    test_bitfield_single_async;
    gen_bitfield_single::T1,
    gen_bitfield_single::T1 { f: 5 }
);

round_trip!(
    test_bitfield_multiple,
    test_bitfield_multiple_async;
    gen_bitfield_multiple::T1,
    gen_bitfield_multiple::T1 {
        f: 5,
        g: 2,
    }
);

round_trip!(
    test_const_int,
    test_const_int_async;
    gen_const_int::T1,
    gen_const_int::T1 {}
);

round_trip!(
    test_align,
    test_align_async;
    gen_align::T1,
    gen_align::T1 {
        f: 1,
        g: 1,
    },
    [1u8, 0u8, 0u8, 0u8, 1u8]
);

round_trip!(
    test_bool,
    test_bool_async;
    gen_bool_::T1,
    gen_bool_::T1 { f: true }
);

round_trip!(
    test_float,
    test_float_async;
    gen_float_::T1,
    gen_float_::T1 { f: -32.132 }
);

round_trip!(
    test_delimited_bytes,
    test_delimited_bytes_async;
    gen_delimited_bytes::T1,
    gen_delimited_bytes::T1 {
        f: *b"helo",
        g: b"1234567890".to_vec(),
    }
);

round_trip!(
    test_dynamic_bytes,
    test_dynamic_bytes_async;
    gen_dynamic_bytes::T1,
    gen_dynamic_bytes::T1 { f: b"hello".to_vec() }
);

round_trip!(
    test_remaining_bytes,
    test_remaining_bytes_async;
    gen_remaining_bytes::T1,
    gen_remaining_bytes::T1 {
        f: *b"helo",
        g: b"1234567890".to_vec(),
    }
);

round_trip!(
    test_dynamic_array,
    test_dynamic_array_async;
    gen_dynamic_array::T1,
    gen_dynamic_array::T1 { thrusters: vec![gen_dynamic_array::Thrusters { f: 7 }] }
);

round_trip!(
    test_enum,
    test_enum_async;
    gen_enum::T1,
    gen_enum::T1 { august: gen_enum::August::December(gen_enum::December { f: 107 }) }
);

round_trip!(
    test_enum_default,
    test_enum_default_async;
    gen_enum_default::T1,
    gen_enum_default::T1 { august: gen_enum_default::August::December(gen_enum_default::December { what: 3 }) }
);

round_trip!(
    test_enum_external_deps,
    test_enum_external_deps_async;
    gen_enum_external_deps::T1,
    gen_enum_external_deps::T1 {
        august: gen_enum_external_deps::August::December(gen_enum_external_deps::December { f: 107 }),
    }
);

round_trip!(
    test_string,
    test_string_async;
    gen_string::T1,
    gen_string::T1 { g: "the last test".to_string() }
);
