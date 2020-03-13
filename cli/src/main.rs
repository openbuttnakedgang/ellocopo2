
mod usb_util;
//mod cmd;

use std::io::{self, BufRead};
//use cmd::{cmd_from_str_parser, Cmd};

use ellocopo2::*;

use usb_util::*;

fn main() {
    let c = libusb::Context::new().expect("Can not obtain libusb context");
    let usb_e = usb_util::open_device(&c).expect("Device not found");
    println!("Device connected!");
    println!("Device desc: {}\n", usb_util::string_desc(&usb_e.dh));

    let mut buf = [0x00u8;MAX_MSG_SZ];
    let sz = RequestBuilder::new(&mut buf)
        .code(RequestCode::READ)
        .path("/survey/name")
        .build()
        .unwrap();
    println!("{:?}", write_cmd(&usb_e.dh, &buf[..sz]));
    let sz = read_cmd(&usb_e.dh, &mut buf).unwrap();
    println!("{:x?}", &buf[..sz]);
    let mut parser = ParseMsg::new();
    let msg = parser.try_parse(&buf[..sz]).unwrap();
    println!("msg: {:?}", &msg);

    let mut buf = [0x00u8;MAX_MSG_SZ];
    let sz = RequestBuilder::new(&mut buf)
        .code(RequestCode::READ)
        .path("/survey/surname")
        .build()
        .unwrap();
    println!("{:?}", write_cmd(&usb_e.dh, &buf[..sz]));
    let sz = read_cmd(&usb_e.dh, &mut buf).unwrap();
    println!("{:x?}", &buf[..sz]);
    let mut parser = ParseMsg::new();
    let msg = parser.try_parse(&buf[..sz]).unwrap();
    println!("msg: {:?}", &msg);


    //let stdin = io::stdin();
    //for line in stdin.lock().lines() {
    //    println!("{}", &line.unwrap())
    //}
}
