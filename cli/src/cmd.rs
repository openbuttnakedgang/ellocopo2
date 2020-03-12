
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, multispace0, multispace1};
use nom::multi::many1;
use nom::error::ErrorKind;
use nom::{Err, IResult, error::ParseError};
use nom::combinator::opt;
use nom::combinator::map;
use nom::sequence::preceded;

use std::convert::From;

use ellocopo::PROTOCOL_SIGN;
use ellocopo::OperationStatus;
use ellocopo::MsgHeader;
use ellocopo::RegTypeId;

#[derive(Debug, Clone)]
pub enum Cmd {
    Read(String),
    Write(String, RegTypeId, Vec<u8>),
    Erase(String),
}

#[derive(Debug, Clone, Copy)]
pub enum StrParseError {
    BadPath,
    BadType,
    NoValue,
    ValueNotFit,
    BadValue,
    RubbishEnd,
    Other(ErrorKind)
}

impl<'a> ParseError<&'a str> for StrParseError {
  fn from_error_kind(_input: &'a str, kind: ErrorKind) -> Self {
      Self::Other(kind)
  }

  fn append(_input: &'a str, _kind: ErrorKind, other: Self) -> Self {
     other
  }
}

pub fn cmd_from_str_parser(i: &str) -> IResult<&str, Cmd, StrParseError> {
    let (i, _) = multispace0(i)?;
    let (i, path) = match many1(alt((alphanumeric1, tag("."), tag("/"))))(i) {
        Ok(r) => r,
        Err(e) => { 
            let _e : nom::Err::<(&str,ErrorKind)> = e; // Господи спаси
            IResult::Err(Err::Failure(StrParseError::BadPath))?
        }
    };

    let path = path.iter().fold(String::new(), |mut acc, x| {
        match *x {
            "/" => acc.push_str("."),
            _   => acc.push_str(x),
        }
        acc
    });
    
    let typeid_or_erase_parser = alt((tag("u8"), tag("u16"), tag("u32"), tag("i8"), tag("i16"), tag("i32"), tag("()"), tag("[u8]"), tag("bytes"), tag("str"), tag("erase")));
    let typeid_or_erase_parser = opt(preceded(multispace1, typeid_or_erase_parser));

    let (i, type_id_or_erase) = typeid_or_erase_parser(i)?;

    match type_id_or_erase {
        Some("erase") => { //erase op
            Ok((i, Cmd::Erase(path.to_string())))
        }
        Some(type_id_str) => { // write op
            let type_id = match type_id_str {
                "()"  => RegTypeId::UNIT,
                "u8"  => RegTypeId::U8,
                "u16" => RegTypeId::U16,
                "u32" => RegTypeId::U32,
                "i8"  => RegTypeId::I8,
                "i16" => RegTypeId::I16,
                "i32" => RegTypeId::I32,
                "[u8]" | "bytes" => RegTypeId::BYTES,
                "str" => RegTypeId::STR,
                _ =>  unreachable!()
                
            };

            let (i, _) = multispace1(i)?;

            let (_, value) = parse_value_from_str(i, type_id)?;
            
            Ok((i, Cmd::Write(path.to_string(), type_id, value)))

        }
        None => { // read op
            let (i, _) = multispace0(i)?;
            if i.len() != 0 {
                Err(Err::Failure(StrParseError::RubbishEnd))?
            }
            Ok((i, Cmd::Read(path.to_string())))
        }
    }
}

fn parse_value_from_str(i: &str, type_id : RegTypeId, ) -> IResult<&str, Vec<u8>, StrParseError> {

    if let RegTypeId::UNIT = type_id {
        return Ok((i, Vec::new()))
    }
    #[derive(Copy,Clone)]
    enum Radix { Hex = 16, Dec = 10, Bin = 2};

    let (i, minus) = opt(tag("-"))(i)?;
    let (i, radix) = map(alt((tag("0x"), tag("0b"), tag(""))), |s: &str| {
        match s {
            "0x" => Radix::Hex,
            "0b" => Radix::Bin,
            ""   => Radix::Dec,
            _ => unreachable!(),
        }
    })(i)?;

    macro_rules! typeid_match_arm {
        ($type:ident, $len:literal, $s:ident) => {{
            let value = match $type::from_str_radix($s, radix as u32) {
                Ok(value) => value,
                Err(_) => return Err(StrParseError::ValueNotFit),
            };
            let value_bytes : [u8;$len] = unsafe {transmute(value)};
            Ok(Vec::from(&value_bytes[..]))            
        }};
    }

    // TODO: and digital delimeter _
    let (_, value_bytes) = map(alphanumeric1, |s : &str| {
        use RegTypeId::*;
        use std::mem::transmute;
        match type_id {
            I8  => typeid_match_arm!(i8, 1, s),
            U8  => typeid_match_arm!(u8, 1, s),
            I16 => typeid_match_arm!(i16, 2, s),
            U16 => typeid_match_arm!(u16, 2, s),
            I32 => typeid_match_arm!(i32, 4, s),
            U32 => typeid_match_arm!(u32, 4, s),

            BYTES => {
                if s.len() % 2 != 0 { return Err(StrParseError::BadValue) }
                let mut value = Vec::new();
                value.push((s.len() / 2) as u8);

                for i in (0 .. s.len()).step_by(2) {
                    match u8::from_str_radix(&s[i .. i + 2], Radix::Hex as u32) {
                        Ok(digit) => value.push(digit),
                        Err(_) => return Err(StrParseError::ValueNotFit),
                    };
                }
                Ok(value)                     
            }

            STR => {
                let mut value = Vec::new();
                value.push(s.len() as u8);
                value.extend_from_slice(s.as_bytes());
                Ok(value)                     
            }

            _ => unimplemented!()
        }
    })(i)?;
    
    value_bytes.map_or_else( |e| Err(Err::Failure(e)), |v| Ok((i, v)) )
}


