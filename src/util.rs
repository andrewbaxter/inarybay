use std::{
    cell::RefCell,
    ops::{
        Add,
        Sub,
        AddAssign,
        SubAssign,
    },
    fmt::Display,
};
use proc_macro2::Ident;
use quote::{
    IdentFragment,
    format_ident,
};

pub(crate) trait ToIdent {
    fn ident(&self) -> Ident;
}

impl<T: IdentFragment> ToIdent for T {
    fn ident(&self) -> Ident {
        return format_ident!("{}", self);
    }
}

pub(crate) type S<T> = &'static RefCell<T>;

pub(crate) fn new_s<T>(t: T) -> S<T> {
    return Box::leak(Box::new(RefCell::new(t)));
}

#[derive(PartialEq)]
pub(crate) struct Coord {
    pub(crate) bytes: usize,
    /// Excess of bytes
    pub(crate) bits: usize,
}

impl Coord {
    pub(crate) fn zero() -> Coord {
        return Coord {
            bytes: 0,
            bits: 0,
        };
    }

    pub(crate) fn bytes(l: usize) -> Coord {
        return Coord {
            bytes: l,
            bits: 0,
        };
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return format_args!("{}B {}b", self.bytes, self.bits).fmt(f);
    }
}

impl Add for Coord {
    type Output = Coord;

    fn add(self, rhs: Self) -> Self::Output {
        let bit_sum = self.bits + rhs.bits;
        return Coord {
            bytes: self.bytes + rhs.bytes + bit_sum / 8,
            bits: bit_sum % 8,
        };
    }
}

impl AddAssign for Coord {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Coord {
    type Output = Coord;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut avail_bits = self.bits;
        let mut avail_bytes = self.bytes;
        let mut bits = rhs.bits;
        if bits > self.bits {
            bits -= avail_bits;
            avail_bits = 0;
        }
        if bits > 0 {
            avail_bytes -= 1;
            avail_bits = 8 - bits;
        }
        avail_bytes -= rhs.bytes;
        return Coord {
            bytes: avail_bytes,
            bits: avail_bits,
        };
    }
}

impl SubAssign for Coord {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl PartialOrd for Coord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.bytes.partial_cmp(&other.bytes) {
            Some(std::cmp::Ordering::Less) => return Some(std::cmp::Ordering::Less),
            Some(std::cmp::Ordering::Equal) => return Some(self.bits.cmp(&other.bits)),
            Some(std::cmp::Ordering::Greater) => return Some(std::cmp::Ordering::Greater),
            None => return None,
        }
    }
}

#[macro_export]
macro_rules! breaker{
    ($b: block) => {
        loop {
            $b break;
        }
    };
    ($l: lifetime $b: block) => {
        $l loop {
            $b break;
        }
    };
}
