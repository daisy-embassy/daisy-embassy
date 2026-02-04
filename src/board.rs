use crate::flash::FlashBuilder;
use crate::led::UserLed;
use crate::pins::*;
use crate::usb::UsbPeripherals;
use crate::{audio::AudioPeripherals, sdram::SdRamBuilder};
pub struct DaisyBoard<'a> {
    pub pins: DaisyPins<'a>,
    pub user_led: UserLed<'a>,
    pub audio_peripherals: AudioPeripherals<'a>,
    pub flash: FlashBuilder<'a>,
    pub sdram: SdRamBuilder<'a>,
    pub usb_peripherals: UsbPeripherals<'a>,
    // on board "BOOT" button.
    pub boot: Boot<'a>,
}
