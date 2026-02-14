// Gain example with faust

#![no_std]
#![no_main]
use core::{array::from_fn, num::Wrapping};
use daisy_embassy::{
    DaisyBoard, audio::HALF_DMA_BUFFER_LENGTH, hal, led::UserLed, new_daisy_board,
};
use defmt::{debug, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;
use faust_ui::{UIRange, UISetAny};
use panic_probe as _;

mod dsp;

static SHARED_VOLUME: Signal<CriticalSectionRawMutex, f32> = Signal::new();

#[embassy_executor::task]
async fn blink(mut led: UserLed<'static>) {
    let mut volume = 0.01;
    // Blink LED while audio passthrough to show sign of life
    loop {
        led.on();
        Timer::after_millis(500).await;

        led.off();
        Timer::after_millis(500).await;

        if volume <= 0.5 {
            volume *= 2.0;
        } else {
            volume = 0.01;
        }

        SHARED_VOLUME.signal(volume);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    debug!("====program start====");
    let config = daisy_embassy::default_rcc();
    let p = hal::init(config);
    let board: DaisyBoard<'_> = new_daisy_board!(p);

    let led = board.user_led;
    spawner.spawn(blink(led)).unwrap();

    let interface = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;

    dsp::LpVol::class_init(48000);
    let mut dsp = dsp::LpVol::new();
    dsp.instance_init(48000);
    let mut interface = unwrap!(interface.start_interface().await);
    unwrap!(
        interface
            .start_callback(|input, output| {
                process_audio_faust(&mut dsp, input, output);
            })
            .await
    );
}

fn process_audio_faust(dsp: &mut dsp::LpVol, input: &[u32], output: &mut [u32]) {
    let ibuf: [[f32; 64]; dsp::FAUST_INPUTS] = from_fn(|_| from_fn(|i| u24_to_f32(input[i])));
    let mut obuf: [[f32; 64]; dsp::FAUST_OUTPUTS] = from_fn(|_| [0.0_f32; HALF_DMA_BUFFER_LENGTH]);

    // if a new value is recieved, set it.
    // only checked once per buffer copy
    if let Some(volume) = SHARED_VOLUME.try_take() {
        dsp::UIActive::Gain.set(dsp, dsp::UIActive::Gain.map(volume));
    };

    dsp.compute(HALF_DMA_BUFFER_LENGTH, &ibuf, &mut obuf);

    for (i, f32_value) in obuf[0].iter().enumerate() {
        output[i] = f32_to_u24(*f32_value);
    }
}

// see https://github.com/zlosynth/daisy
// Convert audio PCM data from u24 to f32,
#[inline(always)]
fn u24_to_f32(y: u32) -> f32 {
    let y = (Wrapping(y) + Wrapping(0x0080_0000)).0 & 0x00FF_FFFF; // convert to i32
    (y as f32 / 8_388_608.0) - 1.0 // (2^24) / 2
}

// Convert audio data from f32 to u24 PCM
#[inline(always)]
fn f32_to_u24(x: f32) -> u32 {
    let x = x * 8_388_607.0;
    let x = x.clamp(-8_388_608.0, 8_388_607.0);
    (x as i32) as u32
}