pub fn write(path: String, type_id: RegTypeId, value: Vec<u8>) -> Vec<u8> {
    let mut cmd = Vec::with_capacity(0x40);
    cmd.push(PROTOCOL_SIGN);
    cmd.push(OperationStatus::Write as u8);
    cmd.push(path.len() as u8);
    cmd.extend_from_slice(path.as_bytes());
    cmd.push(type_id as u8);
    cmd.extend_from_slice(&value);

    cmd
}

pub fn read(path: String) -> Vec<u8> {
    let mut cmd = Vec::with_capacity(0x40);
    cmd.push(PROTOCOL_SIGN);
    cmd.push(OperationStatus::Read as u8);
    cmd.push(path.len() as u8);
    cmd.extend_from_slice(path.as_bytes());

    cmd
}

pub fn erase(path : String) -> Vec<u8> {
    let mut cmd = Vec::with_capacity(0x40);
    cmd.push(PROTOCOL_SIGN);
    cmd.push(OperationStatus::Erase as u8);
    cmd.push(path.len() as u8);
    cmd.push(0);
    cmd.extend_from_slice(path.as_bytes());

    cmd
}

pub fn cmd_bytes_to_str(i : &[u8]) -> IResult<&[u8], String> {
    use std::fmt::Write;

    if i.len() < std::mem::size_of::<MsgHeader>() {
        Err(Err::Error((i, ErrorKind::LengthValueFn)))?
    }

    let (i, _) = sign_parser(i)?;
    let (i, (op, is_error)) = op_parser(i)?;
    let (i, name_sz) = (&i[1..], i[0] as usize);
    let (i, name) = name_parser(i, name_sz)?;

    match (is_error, i.len()) {
        (false, 0) => {
            let mut res = String::with_capacity(0x40);
            let _ = write!(&mut res, "{} {} No value", op, name);
            Ok((i, res))
        }
        (false, _) => {
            //TODO: fix value parser
            let (i, _type_id) = (&i[1..], i[0] as usize);
            let (i, value_sz) = (&i[1..], i[0] as usize);
            let (i, value) = value_parser(i, value_sz)?;
            let mut res = String::with_capacity(0x40);
            let _ = write!(&mut res, "{} {} Value : {}", op, name, value);
            Ok((i, res))
        }
        (true, _) => {
            let (i, _type_id) = (&i[1..], i[0] as usize);
            let (_, error) = error_parser(i[0])?;
            let mut res = String::with_capacity(0x40);
            let _ = write!(&mut res, "{} {} Error : {}", op, name, error);
            Ok((i, res))
        }
    }

}

pub fn sign_parser(i : &[u8]) -> IResult<&[u8], ()> {
    if i[0] == PROTOCOL_SIGN {
        Ok((&i[1..], ()))
    }
    else {
        Err(Err::Error((i, ErrorKind::Tag)))
    }
}

pub fn op_parser(i : &[u8]) -> IResult<&[u8], (String, bool)> {
    use std::convert::TryFrom;
    use OperationStatus::*;

    let byte_op : i8 = unsafe { std::mem::transmute(i[0]) };
    let op = match OperationStatus::try_from(byte_op) {
        Ok(op) => op,
        Err(_) => Err(Err::Error((i, ErrorKind::Tag)))?
    };

    let op = match op {
        Read  => (String::from("Read"), false),
        Write => (String::from("Write"), false),
        Erase => (String::from("Erase"), false),
        ReadFailed  => (String::from("Error : ReadFailed"), true),
        WriteFailed => (String::from("Error: WriteFailed"), true),
        EraseFailed => (String::from("Error: EraseFailed"), true),
        Unsupported => (String::from("Error: Unsupported"), true),
    };

    Ok((&i[1..], op))
} 

pub fn name_parser(i : &[u8], sz : usize) -> IResult<&[u8], String> {
    use std::str::from_utf8;
    if i.len() < sz {
        Err(Err::Error((i, ErrorKind::LengthValueFn)))?
    }

    let name = match from_utf8(&i[.. sz]) {
        Ok(name) => name,
        Err(_) => Err(Err::Error((i, ErrorKind::IsNot)))?,
    };

    Ok((&i[sz..], String::from(name)))
} 

pub fn value_parser(i : &[u8], sz : usize) -> IResult<&[u8], String> {
    use std::fmt::Write;
    use std::str::from_utf8_unchecked;

    if i.len() < sz {
        Err(Err::Error((i, ErrorKind::LengthValueFn)))?
    }

    let mut value = String::with_capacity(0x40);
    let _ = write!(&mut value, "{:X?}", &i[..sz]);
    let _ = write!(&mut value, "\n as str: {}", unsafe { from_utf8_unchecked(&i[..sz]) } );

    Ok((&i[sz..], value))
} 

pub fn error_parser<'a>(i : u8) -> IResult<&'a [u8], String> {
    use std::convert::TryFrom;
    // empty slice in return type for compatibility with other parsers
    let empty = &[0u8;0];

    let error = match ellocopo::Error::try_from(i) {
        Ok(e) => e,
        Err(_) => Err(Err::Error((&empty[..], ErrorKind::IsNot)))?,
    };

    use ellocopo::Error::*;
    let error = match error {
        OkAsync => "OkAsync, Should never see this error",
        BadProtocol => "BadProtocol",
        BadFormat => "BadFormat",
        WrongOperation => "WrongOperation",
        NoSuch => "NoSuch",
        BadParam => "BadParam",
        NonWriteable => "NonWriteable",
    };

    Ok((&empty[..], String::from(error)))
} 