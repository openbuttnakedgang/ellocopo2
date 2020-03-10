#![allow(non_camel_case_types)]

use core::mem::size_of;
use core::convert::{From, TryFrom};

use num_enum::TryFromPrimitive;

use crate::ty::{Value, TypeTag};

pub const MAX_MSG_SZ: usize = 512;
pub const HEADER_SZ: usize = size_of::<Header>();
pub const MAX_PATH_SZ: usize = MAX_MSG_SZ / 2 - HEADER_SZ;
pub const MAX_PAYLOAD_SZ: usize = MAX_MSG_SZ / 2;

// Signature and protocol version
pub const SIGN: u8 = 0x8E;

// REQUEST:
//  Client -> Server
// |  SIGN/PROTO_VER | PATH_SZ | PAYLOAD_SZ | REQ_CODE  | PAYLOAD_TY  |     PATH      |   PAYLOAD       |
// |:---------------:|:-------:|:----------:|:---------:|:-----------:|:-------------:|:---------------:|
// |     1 байт      | 1 байт  |   1 байт   |  1 байт   |    1 байт   |  PATH_SZ байт | PAYLOAD_SZ байт |
// |  <------------------ HEADER -------------------------------->    |                                 |
//


// ANSWER:
//  Server -> Client
// |  SIGN/PROTO_VER | PATH_SZ | PAYLOAD_SZ | ANS_CODE  | PAYLOAD_TY  |     PATH      |   PAYLOAD       |
// |:---------------:|:-------:|:----------:|:---------:|:-----------:|:-------------:|:---------------:|
// |     1 байт      | 1 байт  |   1 байт   |  1 байт   |    1 байт   |  PATH_SZ байт | PAYLOAD_SZ байт |
// |  <------------------ HEADER -------------------------------->    |                                 |
//


#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum RequestCode {
    READ = 0,
    WRITE = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum AnswerCode {
    OK_READ = 0,
    OK_WRITE = 1,
    ERR_LOCK = 2,
    ERR_BAD_PROTO = 3,
    ERR_BAD_FORMAT = 4,
    ERR_PATH = 5,
    ERR_TYPE = 6,
    ERR_USER = 7,
}

#[repr(packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Header {
    pub sign: u8,
    pub path_sz: u8,
    pub payload_sz: u8,
    pub code: u8,
    pub payload_ty: u8,
}

#[derive(Default)]
pub struct RequestBuilder<'a> {
    buf: &'a mut [u8],
    path: Option<&'a str>,
    payload: &'a [u8],
    req_code: Option<RequestCode>,
    payload_ty: TypeTag,
}

impl <'a> RequestBuilder<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buf,
            ..
            core::default::Default::default()
        }
    }

    pub fn path(&mut self, path: &'a str) -> &mut Self {
        self.path = Some(path);
        self
    }

    pub fn payload(&mut self, value: Value<'a>) -> &mut Self {
        self.payload = (&value).into();
        self.payload_ty = (&value).into();
        self
    }

    pub fn code(&mut self, code : RequestCode) -> &mut Self {
        self.req_code = Some(code);
        self
    }

    pub fn build(&mut self) -> Result<usize, &'static str> {
        {
            let header : &mut Header = unsafe { &mut*(self.buf.as_mut_ptr() as *mut _)};
            header.sign = SIGN;
            header.payload_sz = self.payload.len() as u8;
            header.path_sz = self.path.ok_or("No path")?.len() as u8;
            header.code = self.req_code.ok_or("No req code")? as u8;
            header.payload_ty = self.payload_ty as u8;
        }

        let header : &Header = unsafe { &*(self.buf.as_ptr() as *const _)};
        let header_sz = core::mem::size_of::<Header>();

        let path_end_pos = header_sz + (header.path_sz as usize);
        let payload_end_pos = path_end_pos + (header.payload_sz as usize);

        let path_dst = &mut self.buf[header_sz .. path_end_pos];
        path_dst.copy_from_slice(self.path.ok_or("No path")?.as_bytes());

        let payload_dst = &mut self.buf[path_end_pos .. payload_end_pos];
        payload_dst.copy_from_slice(self.payload);

        Ok(payload_end_pos)
    }
}

#[derive(Default)]
pub struct AnswerBuilder<'a> {
    buf: &'a mut [u8],
    ans_code: Option<AnswerCode>,
    payload: &'a [u8],
    payload_ty: TypeTag,
}

impl <'a> AnswerBuilder<'a> {

    pub fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buf,
            ..
            core::default::Default::default()
        }
    }

    pub fn code(&mut self, code : AnswerCode) -> &mut Self {
        self.ans_code = Some(code);
        self
    }

    pub fn payload(&mut self, value: Value<'a>) -> &mut Self {
        self.payload = (&value).into();
        self.payload_ty = (&value).into();
        self
    }

    pub fn build(&mut self) -> usize {

        let header : &mut Header = unsafe { &mut*(self.buf.as_mut_ptr() as *mut _)};
        let header_sz = core::mem::size_of::<Header>();

        // Update answer code if needed
        if let Some(code) = self.ans_code {
            header.code = code as u8;
        }

        if let TypeTag::UNIT = self.payload_ty {
            header.payload_ty = TypeTag::UNIT as u8;
            header.payload_sz = 0;
            header_sz + header.path_sz as usize
        } else {
            header.payload_ty = self.payload_ty as u8;
            header.payload_sz = self.payload.len() as u8;

            let path_end_pos = header_sz + (header.path_sz as usize);
            let payload_end_pos = path_end_pos + (header.payload_sz as usize);

            let payload_dst = &mut self.buf[path_end_pos .. payload_end_pos];
            payload_dst.copy_from_slice(self.payload);

            payload_end_pos
        }
    }
}

