#![allow(dead_code)]

use libusb::*;
pub use std::sync::Arc;
use log::*;

/// Some public config definitions
const VID: u16 = 0x0483;
const PID: u16 = 0x7503;
const EP_VIS: u8 = 0x83;
const EP_OUT: u8 = 0x01;
const EP_IN: u8 = 0x81;
const EP_DATA_OUT: u8 = 0x02;
const EP_DATA_IN: u8 = 0x82;

const BULK_TRANSFER_TIMEOUT_MS: u64 = 500;
const DATA_TIMEOUT_MS: u64 = 300;

pub struct LibUsbEntity<'a> {
    pub d: Device<'a>,
    pub dd: libusb::DeviceDescriptor,
    pub dh: libusb::DeviceHandle<'a>,
}

pub fn open_device<'a>(
    context: &'a Context,
    //vid: u16,
    //pid: u16,
) -> Option<LibUsbEntity<'a>> {
    let vid = VID;
    let pid = PID;

    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => {
                    return Some(LibUsbEntity {
                        d: device,
                        dd: device_desc,
                        dh: handle,
                    });
                }
                Err(_) => continue,
            }
        }
    }

    None
}

pub fn string_desc(handle : &libusb::DeviceHandle) -> String {
    const TIMEOUT : u64 = 600;

    let ls = handle.read_languages(std::time::Duration::from_millis(TIMEOUT)).expect("Timeout reading dev lang");
    let sd1 = handle
        .read_string_descriptor(ls[0], 1, std::time::Duration::from_millis(TIMEOUT))
        .unwrap_or("FAILED".to_string());
    let sd2 = handle
        .read_string_descriptor(ls[0], 2, std::time::Duration::from_millis(TIMEOUT))
        .unwrap_or("FAILED".to_string());
        //.expect("Timeout reading dev string desc 2");
    let sd3 = handle
        .read_string_descriptor(ls[0], 3, std::time::Duration::from_millis(TIMEOUT))
        .unwrap_or("FAILED".to_string());
        //.expect("Timeout reading dev string desc 3");

    format!("{} {} {}", sd1, sd2, sd3)
}


pub fn read_vis(dh: &libusb::DeviceHandle, buf: &mut [u8]) -> Result<usize> {
    //trace!("vis buf size : {}", buf.len());
    match dh.read_bulk(
        EP_VIS,
        buf,
        std::time::Duration::from_millis(BULK_TRANSFER_TIMEOUT_MS),
    ) {
        Ok(r) => {
            trace!("read vis {:?} bytes", r);
            Ok(r)
        }
        Err(e) => {
            debug!("read vis error: {:?}", e);
            Err(e)
        }
    }
}

pub fn write_cmd(dh: &libusb::DeviceHandle, buf: &[u8]) -> std::result::Result<(usize), ()> {
    match dh.write_bulk(
        EP_OUT,
        buf,
        std::time::Duration::from_millis(BULK_TRANSFER_TIMEOUT_MS),
    ) {
        Ok(sz) => {
            trace!("write cmd {:?} bytes", sz);
            Ok(sz)
        }
        Err(e) => {
            debug!("write cmd error: {:?}", e);
            Err(())
        }
    }
}
pub fn read_cmd(dh: &libusb::DeviceHandle, buf: &mut [u8]) -> std::result::Result<usize, ()> {
    match dh.read_bulk(
        EP_IN,
        buf,
        std::time::Duration::from_millis(DATA_TIMEOUT_MS),
    ) {
        Ok(r) => {
            trace!("read cmd {:?} bytes", r);
            Ok(r)
        }
        Err(e) => {
            debug!("read cmd error: {:?}", e);
            Err(())
        }
    }
}

pub fn write_data(dh: &libusb::DeviceHandle, buf: &[u8]) -> Result<usize> {
    match dh.write_bulk(
        EP_DATA_OUT,
        buf,
        std::time::Duration::from_millis(DATA_TIMEOUT_MS),
    ) {
        Ok(r) => {
            trace!("write data {:?} bytes", r);
            Ok(r)
        }
        Err(e) => {
            debug!("write data error: {:?}", e);
            Err(e)
        }
    }
}
pub fn read_data(dh: &libusb::DeviceHandle, buf: &mut [u8]) -> Result<usize> {
    match dh.read_bulk(
        EP_DATA_IN,
        buf,
        std::time::Duration::from_millis(BULK_TRANSFER_TIMEOUT_MS),
    ) {
        Ok(r) => {
            trace!("read data {:?} bytes", r);
            Ok(r)
        }
        Err(e) => {
            debug!("read data error: {:?}", e);
            Err(e)
        }
    }
}
