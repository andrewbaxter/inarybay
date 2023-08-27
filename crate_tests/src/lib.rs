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
    ($t: ty, $e: expr) => {
        {
            let start = $e;
            let mut bytes = vec![];
            start.write(&mut bytes).unwrap();
            let end =< $t >:: read(&mut Cursor::new(bytes)).unwrap();
            assert_eq!(start, end);
        }
    };
}

#[test]
fn test_fixed_bytes() {
    round_trip!(gen_fixed_bytes::T1, gen_fixed_bytes::T1 { f: [1u8, 12u8, 0u8, 3u8] });
}

#[test]
fn test_int_singlebyte() {
    round_trip!(gen_int_singlebyte::T1, gen_int_singlebyte::T1 { f: 29u8 });
}

#[test]
fn test_int_signed() {
    round_trip!(gen_int_signed::T1, gen_int_signed::T1 { f: -122i8 });
}

#[test]
fn test_int_multibyte_le() {
    round_trip!(gen_int_multibyte_le::T1, gen_int_multibyte_le::T1 { f: -3 });
}

#[test]
fn test_int_multibyte_be() {
    round_trip!(gen_int_multibyte_be::T1, gen_int_multibyte_be::T1 { f: -3 });
}

#[test]
fn test_int_multibyte_npo2_le() {
    round_trip!(gen_int_multibyte_npo2_le::T1, gen_int_multibyte_npo2_le::T1 { f: -3 });
}

#[test]
fn test_int_multibyte_npo2_be() {
    round_trip!(gen_int_multibyte_npo2_be::T1, gen_int_multibyte_npo2_be::T1 { f: -3 });
}

#[test]
fn test_bitfield_single() {
    round_trip!(gen_bitfield_single::T1, gen_bitfield_single::T1 { f: 5 });
}

#[test]
fn test_bitfield_multiple() {
    round_trip!(gen_bitfield_multiple::T1, gen_bitfield_multiple::T1 {
        f: 5,
        g: 2,
    });
}

#[test]
fn test_const_int() {
    round_trip!(gen_const_int::T1, gen_const_int::T1 {});
}

#[test]
fn test_dynamic_bytes() {
    round_trip!(gen_dynamic_bytes::T1, gen_dynamic_bytes::T1 { f: b"hello".to_vec() });
}

#[test]
fn test_dynamic_array() {
    round_trip!(
        gen_dynamic_array::T1,
        gen_dynamic_array::T1 { thrusters: vec![gen_dynamic_array::Thrusters { f: 7 }] }
    );
}

#[test]
fn test_enum() {
    round_trip!(gen_enum::T1, gen_enum::T1 { august: gen_enum::August::December(gen_enum::December { f: 107 }) });
}

#[test]
fn test_enum_external_deps() {
    round_trip!(
        gen_enum_external_deps::T1,
        gen_enum_external_deps::T1 {
            august: gen_enum_external_deps::August::December(gen_enum_external_deps::December { f: 107 }),
        }
    );
}
