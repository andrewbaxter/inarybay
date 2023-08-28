use std::io::Cursor;

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
mod gen_dynamic_bytes;
mod gen_dynamic_array;
mod gen_enum;
mod gen_enum_external_deps;

macro_rules! round_trip{
    ($test: ident, $asynctest: ident; $t: ty, $e: expr) => {
        #[test] fn $test() {
            let start = $e;
            let mut bytes = vec![];
            start.write(&mut bytes).unwrap();
            let end =< $t >:: read(&mut Cursor::new(&bytes)).unwrap();
            assert_eq!(start, end, "Round trip failed; intermediate: {:?}", bytes);
        }
        #[tokio::test] async fn $asynctest() {
            let start = $e;
            let mut bytes = vec![];
            start.write_async(&mut bytes).await.unwrap();
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
    test_dynamic_bytes,
    test_dynamic_bytes_async;
    gen_dynamic_bytes::T1,
    gen_dynamic_bytes::T1 { f: b"hello".to_vec() }
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
    test_enum_external_deps,
    test_enum_external_deps_async;
    gen_enum_external_deps::T1,
    gen_enum_external_deps::T1 {
        august: gen_enum_external_deps::August::December(gen_enum_external_deps::December { f: 107 }),
    }
);
