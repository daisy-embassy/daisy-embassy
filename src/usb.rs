use embassy_stm32::{self as hal, Peri};
use hal::{peripherals::USB_OTG_FS, usb::Driver};

use crate::pins::USB2Pins;

pub type DaisyUsb = Driver<'static, USB_OTG_FS>;

pub struct UsbPeripherals<'a> {
    pub usb_otg_fs: Peri<'a, USB_OTG_FS>,
    pub pins: USB2Pins<'a>,
}
