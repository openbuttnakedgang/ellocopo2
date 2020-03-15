
use core::convert::TryFrom;
use core::ops::Range;

use crate::ty::*;
use crate::protocol::*;


#[derive(Debug)]
pub enum ParserError {
    NeedMoreData,
    BadCode,
    BadHeader,
    BadPathSz,
    BadPayloadSz,
    BadTypeID,
    BadValue,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Msg<'a> (
    pub AnswerCode,
    pub &'a str,
    pub Value<'a>
);

pub type ParseResult<'a> = Result<Msg<'a>,ParserError>;

pub enum ParseState {
    ParsingHeader,
    ParsingPath,
    ParsingValue,
    ParsingDone,
}

pub struct ParseMsg {
    header: Header,
    path: Range<usize>,
    payload: Range<usize>,
    pos: usize,
    state: ParseState,
}

impl ParseMsg {
    pub fn new() -> Self { 
        ParseMsg { 
            path: 0 .. 0,
            payload: 0 .. 0, 
            pos: 0,
            state: ParseState::ParsingHeader,
            header: Default::default(),
        } 
    }

    pub fn reset(&mut self) {
        self.pos = 0;
        self.state = ParseState::ParsingHeader;
    }

    pub fn try_parse<'a>(&mut self, i: &'a[u8]) -> ParseResult<'a> {
        use ParseState::*;
        loop {
            match &self.state {
                ParsingHeader => {
                    let header = header_parser(i)?;
                    self.header = *header;
                    if self.header.path_sz as usize > MAX_PATH_SZ { 
                        return ParseResult::Err(ParserError::BadPathSz); 
                    }
                    if self.header.payload_sz as usize > MAX_PAYLOAD_SZ { 
                        return ParseResult::Err(ParserError::BadPayloadSz); 
                    }

                    self.state = ParsingPath;
                    self.pos = self.pos + HEADER_SZ;
                }
                ParsingPath => {
                    if i.len() - self.pos < self.header.path_sz as usize {
                        return Err(ParserError::NeedMoreData);
                    }
                    self.path = self.pos .. self.pos + self.header.path_sz as usize;

                    self.state = ParsingValue;
                    self.pos += self.header.path_sz as usize;
                }
                ParsingValue => {
                    if i.len() - self.pos < self.header.payload_sz as usize {
                        return Err(ParserError::NeedMoreData);
                    }
                    self.payload = self.pos .. self.pos + self.header.payload_sz as usize;

                    self.state = ParsingDone;
                    
                    let code = AnswerCode::try_from(self.header.code).map_err(|_| ParserError::BadCode)?;
                    let path_str: & str = unsafe { core::str::from_utf8_unchecked(&i[self.path.clone()]) };
                    let value: Value = value_parser(& i[self.pos .. self.pos + self.header.payload_sz as usize],
                        self.header.payload_ty)?;
                    
                    self.reset();
                    return Ok(Msg(code, path_str, value));
                }
                ParsingDone => unreachable!("Msg parser used after ParsingDone"),
            }
        }
    }
}


#[inline(always)]
fn header_parser(i: &[u8]) -> Result<&Header, ParserError> {
    if i.len() >= HEADER_SZ {
        let header : &Header = unsafe { &*(i.as_ptr() as *const _)};
        Ok(header)
    } else {
        Err(ParserError::BadHeader)
    }
}

#[inline(always)]
fn value_parser<'a>(payload: &'a[u8], ty_id: u8) -> Result<Value<'a>, ParserError> {
    use crate::ty::TypeTag::*;

    let ty_id = TypeTag::try_from(ty_id).map_err(|_| ParserError::BadTypeID)?;

    match ty_id {
        UNIT => {
            Ok(Value::UNIT(()))
        }
        BOOL => {
            Ok(Value::BOOL({
                if payload[0] == 0 { false } else { true }
            }))
        }
        I32 => {
            Ok(Value::I32({
                unsafe { *(payload.as_ptr() as *const _)}
            }))
        }
        I16 => {
            Ok(Value::I16({
                unsafe { *(payload.as_ptr() as *const _)}
            }))
        }
        I8 => {
            Ok(Value::I8({
                unsafe { *(payload.as_ptr() as *const _)}
            }))
        }
        U32 => {
            Ok(Value::U32({
                unsafe { *(payload.as_ptr() as *const _)}
            }))
        }
        U16 => {
            Ok(Value::U16({
                unsafe { *(payload.as_ptr() as *const _)}
            }))
        }
        U8 => {
            Ok(Value::U8({
                unsafe { *(payload.as_ptr() as *const _)}
            }))
        }
        STR => {
            Ok(Value::STR({
                unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(payload.as_ptr(), payload.len())) }
            }))
        }
        BYTES => {
            Ok(Value::BYTES({
                unsafe { core::slice::from_raw_parts(payload.as_ptr(), payload.len()) }
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryInto;
    
    #[test]
    fn parse_req_short() {
        let mut buf = [0x0u8;MAX_MSG_SZ];
        let msg_orig = Msg(
            AnswerCode::OK_WRITE,
            "/test/something",
            Value::STR("wofwofwof")
        );
        let req_sz = RequestBuilder::new(&mut buf)
            .code(msg_orig.0.try_into().unwrap())
            .path(msg_orig.1)
            .payload(msg_orig.2)
            .build()
            .unwrap();
        
        let mut parser = ParseMsg::new();
        let msg = parser.try_parse(&buf[.. req_sz]).unwrap();

        assert_eq!(msg, msg_orig)
    }

    #[test]
    fn parse_req_long() {
        let mut buf = [0x0u8;MAX_MSG_SZ];
        let msg_orig = Msg(
            AnswerCode::OK_WRITE,
            "/test/something/somethingsomethingrlylong/приветкакдела",
            Value::STR("/test/something/somethingsomethingrlylong/приветкакдела")
        );
        let _req_sz = RequestBuilder::new(&mut buf)
            .code(msg_orig.0.try_into().unwrap())
            .path(msg_orig.1)
            .payload(msg_orig.2)
            .build()
            .unwrap();
        
        let mut parser = ParseMsg::new();
        
        let mut pos = 0x40;
        let mut chunk = &buf[0 .. 0x40];
        let msg;
        loop {
            match parser.try_parse(chunk) {
                Ok(r) => {
                    msg = Some(r);
                    break;
                }
                Err(ParserError::NeedMoreData) => { 
                    println!("NeedMoreData: pos: {}", pos);
                    chunk = &buf[0 .. pos];
                    pos += 0x40;
                }
                Err(e) => panic!("{:?}", e),
            };
        }

        println!("Msg: {:?}", &msg.unwrap());
        
        assert_eq!(msg.unwrap(), msg_orig)
    }

}


