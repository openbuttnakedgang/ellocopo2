use core::mem::{size_of_val, transmute};
use core::slice::from_raw_parts;
use core::convert::TryInto;

use num_enum::TryFromPrimitive;
use num_enum::IntoPrimitive;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Available types
///
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize) )]
pub enum Value<'a> {
    UNIT(()),
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

impl Default for TypeTag {
    fn default() -> Self {
        TypeTag::UNIT
    }
}

impl<'a> From<&Value<'a>> for &'a [u8] {
    fn from(v: &Value<'a>) -> &'a [u8] {
        use Value::*;
        match v {
            UNIT(_)  => &[0u8; 0],
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
            UNIT(_)  => TypeTag::UNIT,
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

impl<'a> TryInto<&'a str> for Value<'a> {
    type Error = crate::protocol::AnswerCode;

    fn try_into(self) -> Result<&'a str, Self::Error> {
        if let Value::STR(v) = self {
            Ok(v)
        } else {
            Err(crate::protocol::AnswerCode::ERR_TYPE)
        }
    }
}

impl<'a> TryInto<&'a [u8]> for Value<'a> {
    type Error = crate::protocol::AnswerCode;

    fn try_into(self) -> Result<&'a [u8], Self::Error> {
        if let Value::BYTES(v) = self {
            Ok(v)
        } else {
            Err(crate::protocol::AnswerCode::ERR_TYPE)
        }
    }
}

macro_rules! impl_try_into_value{
    ($val_var:ident, $ty:ty, $error_ty:path, $error_var:path) => {

        impl<'a> TryInto<$ty> for Value<'_> {
            type Error = $error_ty;

            fn try_into(self) -> Result<$ty, Self::Error> {
                if let Value::$val_var(v) = self {
                    Ok(v)
                } else {
                    Err($error_var)
                }
            }
        }
    };
}

impl_try_into_value!(UNIT, (), crate::protocol::AnswerCode, crate::protocol::AnswerCode::ERR_TYPE);
impl_try_into_value!(BOOL, bool, crate::protocol::AnswerCode, crate::protocol::AnswerCode::ERR_TYPE);
impl_try_into_value!(U8,   u8, crate::protocol::AnswerCode, crate::protocol::AnswerCode::ERR_TYPE);
impl_try_into_value!(I8,   i8, crate::protocol::AnswerCode, crate::protocol::AnswerCode::ERR_TYPE);
impl_try_into_value!(U16, u16, crate::protocol::AnswerCode, crate::protocol::AnswerCode::ERR_TYPE);
impl_try_into_value!(I16, i16, crate::protocol::AnswerCode, crate::protocol::AnswerCode::ERR_TYPE);
impl_try_into_value!(U32, u32, crate::protocol::AnswerCode, crate::protocol::AnswerCode::ERR_TYPE);
impl_try_into_value!(I32, i32, crate::protocol::AnswerCode, crate::protocol::AnswerCode::ERR_TYPE);


