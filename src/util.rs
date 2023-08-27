use std::{
    ops::{
        Add,
        Sub,
        AddAssign,
        SubAssign,
    },
    fmt::Display,
};
use gc::{
    GcCell,
    Gc,
    Trace,
    Finalize,
};
use proc_macro2::Ident;
use quote::{
    IdentFragment,
    format_ident,
};

pub(crate) type LateInit<T> = Option<T>;

pub(crate) trait ToIdent {
    fn ident(&self) -> Ident;
}

impl<T: IdentFragment> ToIdent for T {
    fn ident(&self) -> Ident {
        return format_ident!("{}", self);
    }
}

pub(crate) type S<T> = Gc<GcCell<T>>;

pub(crate) fn new_s<T: Finalize + Trace>(t: T) -> S<T> {
    return Gc::new(GcCell::new(t));
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) struct BVec {
    pub(crate) bytes: usize,
    /// Excess of bytes
    pub(crate) bits: usize,
}

unsafe impl Trace for BVec {
    unsafe fn trace(&self) { }

    unsafe fn root(&self) { }

    unsafe fn unroot(&self) { }

    fn finalize_glue(&self) { }
}

impl Finalize for BVec {
    fn finalize(&self) { }
}

impl BVec {
    pub(crate) fn zero() -> BVec {
        return BVec {
            bytes: 0,
            bits: 0,
        };
    }

    pub(crate) fn bytes(l: usize) -> BVec {
        return BVec {
            bytes: l,
            bits: 0,
        };
    }
}

impl Display for BVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return format_args!("{}B {}b", self.bytes, self.bits).fmt(f);
    }
}

impl Add for BVec {
    type Output = BVec;

    fn add(self, rhs: Self) -> Self::Output {
        let bit_sum = self.bits + rhs.bits;
        return BVec {
            bytes: self.bytes + rhs.bytes + bit_sum / 8,
            bits: bit_sum % 8,
        };
    }
}

impl AddAssign for BVec {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for BVec {
    type Output = BVec;

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
        return BVec {
            bytes: avail_bytes,
            bits: avail_bits,
        };
    }
}

impl SubAssign for BVec {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl PartialOrd for BVec {
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
