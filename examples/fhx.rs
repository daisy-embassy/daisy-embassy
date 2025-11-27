//! Example of using `https://github.com/daisy-embassy/fhx` to drive FHX-8CV or FHX-8GT Eurorack
//! module using a SPI interface.
//!
//! Since the SPI protocol used is strictly unidirectional, this example with work with any
//! configuration of FHX-8CV and/or FHX-8GT modules, including none, or a single one of either.

#![no_std]
#![no_main]
#![cfg(feature = "patch_sm")]

use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{Pll, PllDiv, PllMul, PllPreDiv, PllSource};
use embassy_stm32::time::mhz;
use embassy_stm32::{gpio, spi, Config};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL50,
        divp: None,
        divq: Some(PllDiv::DIV8), // SPI
        divr: None,
    });

    let p = embassy_stm32::init(config);
    info!("Hello World!");
    let daisy_p = new_daisy_board!(p);
    let mut led = daisy_p.user_led;

    let mut spi_config = spi::Config::default();
    spi_config.frequency = mhz(1);
    spi_config.miso_pull = gpio::Pull::Down; // unused, NC

    let spi = spi::Spi::new_txonly(
        p.SPI2,
        daisy_p.pins.d10,
        daisy_p.pins.d9,
        p.DMA2_CH4,
        spi_config,
    );

    #[link_section = ".sram1_bss"]
    static mut TX_BUFFER: [u8; 4] = [0; 4];

    let mut fhx = fhx::Fhx::new(
        spi,
        gpio::Output::new(daisy_p.pins.d1, gpio::Level::High, gpio::Speed::Low),
        gpio::Output::new(daisy_p.pins.a3, gpio::Level::Low, gpio::Speed::Low),
        gpio::Output::new(daisy_p.pins.a8, gpio::Level::Low, gpio::Speed::Low),
        gpio::Output::new(daisy_p.pins.a9, gpio::Level::Low, gpio::Speed::Low),
        unsafe { &mut TX_BUFFER },
    );

    let cv_addr = fhx::CvAddress::Cv1;
    let gt_addr = fhx::GtAddress::Gt0;

    fhx.set_cv_polarity(cv_addr, 0x00); // unipolar
    fhx.set_cv_raw(cv_addr, fhx::CvChannel::Channel1, 0).await;
    fhx.set_cv_raw(cv_addr, fhx::CvChannel::Channel2, 0x7FFF)
        .await;
    fhx.set_cv_raw(cv_addr, fhx::CvChannel::Channel3, 0xFFFF)
        .await;

    let mut cnt = 0;
    loop {
        info!("on");
        led.on();
        fhx.gate_high(gt_addr, fhx::GtChannel::Channel3).await;
        fhx.gate_low(gt_addr, fhx::GtChannel::Channel5).await;
        Timer::after_millis(300).await;

        info!("off");
        led.off();

        fhx.gate_low(gt_addr, fhx::GtChannel::Channel3).await;
        fhx.gate_high(gt_addr, fhx::GtChannel::Channel5).await;
        Timer::after_millis(300).await;

        fhx.set_cv_raw(cv_addr, fhx::CvChannel::Channel4, cnt).await;
        cnt = cnt.wrapping_add(1000);
    }
}
