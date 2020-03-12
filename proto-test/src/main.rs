
pub mod protocol {
    pub use ellocopo2::*;
    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}

use protocol::*;
use std::convert::TryInto;

fn main() {
    let mut buf = [0x00u8;MAX_MSG_SZ];
    let request_sz = RequestBuilder::new(&mut buf)
        .code(RequestCode::READ)
        .path("/ctrl/record")
        .build().unwrap();

    let mut parser = ParseMsg::new();

    let Msg(code, path, val) = parser.try_parse(&buf[.. request_sz]).unwrap();
    
    let msg = req2msg(code.try_into().unwrap(), path, val).unwrap();

    println!("msg: {:?}", msg);

    let request_sz = RequestBuilder::new(&mut buf)
        .code(RequestCode::WRITE)
        .path("/ctrl/record")
        .payload(Value::BOOL(true))
        .build().unwrap();

    let mut parser = ParseMsg::new();

    let Msg(code, path, val) = parser.try_parse(&buf[.. request_sz]).unwrap();
    
    let msg = req2msg(code.try_into().unwrap(), path, val).unwrap();

    println!("msg: {:?}", msg);
}
