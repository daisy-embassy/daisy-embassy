// controls the speed of the blinking onboard LED using a potentiometer
// Connect a potentiometer to pin ADC1/D16

#![no_std]
#![no_main]

use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{adc::Adc, Config};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config: Config = Default::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV8), // 100mhz
            divq: None,
            divr: None,
        });
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.adcsel = mux::Adcsel::PLL2_P;
    }
    let p = embassy_stm32::init(config);
    let mut daisy_p = new_daisy_board!(p);

    let mut led = daisy_p.user_led;
    let mut adc = Adc::new(p.ADC1);

    let mut period = 100;
    loop {
        info!("on");
        led.on();
        Timer::after_millis(period).await;

        info!("off");
        led.off();
        Timer::after_millis(period).await;

        let measured = adc.blocking_read(&mut daisy_p.pins.d16);
        period = (measured / 256 + 100) as u64;
    }
}
