use std::time::Duration;

use serialport::{Error, ErrorKind, Result, SerialPort, SerialPortInfo, SerialPortType};
use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};

use crate::common::MESSAGE_LENGTH;

const MANUFACTURER_0: &str = "wch.cn";
const MANUFACTURER_1: &str = "QinHeng Electronics";
const PRODUCT: &str = "CH340";

const BAUD_RATE: u32 = 9600;
const TIMEOUT_S: u64 = 1;

fn match_manufacturer_and_product(port_info: &SerialPortInfo) -> bool {
    if let SerialPortType::UsbPort(usb_port_info) = &port_info.port_type {
        let manufacturer_matched = usb_port_info.manufacturer.as_ref()
            .map(|man| man == MANUFACTURER_0 || man == MANUFACTURER_1)
            .unwrap_or(false);
        let product_matched = usb_port_info.product.as_ref()
            .map(|product| product.contains(PRODUCT))
            .unwrap_or(false);
        manufacturer_matched && product_matched
    } else {
        false
    }
}

pub fn connect() -> Result<Box<dyn SerialPort>> {
    let default_err = Err(Error::new(ErrorKind::NoDevice, "Cannot find power supply"));
    serialport::available_ports()?
        .into_iter()
        .filter(match_manufacturer_and_product)
        .map(|port_info| {
            serialport::new(&port_info.port_name, BAUD_RATE)
                .timeout(Duration::from_secs(TIMEOUT_S))
                .open()
        })
        .fold_while(default_err, |_, new| {
            if new.is_ok() {
                Done(new)
            } else {
                Continue(new)
            }
        })
        .into_inner()
}

pub fn read(port: &mut Box<dyn SerialPort>) -> Result<[u8; MESSAGE_LENGTH]> {
    let mut output = [0; MESSAGE_LENGTH];
    let _ = port.read(&mut output)?;

    Ok(output)
}
