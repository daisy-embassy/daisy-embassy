use embassy_stm32 as hal;
use hal::peripherals::*;

use defmt::unwrap;
use embassy_time::Timer;

/// A simple HAL for the Texas Instruments PCM3060 audio codec
pub struct Codec {}

// PCM3060 I2C constants
const I2C_CODEC_ADDRESS: u8 = 0x8c >> 1;

// PCM3060 register addresses
const SYS_CTRL_REGISTER: u8 = 0x40; // 64
const ADC_CTRL1_REGISTER: u8 = 0x48; // 72
const DAC_CTRL1_REGISTER: u8 = 0x43; // 67

// PCM3060 register masks
const MRST_MASK: u8 = 0x80;
const SRST_MASK: u8 = 0x40;
const ADC_PSV_MASK: u8 = 0x20;
const DAC_PSV_MASK: u8 = 0x10;
const FMT_MASK: u8 = 0x1;

impl Codec {
    pub async fn setup_pcm3060(i2c: &mut hal::i2c::I2c<'_, hal::mode::Blocking>) {
        // Reset codec
        Self::write_pcm3060_reg(i2c, SYS_CTRL_REGISTER, MRST_MASK, false).await;
        Self::write_pcm3060_reg(i2c, SYS_CTRL_REGISTER, SRST_MASK, false).await;

        // Set 24-bit Left-Justified format
        Self::write_pcm3060_reg(i2c, ADC_CTRL1_REGISTER, FMT_MASK, true).await;
        Self::write_pcm3060_reg(i2c, DAC_CTRL1_REGISTER, FMT_MASK, true).await;

        // Disable power saving
        Self::write_pcm3060_reg(i2c, SYS_CTRL_REGISTER, ADC_PSV_MASK, false).await;
        Self::write_pcm3060_reg(i2c, SYS_CTRL_REGISTER, DAC_PSV_MASK, false).await;
    }

    async fn write_pcm3060_reg(
        i2c: &mut hal::i2c::I2c<'_, hal::mode::Blocking>,
        register: u8,
        mask: u8,
        set: bool,
    ) {
        // Read current register value
        let mut buffer = [0];
        unwrap!(i2c.blocking_write_read(I2C_CODEC_ADDRESS, &[register], &mut buffer));

        // Modify value based on mask and set flag
        let value = if set {
            buffer[0] | mask
        } else {
            buffer[0] & !mask
        };

        // Write back modified value
        unwrap!(i2c.blocking_write(I2C_CODEC_ADDRESS, &[register, value]));

        Timer::after_micros(10).await;
    }
}

#[allow(non_snake_case)]
pub struct Pins {
    pub SCL: PB10,   // I2C2 SCL
    pub SDA: PB11,   // I2C2 SDA
    pub MCLK_A: PE2, // SAI1 MCLK_A
    pub SCK_A: PE5,  // SAI1 SCK_A
    pub FS_A: PE4,   // SAI1 FS_A
    pub SD_A: PE6,   // SAI1 SD_A
    pub SD_B: PE3,   // SAI1 SD_B
}
