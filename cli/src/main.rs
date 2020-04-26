
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
    
    let mut buf = [0x0u8;MAX_MSG_SZ];

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let res = interpret(&line, &mut buf);
        println!("{:?}", res);
        if let Err(_) = res {
            continue;
        }
        let sz = res.unwrap();

        println!("{:x?}", &buf[..sz]);
        println!("{:?}", write_cmd(&usb_e.dh, &buf[..sz]));
        let sz = read_cmd(&usb_e.dh, &mut buf).unwrap();
        println!("{:x?}", &buf[..sz]);
        let mut parser = ParseMsg::new();
        let msg = parser.try_parse(&buf[..sz]).unwrap();
        println!("msg: {:?}", &msg);
    }
}

fn interpret(i: &str, buf: &mut [u8]) -> Result<usize, String> {
    let args: Vec<&str> =  { 
        if let Some(idx) = i.find(' ') {
            let parts = i.split_at(idx);
            vec![parts.0, parts.1]
        }
        else {
            vec![i]
        }
    };
    println!("args: {:?}", &args);
    
    let path = args[0];

    let (code, path) = if path.starts_with("@") 
               || path.starts_with("w") 
               || path.starts_with("W") {
        (RequestCode::WRITE, &path[1..])
    } else {
        (RequestCode::READ, path)
    };
    
    let mut tmp_buf = Vec::new();
    let val = if args.len() == 1 {
        Value::UNIT(())
    } else {
        parse_value(args[1], &mut tmp_buf).map_err(|e| format!("{}", e))?
    };

    println!("{:?}::{}::{:?}", code, &path, &val);

    let sz = RequestBuilder::new(buf)
        .code(code)
        .path(path)
        .payload(val)
        .build()
        .unwrap();

    Ok(sz)
}

use proc_macro2::Span;
use syn::{LitInt, LitStr, LitBool};

