
mod usb_util;
//mod cmd;

use std::io::{self, BufRead};
//use cmd::{cmd_from_str_parser, Cmd};

use usb_util::*;

fn main() {
    let c = libusb::Context::new().expect("Can not obtain libusb context");
    let usb_e = usb_util::open_device(&c).expect("Device not found");
    println!("Device connected!");
    println!("Device desc: {}\n", usb_util::string_desc(&usb_e.dh));

    let mut buf = [0x00u8;0x40];
    println!("{:?}", write_cmd(&usb_e.dh, &buf));
    let sz = read_cmd(&usb_e.dh, &mut buf).unwrap();
    println!("{:x?}", &buf[..sz]);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        println!("{}", &line.unwrap())
    }
}
