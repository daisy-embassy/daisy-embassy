// controls the speed of the blinking onboard LED using a potentiometer
// Connect the middle pin/wiper of the potentiometer to pin ADC1/D16, connect bottom pin to ground, and top pin to 3V3 Analogue(pin 21).

#![no_std]
#![no_main]

use daisy_embassy::led::UserLed;
use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel as _, SampleTime};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use grounded::uninit::GroundedArrayCell;
use {defmt_rtt as _, panic_probe as _};

use {defmt_rtt as _, panic_probe as _};

static PERIOD_CONTROL: Signal<CriticalSectionRawMutex, u16> = Signal::new();

#[unsafe(link_section = ".sram1_bss")]
static ADC_BUFFER: GroundedArrayCell<u16, 2> = GroundedArrayCell::uninit();

#[embassy_executor::task]
async fn blink(mut led: UserLed<'static>) {
    // Blink LED while audio passthrough to show sign of life
    let mut period = 100;
    loop {
        info!("on");
        led.on();
        Timer::after_millis(period).await;

        info!("off");
        led.off();
        Timer::after_millis(period).await;

        if let Some(measured) = PERIOD_CONTROL.try_take() {
            period = (measured / 256 + 100) as u64;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = daisy_embassy::default_rcc();
    config.rcc.mux.adcsel = embassy_stm32::rcc::mux::Adcsel::PLL3_R;

    let mut p = embassy_stm32::init(config);
    let daisy_p = new_daisy_board!(p);
    spawner.spawn(blink(daisy_p.user_led)).unwrap();

    // let mut read_buffer: [u16; 2] = [0; 2];
    let adc_buffer: &mut [u16] = unsafe {
        ADC_BUFFER.initialize_all_copied(0);
        let (ptr, len) = ADC_BUFFER.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };

    let mut adc = Adc::new(p.ADC1);
    let mut vrefint_channel = adc.enable_vrefint().degrade_adc();
    let mut pc0 = daisy_p.pins.d16.degrade_adc();

    loop {
        Timer::after_millis(10).await;
        adc.read(
            &mut p.DMA2_CH1,
            [
                (&mut vrefint_channel, SampleTime::CYCLES387_5),
                (&mut pc0, SampleTime::CYCLES387_5),
            ]
            .into_iter(),
            adc_buffer,
        )
        .await;

        let vrefint = adc_buffer[0];
        info!("vrefint: {}", vrefint);
        let measured = adc_buffer[1];
        info!("measured: {}", measured);
        PERIOD_CONTROL.signal(measured);
    }
}