impl From<RequestCode> for AnswerCode {
    fn from(c: RequestCode) -> Self {
        use RequestCode::*;
        match c {
            READ => AnswerCode::OK_READ,
            WRITE => AnswerCode::OK_WRITE,
        }
    }
}

impl TryFrom<AnswerCode> for RequestCode {
    type Error = ();
    fn try_from(c: AnswerCode) -> Result<Self, Self::Error> {
        use AnswerCode::*;
        match c {
            OK_READ => Ok(RequestCode::READ),
            OK_WRITE => Ok(RequestCode::WRITE),
            _ => Err(())
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use core::str::from_utf8_unchecked;

    #[test]
    fn test_request() {
        let mut buf = [0u8; 0x40];

        // op = read, value = None
        let request_sz = RequestBuilder::new(&mut buf)
            .code(RequestCode::READ)
            .path("path/name")
            .build().unwrap();

        let req_slice = &buf[..request_sz];

        assert_eq!(
            req_slice,
            [
                SIGN,
                0x09,
                0x00,
                RequestCode::READ as u8,
                TypeTag::UNIT as u8,
                b'p',
                b'a',
                b't',
                b'h',
                b'/',
                b'n',
                b'a',
                b'm',
                b'e'
            ]
        );

        // op = read, value = u32
        let request_sz = RequestBuilder::new(&mut buf)
            .code(RequestCode::READ)
            .path("path/name")
            .payload(Value::U32(0xDEAD_BEAF))
            .build().unwrap();

        let req_slice = &buf[0..request_sz];

        assert_eq!(
            req_slice,
            [
                SIGN,
                0x09,
                0x04,
                RequestCode::READ as u8,
                TypeTag::U32 as u8,
                b'p',
                b'a',
                b't',
                b'h',
                b'/',
                b'n',
                b'a',
                b'm',
                b'e',
                0xAF,
                0xBE,
                0xAD,
                0xDE
            ]
        );

        // op = write, value = str
        let var_len_value = [0xAF, 0xBE, 0xAD, 0xDE, 0xDE, 0xAD, 0xCE, 0x11];
        let request_sz = RequestBuilder::new(&mut buf)
            .code(RequestCode::WRITE)
            .path("path")
            .payload(Value::STR(unsafe {
                from_utf8_unchecked(&var_len_value)
            }))
            .build().unwrap();

        let req_slice = &buf[..request_sz];

        assert_eq!(
            req_slice,
            [
                SIGN,
                0x04,
                0x08,
                RequestCode::WRITE as u8,
                TypeTag::STR as u8,
                b'p',
                b'a',
                b't',
                b'h',
                0xAF,
                0xBE,
                0xAD,
                0xDE,
                0xDE,
                0xAD,
                0xCE,
                0x11
            ]
        );
    }

    #[test]
    fn test_answer() {
        let mut buf = [0u8; 0x40];

        // op = read, value = u8
        let _ = RequestBuilder::new(&mut buf)
            .code(RequestCode::READ)
            .path("path")
            .build();
        let ans_sz = AnswerBuilder::new(&mut buf)
            .payload(Value::U8(0xAD))
            .build();
        let ans_slice = &buf[0..ans_sz];
        assert_eq!(
            ans_slice,
            [
                SIGN,
                0x04,
                0x01,
                AnswerCode::OK_READ as u8,
                TypeTag::U8 as u8,
                b'p',
                b'a',
                b't',
                b'h',
                0xAD
            ]
        );

        // op = write, error
        let var_len_value = [0x22, 0xCE, 0x11];
        let req_sz = RequestBuilder::new(&mut buf)
            .code(RequestCode::WRITE)
            .path("boooom/baaaaaam")
            .payload(Value::STR(unsafe {
                from_utf8_unchecked(&var_len_value)
            }))
            .build().unwrap();

        let req_slice = &buf[..req_sz];
        assert_eq!(
            req_slice,
            [
                SIGN,
                0x0f,
                0x03,
                AnswerCode::OK_WRITE as u8,
                TypeTag::STR as u8,
                b'b',
                b'o',
                b'o',
                b'o',
                b'o',
                b'm',
                b'/',
                b'b',
                b'a',
                b'a',
                b'a',
                b'a',
                b'a',
                b'a',
                b'm',
                0x22,
                0xCE,
                0x11,
            ]
        );

        let ans_sz = AnswerBuilder::new(&mut buf)
            .code(AnswerCode::ERR_LOCK)
            .build();
        let ans_slice = &buf[0..ans_sz];
        assert_eq!(
            ans_slice,
            [
                SIGN,
                0xf,
                0x0,
                AnswerCode::ERR_LOCK as u8,
                TypeTag::UNIT as u8,
                b'b',
                b'o',
                b'o',
                b'o',
                b'o',
                b'm',
                b'/',
                b'b',
                b'a',
                b'a',
                b'a',
                b'a',
                b'a',
                b'a',
                b'm',
            ]
        );
    }

    #[test]
    fn test_unit() {
        let mut buf = [0u8; 0x40];

        // op = read, value = u8
        let _ = RequestBuilder::new(&mut buf)
            .code(RequestCode::WRITE)
            .path("path")
            .payload(Value::UNIT)
            .build();

        let ans_sz = AnswerBuilder::new(&mut buf)
            .payload(Value::UNIT)
            .build();
        let ans_slice = &buf[0..ans_sz];

        assert_eq!(
            ans_slice,
            [
                SIGN,
                0x04,
                0x00,
                AnswerCode::OK_WRITE as u8,
                TypeTag::UNIT as u8,
                b'p',
                b'a',
                b't',
                b'h',
            ]
        );
    }
}