fn parse_value<'a>(i: &str, tmp_buf: &'a mut Vec<u8>) -> Result<Value<'a>, syn::Error> {

    println!("parse_value: {}", i);
    
    let val = if let Ok(res) = syn::parse_str::<LitInt>(i) {
        match res.suffix() {
            "u8"  => { Value::U8(res.base10_parse::<u8>()?) },
            "u16" => { Value::U16(res.base10_parse::<u16>()?) },
            "u32" => { Value::U32(res.base10_parse::<u32>()?) },
            "i32" | "" => { Value::I32(res.base10_parse::<i32>()?) },
            _ => todo!(),
        }
    } 
    else if let Ok(res) = syn::parse_str::<LitBool>(i) {
        Value::BOOL(res.value)
    } 
    else if let Ok(res) = syn::parse_str::<LitStr>(i) {
        *tmp_buf = res.value().as_bytes().to_vec();
        Value::STR(std::str::from_utf8(&tmp_buf[..]).unwrap())
    } 
    else {
        return Err(syn::Error::new(Span::call_site(), "else clause"));
    };

    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_init() -> LibUsbEntity<'static> {
        let c : &'static libusb::Context = 
            Box::leak(Box::new( libusb::Context::new().expect("Can not obtain libusb context")));
        let mut usb_e = usb_util::open_device(&c).expect("Device not found");
        // Reset
        usb_e.dh.reset().unwrap();

        println!("Device connected!");
        println!("Device desc: {}\n", usb_util::string_desc(&usb_e.dh));

        usb_e
    }

    #[test]
    fn vis_test() {
        let dev = fixture_init();
        let dh = dev.dh;
        
        // VIS ON
        let mut buf = [0x00u8;MAX_MSG_SZ];
        let sz = RequestBuilder::new(&mut buf)
            .code(RequestCode::WRITE)
            .path("/ctrl/vis")
            .payload(Value::BOOL(true))
            .build()
            .unwrap();
        println!("{:?}", write_cmd(&dh, &buf[..sz]));
        let sz = read_cmd(&dh, &mut buf).unwrap();
        println!("{:x?}", &buf[..sz]);
        let mut parser = ParseMsg::new();
        let msg = parser.try_parse(&buf[..sz]).unwrap();
        println!("msg: {:?}", &msg);

        // READ VIS
        loop {
            let mut buf = [0x0u8; 0x40];
            let sz = read_vis(&dh, &mut buf).unwrap();
            println!("{:x?}", &buf[..sz]);
        }
    }

    #[test]
    fn data_read_test() {
        let dev = fixture_init();
        let dh = dev.dh;
        
        // SETUP TRANS
        let mut buf = [0x00u8;MAX_MSG_SZ];
        let sz = RequestBuilder::new(&mut buf)
            .code(RequestCode::WRITE)
            .path("/io/file/pos")
            .payload(Value::U32(0))
            .build()
            .unwrap();
        println!("{:?}", write_cmd(&dh, &buf[..sz]));
        let sz = read_cmd(&dh, &mut buf).unwrap();
        println!("{:x?}", &buf[..sz]);
        let mut parser = ParseMsg::new();
        let msg = parser.try_parse(&buf[..sz]).unwrap();
        println!("msg: {:?}", &msg);
        
        const BLOCK_CNT: u32 = 1000;
        let mut buf = [0x00u8;MAX_MSG_SZ];
        let sz = RequestBuilder::new(&mut buf)
            .code(RequestCode::WRITE)
            .path("/io/file/len")
            .payload(Value::U32(BLOCK_CNT))
            .build()
            .unwrap();
        println!("{:?}", write_cmd(&dh, &buf[..sz]));
        let sz = read_cmd(&dh, &mut buf).unwrap();
        println!("{:x?}", &buf[..sz]);
        let mut parser = ParseMsg::new();
        let msg = parser.try_parse(&buf[..sz]).unwrap();
        println!("msg: {:?}", &msg);

        let mut buf = [0x00u8;MAX_MSG_SZ];
        let sz = RequestBuilder::new(&mut buf)
            .code(RequestCode::WRITE)
            .path("/io/file/start")
            .payload(Value::UNIT(()))
            .build()
            .unwrap();
        println!("{:?}", write_cmd(&dh, &buf[..sz]));
        let sz = read_cmd(&dh, &mut buf).unwrap();
        println!("{:x?}", &buf[..sz]);
        let mut parser = ParseMsg::new();
        let msg = parser.try_parse(&buf[..sz]).unwrap();
        println!("msg: {:?}", &msg);
        
        //std::thread::sleep(std::time::Duration::from_millis(1000));
        // DATA READ
        let mut total_bytes = BLOCK_CNT * 0x800;
        while total_bytes != 0 {
            let mut buf = [0x0u8; 0x800];
            let sz = read_data(&dh, &mut buf).unwrap();
            if sz == 0x800 {
                let header = unsafe { &*(buf.as_ptr() as *const [u32;2]) };
                print!("{:x}\t", header[1]);
                //print!("{}\t", sz);
                //print!("{:x?}", &buf[.. sz]);
            }
            else {
                panic!("sz: {}", sz);
            }

            total_bytes -= sz as u32;
            //std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    #[test]
    #[ignore]
    fn parse_value_test() {
        let mut tmp_buf = Vec::new();

        let _ = parse_value("t", &mut tmp_buf);
        let _ = parse_value("0b00111u8", &mut tmp_buf);
        let _ = parse_value("true", &mut tmp_buf);
        let _ = parse_value("\"test\"", &mut tmp_buf);
        let _ = parse_value("[0,1,2]", &mut tmp_buf);
    }
    
    #[test]
    #[ignore]
    fn wrong_cmd_seq() {
        let mut usb_e = fixture_init();

        let mut buf = [0x00u8;MAX_MSG_SZ];
        let sz = RequestBuilder::new(&mut buf)
            .code(RequestCode::READ)
            .path("/survey/surname")
            .build()
            .unwrap();


        for _ in 0 .. 4 {
            println!("{:?}", write_cmd(&usb_e.dh, &buf[..sz]));
            let mut buf = [0x00u8;0x40];
            let sz = read_cmd(&usb_e.dh, &mut buf).unwrap();
            print!("Read bytes: {:x?}", &buf[..sz]);
        }
        let _ = usb_e.dh.reset();
    }

    #[test]
    #[ignore]
    fn cmds() {
        let c = libusb::Context::new().expect("Can not obtain libusb context");
        let usb_e = usb_util::open_device(&c).expect("Device not found");
        println!("Device connected!");
        println!("Device desc: {}\n", usb_util::string_desc(&usb_e.dh));

        //let mut buf = [0x00u8;MAX_MSG_SZ];
        //let sz = RequestBuilder::new(&mut buf)
        //    .code(RequestCode::READ)
        //    .path("/survey/name")
        //    .build()
        //    .unwrap();
        //println!("{:?}", write_cmd(&usb_e.dh, &buf[..sz]));
        //let sz = read_cmd(&usb_e.dh, &mut buf).unwrap();
        //println!("{:x?}", &buf[..sz]);
        //let mut parser = ParseMsg::new();
        //let msg = parser.try_parse(&buf[..sz]).unwrap();
        //println!("msg: {:?}", &msg);

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
        if let Value::STR(str) = msg.2 {
            println!("\n{}", str);
        }

        //let surname = "\
        //        To be, or not to be, that is the question:
        //    Whether 'tis nobler in the mind to suffer
        //    The slings and arrows of outrageous fortune,
        //    Or to take Arms against a Sea of troubles,
        //    And by opposing end them: to die, to sleep;
        //";
        //println!("surname len: {}", surname.as_bytes().len());
        //let mut buf = [0x00u8;MAX_MSG_SZ];
        //let sz = RequestBuilder::new(&mut buf)
        //    .code(RequestCode::WRITE)
        //    .path("/survey/surname")
        //    .payload(Value::STR(surname))
        //    .build()
        //    .unwrap();
        //println!("{:x?}", &buf[..sz]);
        //println!("{:?}", write_cmd(&usb_e.dh, &buf[..sz]));
        //let sz = read_cmd(&usb_e.dh, &mut buf).unwrap();
        //println!("{:x?}", &buf[..sz]);
        //let mut parser = ParseMsg::new();
        //let msg = parser.try_parse(&buf[..sz]).unwrap();
        //println!("msg: {:?}", &msg);
    
    }
}

