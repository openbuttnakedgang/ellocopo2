use core::mem::{size_of_val, transmute};
use core::slice::from_raw_parts;

use num_enum::TryFromPrimitive;
use num_enum::IntoPrimitive;

/// Available types
///
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value<'a> {
    UNIT,
    BOOL(bool),
    I32(i32),
    I16(i16),
    I8(i8),
    U32(u32),
    U16(u16),
    U8(u8),
    STR(&'a str),
    BYTES(&'a [u8]),
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum TypeTag {
    UNIT = 0,  
    BOOL = 1,
    I32 = 2, 
    I16 = 3,
    I8 = 4,
    U32 = 5,
    U16 = 6,
    U8 = 7,
    STR = 8,
    BYTES = 9,    
}

impl core::default::Default for TypeTag {
    fn default() -> Self {
        TypeTag::UNIT
    }
}

impl<'a> From<&Value<'a>> for &'a [u8] {
    fn from(v: &Value<'a>) -> &'a [u8] {
        use Value::*;
        match v {
            UNIT     => &[0u8; 0],
            BOOL(v)  => unsafe { from_raw_parts(transmute(v), size_of_val(v)) },
            I32(v)   => unsafe { from_raw_parts(transmute(v), size_of_val(v)) },
            I16(v)   => unsafe { from_raw_parts(transmute(v), size_of_val(v)) },
            I8(v)    => unsafe { from_raw_parts(transmute(v), size_of_val(v)) },
            U32(v)   => unsafe { from_raw_parts(transmute(v), size_of_val(v)) },
            U16(v)   => unsafe { from_raw_parts(transmute(v), size_of_val(v)) },
            U8(v)    => unsafe { from_raw_parts(transmute(v), size_of_val(v)) },
            STR(v)   => v.as_bytes(),
            BYTES(v) => v,
        }
    }
}

impl From<&Value<'_>> for TypeTag {
    fn from(v: &Value) -> TypeTag {
        use Value::*;
        match v {
            UNIT     => TypeTag::UNIT,
            BOOL(_)  => TypeTag::BOOL,
            I32(_)   => TypeTag::I32, 
            I16(_)   => TypeTag::I16, 
            I8(_)    => TypeTag::I8, 
            U32(_)   => TypeTag::U32, 
            U16(_)   => TypeTag::U16, 
            U8(_)    => TypeTag::U8, 
            STR(_)   => TypeTag::STR, 
            BYTES(_) => TypeTag::BYTES, 
        }
    }
}


