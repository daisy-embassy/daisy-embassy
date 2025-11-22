use embassy_stm32::{self as hal, gpio::Output};
use hal::peripherals::*;

use embassy_time::Timer;

pub struct Codec {}

impl Codec {
    pub async fn setup_ak4556(mut reset: Output<'_>) {
        reset.set_high();
        Timer::after_millis(1).await;
        reset.set_low();
        Timer::after_millis(1).await;
        reset.set_high();
    }
}

#[allow(non_snake_case)]
pub struct Pins {
    pub MCLK_A: PE2, // SAI1 MCLK_A
    pub SCK_A: PE5,  // SAI1 SCK_A
    pub FS_A: PE4,   // SAI1 FS_A
    pub SD_A: PE6,   // SAI1 SD_A
    pub SD_B: PE3,   // SAI1 SD_B
    pub RESET: PB11,
}
