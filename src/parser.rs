
use core::convert::TryFrom;

use crate::ty::*;
use crate::protocol::*;

pub enum ParseResult<'a> {
    Ok(Value<'a>),
    Err(AnswerCode),
    NeedMoreData,
}


//#[inline(always)]
//fn sign_parser(i: &[u8]) -> ParseResult {
//    if i[0] != self::SIGN {
//        return Err(AnswerCode::BadProtocol);
//    }
//
//    Ok((&i[1..], ()))
//}

//#[inline(always)]
//fn op_parser(i: &[u8]) -> ParseResult<OperationStatus> {
//
//    let val = i[0] as i8;
//    match OperationStatus::try_from(val) {
//        Ok(v) => Ok((&i[1..], v)),
//        Err(_) => Err(Error::BadProtocol),
//    }
//}
//
//#[inline(always)]
//fn typpe_parser(i: &[u8]) -> ParseResult<()> {
//    Ok((&i[1..], ()))
//}
//
//#[inline(always)]
//fn name_sz_parser(i: &[u8]) -> ParseResult<usize> {
//    let name_sz = i[0] as usize;
//    Ok((&i[1..], name_sz))
//}
//
//
//#[inline(always)]
//fn name_parser<'a>(i: &'a [u8], sz: usize) -> ParseResult<&'a str> {
//    // TODO: think about msg len check and panic behaviour
//    use core::str::from_utf8_unchecked;
//    let name = unsafe { from_utf8_unchecked(&i[..sz as usize]) };
//
//    Ok((&i[sz..], name))
//}
//
//#[inline(always)]
//fn value_parser(i: &[u8]) -> ParseResult<(RegTypeId, &[u8])> {
//    use core::convert::TryFrom;
//    use RegTypeId::*;
//
//    if i.len() < 1 { return Err(Error::BadParam) }
//    let type_id = i[0];
//    let i = &i[1..];
//
//    let type_id = match RegTypeId::try_from(type_id) {
//        Ok(v) => v,
//        Err(_) => return Err(Error::BadFormat),
//    };
//
//    match type_id {
//        UNIT => {
//            if i.len() != 0 { return Err(Error::BadParam) }
//            Ok((&[], (type_id, &[])))
//        }
//        I32 | U32 => {
//            if i.len() != 4 { return Err(Error::BadParam) }
//            Ok((&[], (type_id, i)))
//        }
//        I16 | U16 => {
//            if i.len() != 2 { return Err(Error::BadParam) }
//            Ok((&[], (type_id, i)))
//        }
//        I8 | U8 => {
//            if i.len() != 1 { return Err(Error::BadParam) }
//            Ok((&[], (type_id, i)))
//        }
//        STR | BYTES => {
//            if i.len() < 1 { return Err(Error::BadParam) }
//            let sz = i[0] as usize;
//            let i = &i[1..];
//
//            if i.len() < sz { return Err(Error::BadParam) }
//            Ok((&[], (type_id, i)))
//        }
//    }
//}
