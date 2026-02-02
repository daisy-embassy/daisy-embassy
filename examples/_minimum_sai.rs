//! this example does not belong to daisy_embassy,
//! but is to check proper settings of stm32h750's SAI and WM8731.
#![no_std]
#![no_main]
use defmt::{info, unwrap, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32 as hal;
use embassy_stm32::Peri;
use embassy_stm32::bind_interrupts;
use embassy_stm32::dma;
use embassy_stm32::i2c::Master;
use embassy_time::Timer;
use grounded::uninit::GroundedArrayCell;
use hal::peripherals::*;
use hal::sai::*;
use panic_probe as _;
// - global constants ---------------------------------------------------------

pub const BLOCK_LENGTH: usize = 32; // 32 samples
pub const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2; //  2 channels
pub const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks
pub const SAMPLE_RATE: u32 = 48000;

//DMA buffer must be in special region. Refer https://embassy.dev/book/#_stm32_bdma_only_working_out_of_some_ram_regions
#[unsafe(link_section = ".sram1_bss")]
static TX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();
#[unsafe(link_section = ".sram1_bss")]
static RX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();

bind_interrupts!(
    pub struct Irqs{
        DMA1_STREAM0 => dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH0>;
        DMA1_STREAM1 => dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH1>;
        DMA1_STREAM2 => dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH2>;
});

#[embassy_executor::task]
async fn execute(hal_config: hal::Config) {
    let p = hal::init(hal_config);

    //setup codecs via I2C before init SAI.
    //Once SAI is initiated, the bus will be occupied by it.
    setup_codecs_from_i2c(p.I2C2, p.PH4, p.PB11).await;

    let (sub_block_rx, sub_block_tx) = hal::sai::split_subblocks(p.SAI1);
    let kernel_clock = hal::rcc::frequency::<hal::peripherals::SAI1>().0;
    let mclk_div = mclk_div_from_u8((kernel_clock / (SAMPLE_RATE * 256)) as u8);

    let mut rx_config = Config::default();
    rx_config.mode = Mode::Master;
    rx_config.tx_rx = TxRx::Receiver;
    rx_config.sync_output = true;
    rx_config.clock_strobe = ClockStrobe::Falling;
    rx_config.master_clock_divider = mclk_div;
    rx_config.stereo_mono = StereoMono::Stereo;
    rx_config.data_size = DataSize::Data24;
    rx_config.bit_order = BitOrder::MsbFirst;
    rx_config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
    rx_config.frame_sync_offset = FrameSyncOffset::OnFirstBit;
    rx_config.frame_length = 64;
    rx_config.frame_sync_active_level_length = embassy_stm32::sai::word::U7(32);
    rx_config.fifo_threshold = FifoThreshold::Quarter;

    let mut tx_config = rx_config;
    tx_config.mode = Mode::Slave;
    tx_config.tx_rx = TxRx::Transmitter;
    tx_config.sync_input = SyncInput::Internal;
    tx_config.clock_strobe = ClockStrobe::Rising;
    tx_config.sync_output = false;

    let tx_buffer: &mut [u32] = unsafe {
        TX_BUFFER.initialize_all_copied(0);
        let (ptr, len) = TX_BUFFER.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };
    let rx_buffer: &mut [u32] = unsafe {
        RX_BUFFER.initialize_all_copied(0);
        let (ptr, len) = RX_BUFFER.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };

    let mut sai_receiver = Sai::new_asynchronous_with_mclk(
        sub_block_rx,
        p.PE5,
        p.PE6,
        p.PE4,
        p.PE2,
        p.DMA1_CH0,
        rx_buffer,
        Irqs,
        rx_config,
    );

    let mut sai_transmitter =
        Sai::new_synchronous(sub_block_tx, p.PE3, p.DMA1_CH1, tx_buffer, Irqs, tx_config);

    unwrap!(sai_receiver.start());
    unwrap!(sai_transmitter.start());

    let mut rx_signal = [0u32; HALF_DMA_BUFFER_LENGTH];

    info!("enter audio loop");
    loop {
        match sai_receiver.read(&mut rx_signal).await {
            Ok(_) => {}
            Err(e) => {
                warn!("Error reading from SAI: {:?}", e);
            }
        }

        match sai_transmitter.write(&rx_signal).await {
            Ok(_) => {}
            Err(e) => {
                warn!("Error writing to SAI: {:?}", e);
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = daisy_embassy::default_rcc();
    spawner.spawn(execute(config)).unwrap();
}

async fn setup_codecs_from_i2c(
    i2c2: Peri<'static, I2C2>,
    ph4: Peri<'static, PH4>,
    pb11: Peri<'static, PB11>,
) {
    use wm8731::{WM8731, power_down};
    info!("setup codecs from I2C");
    let i2c_config = hal::i2c::Config::default();
    let mut i2c = embassy_stm32::i2c::I2c::new_blocking(i2c2, ph4, pb11, i2c_config);

    fn final_power_settings(w: &mut power_down::PowerDown) {
        w.power_off().power_on();
        w.clock_output().power_off();
        w.oscillator().power_off();
        w.output().power_on();
        w.dac().power_on();
        w.adc().power_on();
        w.mic().power_off();
        w.line_input().power_on();
    }
    fn write(
        i2c: &mut embassy_stm32::i2c::I2c<'static, hal::mode::Blocking, Master>,
        r: wm8731::Register,
    ) {
        const AD: u8 = 0x1a; // or 0x1b if CSB is high

        // WM8731 has 16 bits registers.
        // The first 7 bits are for the addresses, and the rest 9 bits are for the "value"s.
        // Let's pack wm8731::Register into 16 bits.
        let byte1: u8 = ((r.address << 1) & 0b1111_1110) | (((r.value >> 8) & 0b0000_0001) as u8);
        let byte2: u8 = (r.value & 0b1111_1111) as u8;
        i2c.blocking_write(AD, &[byte1, byte2]).unwrap();
    }
    Timer::after_micros(10).await;

    // reset
    write(&mut i2c, WM8731::reset());
    Timer::after_micros(10).await;

    // wakeup
    write(
        &mut i2c,
        WM8731::power_down(|w| {
            final_power_settings(w);
            //output off during initialization
            w.output().power_off();
        }),
    );
    Timer::after_micros(10).await;

    // disable input mute, set to 0dB gain
    write(
        &mut i2c,
        WM8731::left_line_in(|w| {
            w.both().enable();
            w.mute().disable();
            w.volume().nearest_dB(0);
        }),
    );
    Timer::after_micros(10).await;

    // sidetone off; DAC selected; bypass off; line input selected; mic muted; mic boost off
    write(
        &mut i2c,
        WM8731::analog_audio_path(|w| {
            w.sidetone().disable();
            w.dac_select().select();
            w.bypass().disable();
            w.input_select().line_input();
            w.mute_mic().enable();
            w.mic_boost().disable();
        }),
    );
    Timer::after_micros(10).await;

    // disable DAC mute, deemphasis for 48k
    write(
        &mut i2c,
        WM8731::digital_audio_path(|w| {
            w.dac_mut().disable();
            w.deemphasis().frequency_48();
        }),
    );
    Timer::after_micros(10).await;

    // nothing inverted, slave, 32-bits, MSB format
    write(
        &mut i2c,
        WM8731::digital_audio_interface_format(|w| {
            w.bit_clock_invert().no_invert();
            w.master_slave().slave();
            w.left_right_dac_clock_swap().right_channel_dac_data_right();
            w.left_right_phase().data_when_daclrc_low();
            w.bit_length().bits_24();
            w.format().left_justified();
        }),
    );
    Timer::after_micros(10).await;

    // no clock division, normal mode, 48k
    write(
        &mut i2c,
        WM8731::sampling(|w| {
            w.core_clock_divider_select().normal();
            w.base_oversampling_rate().normal_256();
            w.sample_rate().adc_48();
            w.usb_normal().normal();
        }),
    );
    Timer::after_micros(10).await;

    // set active
    write(&mut i2c, WM8731::active().active());
    Timer::after_micros(10).await;

    // enable output
    write(&mut i2c, WM8731::power_down(final_power_settings));
    Timer::after_micros(10).await;
}

const fn mclk_div_from_u8(v: u8) -> MasterClockDivider {
    match v {
        1 => MasterClockDivider::DIV1,
        2 => MasterClockDivider::DIV2,
        3 => MasterClockDivider::DIV3,
        4 => MasterClockDivider::DIV4,
        5 => MasterClockDivider::DIV5,
        6 => MasterClockDivider::DIV6,
        7 => MasterClockDivider::DIV7,
        8 => MasterClockDivider::DIV8,
        9 => MasterClockDivider::DIV9,
        10 => MasterClockDivider::DIV10,
        11 => MasterClockDivider::DIV11,
        12 => MasterClockDivider::DIV12,
        13 => MasterClockDivider::DIV13,
        14 => MasterClockDivider::DIV14,
        15 => MasterClockDivider::DIV15,
        16 => MasterClockDivider::DIV16,
        17 => MasterClockDivider::DIV17,
        18 => MasterClockDivider::DIV18,
        19 => MasterClockDivider::DIV19,
        20 => MasterClockDivider::DIV20,
        21 => MasterClockDivider::DIV21,
        22 => MasterClockDivider::DIV22,
        23 => MasterClockDivider::DIV23,
        24 => MasterClockDivider::DIV24,
        25 => MasterClockDivider::DIV25,
        26 => MasterClockDivider::DIV26,
        27 => MasterClockDivider::DIV27,
        28 => MasterClockDivider::DIV28,
        29 => MasterClockDivider::DIV29,
        30 => MasterClockDivider::DIV30,
        31 => MasterClockDivider::DIV31,
        32 => MasterClockDivider::DIV32,
        33 => MasterClockDivider::DIV33,
        34 => MasterClockDivider::DIV34,
        35 => MasterClockDivider::DIV35,
        36 => MasterClockDivider::DIV36,
        37 => MasterClockDivider::DIV37,
        38 => MasterClockDivider::DIV38,
        39 => MasterClockDivider::DIV39,
        40 => MasterClockDivider::DIV40,
        41 => MasterClockDivider::DIV41,
        42 => MasterClockDivider::DIV42,
        43 => MasterClockDivider::DIV43,
        44 => MasterClockDivider::DIV44,
        45 => MasterClockDivider::DIV45,
        46 => MasterClockDivider::DIV46,
        47 => MasterClockDivider::DIV47,
        48 => MasterClockDivider::DIV48,
        49 => MasterClockDivider::DIV49,
        50 => MasterClockDivider::DIV50,
        51 => MasterClockDivider::DIV51,
        52 => MasterClockDivider::DIV52,
        53 => MasterClockDivider::DIV53,
        54 => MasterClockDivider::DIV54,
        55 => MasterClockDivider::DIV55,
        56 => MasterClockDivider::DIV56,
        57 => MasterClockDivider::DIV57,
        58 => MasterClockDivider::DIV58,
        59 => MasterClockDivider::DIV59,
        60 => MasterClockDivider::DIV60,
        61 => MasterClockDivider::DIV61,
        62 => MasterClockDivider::DIV62,
        63 => MasterClockDivider::DIV63,
        _ => panic!(),
    }
}
