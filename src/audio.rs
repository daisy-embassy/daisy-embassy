use core::convert::Infallible;
use core::marker::PhantomData;

use crate::codec::{Codec, Pins as CodecPins};
use defmt::info;
use defmt::unwrap;
use embassy_stm32 as hal;
use grounded::uninit::GroundedArrayCell;
use hal::sai::FifoThreshold;
use hal::sai::FrameSyncOffset;
use hal::sai::{BitOrder, SyncInput};
#[cfg(any(feature = "seed_1_1", feature = "patch_sm"))]
use hal::time::Hertz;
use hal::{
    peripherals,
    sai::{
        self, ClockStrobe, DataSize, FrameSyncPolarity, MasterClockDivider, Mode, Sai, StereoMono,
        TxRx,
    },
};

// - global constants ---------------------------------------------------------

#[cfg(any(feature = "seed_1_1", feature = "patch_sm"))]
const I2C_FS: Hertz = Hertz(100_000);
pub const BLOCK_LENGTH: usize = 32; // 32 samples
pub const HALF_DMA_BUFFER_LENGTH: usize = BLOCK_LENGTH * 2; //  2 channels
pub const DMA_BUFFER_LENGTH: usize = HALF_DMA_BUFFER_LENGTH * 2; //  2 half-blocks

// - static data --------------------------------------------------------------

//DMA buffer must be in special region. Refer https://embassy.dev/book/#_stm32_bdma_only_working_out_of_some_ram_regions
#[link_section = ".sram1_bss"]
static TX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();
#[link_section = ".sram1_bss"]
static RX_BUFFER: GroundedArrayCell<u32, DMA_BUFFER_LENGTH> = GroundedArrayCell::uninit();

// - types --------------------------------------------------------------------

pub type InterleavedBlock = [u32; HALF_DMA_BUFFER_LENGTH];

/// `AudioPeripherals` is a builder to make `Interface` safely.
/// It ensures the correct pin mappings and DMA regions for
/// SAI on every supported Seed revision, preventing invalid peripheral
/// configurations at compile time.
/// Use `prepare_interface()` to apply board‐rev-specific SAI setup
/// and transition into the `Interface<'_, Idle>`. From there you can call `start_interface()` to move to
/// `Interface<'_, Running>` and begin audio callbacks.
pub struct AudioPeripherals {
    pub codec: Codec,
    pub codec_pins: CodecPins,
    pub sai1: hal::peripherals::SAI1,
    pub i2c2: hal::peripherals::I2C2,
    pub dma1_ch0: hal::peripherals::DMA1_CH0,
    pub dma1_ch1: hal::peripherals::DMA1_CH1,
    pub dma1_ch2: hal::peripherals::DMA1_CH2,
}

impl AudioPeripherals {
    /// Prepares the audio interface.
    ///
    /// This method sets up the SAI transmitter and receiver, configures the codec (if necessary),
    /// allocates DMA buffers, and applies board-specific SAI settings. It returns an `Interface<'a, Idle>`
    /// in the Idle state, allowing the runtime to decide when to start audio callbacks using `start_interface()`.
    ///
    /// # Arguments
    /// * `audio_config` - Audio configuration parameters such as the sample rate.  
    ///   You can use `AudioConfig::default()` or `Default::default()` for default settings.
    ///
    /// # Notes
    /// - This method is async because `seed_1_1` requires I2C communication with the WM8731 codec.
    /// - The board revision is selected via Cargo features (`seed_1_1`, `seed_1_2`).
    pub async fn prepare_interface<'a>(self, audio_config: AudioConfig) -> Interface<'a, Idle> {
        #[cfg(feature = "seed_1_1")]
        {
            info!("set up i2c");
            let i2c_config = hal::i2c::Config::default();
            let mut i2c = embassy_stm32::i2c::I2c::new_blocking(
                self.i2c2,
                self.codec_pins.SCL,
                self.codec_pins.SDA,
                I2C_FS,
                i2c_config,
            );
            info!("set up WM8731");
            Codec::setup_wm8731(&mut i2c, audio_config.fs).await;

            info!("set up sai_tx");
            let (sub_block_rx, sub_block_tx) = hal::sai::split_subblocks(self.sai1);
            let mut sai_rx_config = sai::Config::default();
            sai_rx_config.mode = Mode::Master;
            sai_rx_config.tx_rx = TxRx::Receiver;
            sai_rx_config.sync_output = true;
            sai_rx_config.clock_strobe = ClockStrobe::Falling;
            sai_rx_config.master_clock_divider = audio_config.fs.into_clock_divider();
            sai_rx_config.stereo_mono = StereoMono::Stereo;
            sai_rx_config.data_size = DataSize::Data24;
            sai_rx_config.bit_order = BitOrder::MsbFirst;
            sai_rx_config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
            sai_rx_config.frame_sync_offset = FrameSyncOffset::OnFirstBit;
            sai_rx_config.frame_length = 64;
            sai_rx_config.frame_sync_active_level_length = embassy_stm32::sai::word::U7(32);
            sai_rx_config.fifo_threshold = FifoThreshold::Quarter;

            let mut sai_tx_config = sai_rx_config;
            sai_tx_config.mode = Mode::Slave;
            sai_tx_config.tx_rx = TxRx::Transmitter;
            sai_tx_config.sync_input = SyncInput::Internal;
            sai_tx_config.clock_strobe = ClockStrobe::Rising;
            sai_tx_config.sync_output = false;

            let tx_buffer: &mut [u32] = unsafe {
                TX_BUFFER.initialize_all_copied(0);
                let (ptr, len) = TX_BUFFER.get_ptr_len();
                core::slice::from_raw_parts_mut(ptr, len)
            };

            let sai_tx = hal::sai::Sai::new_synchronous(
                sub_block_tx,
                self.codec_pins.SD_B,
                self.dma1_ch1,
                tx_buffer,
                sai_tx_config,
            );

            let rx_buffer: &mut [u32] = unsafe {
                RX_BUFFER.initialize_all_copied(0);
                let (ptr, len) = RX_BUFFER.get_ptr_len();
                core::slice::from_raw_parts_mut(ptr, len)
            };

            let sai_rx = hal::sai::Sai::new_asynchronous_with_mclk(
                sub_block_rx,
                self.codec_pins.SCK_A,
                self.codec_pins.SD_A,
                self.codec_pins.FS_A,
                self.codec_pins.MCLK_A,
                self.dma1_ch2,
                rx_buffer,
                sai_rx_config,
            );

            Interface {
                sai_rx_config,
                sai_tx_config,
                sai_rx,
                sai_tx,
                i2c: Some(i2c),
                _state: PhantomData,
            }
        }

        #[cfg(feature = "seed_1_2")]
        {
            let (sub_block_tx, sub_block_rx) = hal::sai::split_subblocks(self.sai1);
            let mut sai_tx_config = hal::sai::Config::default();
            sai_tx_config.mode = Mode::Master;
            sai_tx_config.tx_rx = TxRx::Transmitter;
            sai_tx_config.sync_output = true;
            sai_tx_config.clock_strobe = ClockStrobe::Falling;
            sai_tx_config.master_clock_divider = audio_config.fs.into_clock_divider();
            sai_tx_config.stereo_mono = StereoMono::Stereo;
            sai_tx_config.data_size = DataSize::Data24;
            sai_tx_config.bit_order = BitOrder::MsbFirst;
            sai_tx_config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
            sai_tx_config.frame_sync_offset = FrameSyncOffset::OnFirstBit;
            sai_tx_config.frame_length = 64;
            sai_tx_config.frame_sync_active_level_length = embassy_stm32::sai::word::U7(32);
            sai_tx_config.fifo_threshold = FifoThreshold::Quarter;

            let mut sai_rx_config = sai_tx_config;
            sai_rx_config.mode = Mode::Slave;
            sai_rx_config.tx_rx = TxRx::Receiver;
            sai_rx_config.sync_input = SyncInput::Internal;
            sai_rx_config.clock_strobe = ClockStrobe::Rising;
            sai_rx_config.sync_output = false;

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

            let sai_tx = Sai::new_asynchronous_with_mclk(
                sub_block_tx,
                self.codec_pins.SCK_A,
                self.codec_pins.SD_A,
                self.codec_pins.FS_A,
                self.codec_pins.MCLK_A,
                self.dma1_ch0,
                tx_buffer,
                sai_tx_config,
            );
            let sai_rx = Sai::new_synchronous(
                sub_block_rx,
                self.codec_pins.SD_B,
                self.dma1_ch1,
                rx_buffer,
                sai_rx_config,
            );

            Interface {
                sai_rx_config,
                sai_tx_config,
                sai_rx,
                sai_tx,
                i2c: None, // pcm3060 'hardware mode' doesn't need i2c
                _state: PhantomData,
            }
        }

        #[cfg(feature = "patch_sm")]
        {
            info!("set up i2c");
            let i2c_config = hal::i2c::Config::default();
            let mut i2c = embassy_stm32::i2c::I2c::new_blocking(
                self.i2c2,
                self.codec_pins.SCL,
                self.codec_pins.SDA,
                I2C_FS,
                i2c_config,
            );
            info!("set up PCM3060");
            Codec::setup_pcm3060(&mut i2c).await;

            info!("set up sai");
            let (sub_block_rx, sub_block_tx) = hal::sai::split_subblocks(self.sai1);

            // The configuration was made to match the the register values obtained with
            // https://github.com/zlosynth/daisy
            let mut sai_rx_config = hal::sai::Config::default();
            sai_rx_config.mode = Mode::Master;
            sai_rx_config.tx_rx = TxRx::Receiver;
            sai_rx_config.sync_output = true;
            sai_rx_config.clock_strobe = ClockStrobe::Rising;
            sai_rx_config.master_clock_divider = audio_config.fs.into_clock_divider();
            sai_rx_config.stereo_mono = StereoMono::Stereo;
            sai_rx_config.data_size = DataSize::Data24;
            sai_rx_config.bit_order = BitOrder::MsbFirst;
            sai_rx_config.frame_sync_polarity = FrameSyncPolarity::ActiveHigh;
            sai_rx_config.frame_sync_offset = FrameSyncOffset::OnFirstBit;
            sai_rx_config.frame_length = 64;
            sai_rx_config.frame_sync_active_level_length = sai::word::U7(32);
            sai_rx_config.fifo_threshold = FifoThreshold::Quarter;
            sai_rx_config.mute_detection_counter = hal::dma::word::U5(0);
            sai_rx_config.slot_size = sai::SlotSize::Channel32;
            sai_rx_config.complement_format = sai::ComplementFormat::OnesComplement;

            let mut sai_tx_config = sai_rx_config;
            sai_tx_config.mode = Mode::Slave;
            sai_tx_config.tx_rx = TxRx::Transmitter;
            sai_tx_config.sync_input = SyncInput::Internal;
            sai_tx_config.clock_strobe = ClockStrobe::Rising;
            sai_tx_config.sync_output = false;

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

            let sai_tx = hal::sai::Sai::new_synchronous(
                sub_block_tx,
                self.codec_pins.SD_B,
                self.dma1_ch1,
                tx_buffer,
                sai_tx_config,
            );

            let sai_rx = hal::sai::Sai::new_asynchronous_with_mclk(
                sub_block_rx,
                self.codec_pins.SCK_A,
                self.codec_pins.SD_A,
                self.codec_pins.FS_A,
                self.codec_pins.MCLK_A,
                self.dma1_ch0,
                rx_buffer,
                sai_rx_config,
            );

            Interface {
                sai_rx_config,
                sai_tx_config,
                sai_rx,
                sai_tx,
                i2c: Some(i2c),
                _state: PhantomData,
            }
        }
    }
}

pub struct Idle {}
pub struct Running {}
pub trait InterfaceState {}
impl InterfaceState for Idle {}
impl InterfaceState for Running {}

/// decides when and how you start audio callback at runtime.
/// It enforces a two-state model:
///
/// * **Idle** – peripherals configured but SAI not started.
/// * **Running** – SAI started, ready to execute audio callbacks.
///
/// Transition from Idle to Running by calling `start_interface()`, which performs
/// codec register writes, waits for codec timing, and starts the SAI receiver and transmitter.
/// Once Running, invoke `start_callback()` to enter a continuous read→process→write loop. Any SAI errors are returned
/// to the caller for custom handling.
///
/// `Interface<'a, S>` manages the setup and runtime of an SAI-based audio stream.
/// It drives codec initialization (over I2C if required), configures SAI TX/RX,
/// and enforces a two-state model:
///
/// * **Idle** – peripherals configured but SAI not started.
/// * **Running** – SAI started, ready to execute audio callbacks.
///
/// Transition from Idle to Running by calling `start_interface()`, which performs
/// codec register writes, waits for codec timing, and starts the SAI receiver and transmitter.
/// Once Running, invoke `start_callback()` to enter a continuous read→process→write loop. Any SAI errors are returned
/// to the caller for custom handling.
///
/// # Example
/// ```rust
/// // 1. Configure peripherals into Idle state
/// let idle: Interface<Idle> = board
///     .audio_peripherals
///     .prepare_interface(Default::default())
///     .await;
///
/// // ... initialize your DSP or other resources ...
///
/// // 2. Start interface and transition to Running
/// let mut audio: Interface<Running> = idle
///     .start_interface()
///     .await
///     .unwrap();
///
/// // 3. Audio processing loop with error handling
/// loop {
///     // Runs until an SAI error occurs, then returns Err(e)
///     if let Err(e) = audio
///         .start_callback(|input, output| {
///             // process `input` samples into `output` buffer
///         })
///         .await
///     {
///         // handle SAI error e (be quick to avoid overrun)
///     }
///
///     // ... optionally reset or reinitialize DSP ...
/// }
/// ```
/// # Notes
/// - Always call `start_interface()` before `start_callback()`.
/// - Keep callback and error-handling routines short to prevent SAI overruns.
pub struct Interface<'a, S: InterfaceState> {
    sai_tx_config: sai::Config,
    sai_rx_config: sai::Config,
    sai_tx: Sai<'a, peripherals::SAI1, u32>,
    sai_rx: Sai<'a, peripherals::SAI1, u32>,
    i2c: Option<hal::i2c::I2c<'a, hal::mode::Blocking>>,
    _state: PhantomData<S>,
}

impl<'a> Interface<'a, Idle> {
    /// This has to be called before `Interface::start_callback` can be used to ensure proper setup of the interface.
    /// `Interface::start_callback` should be called immediately afterwards otherwise overruns of the SAI can occur.
    pub async fn start_interface(mut self) -> Result<Interface<'a, Running>, sai::Error> {
        self.setup().await?;
        Ok(Interface {
            sai_tx_config: self.sai_tx_config,
            sai_rx_config: self.sai_rx_config,
            sai_tx: self.sai_tx,
            sai_rx: self.sai_rx,
            i2c: self.i2c,
            _state: PhantomData,
        })
    }

    async fn setup(&mut self) -> Result<(), sai::Error> {
        #[cfg(feature = "seed_1_1")]
        {
            info!("setup WM8731");
            Codec::write_wm8731_reg(
                self.i2c.as_mut().unwrap(),
                wm8731::WM8731::power_down(Codec::final_power_settings),
            );
            embassy_time::Timer::after_micros(10).await;
        }

        info!("start SAI");
        #[cfg(any(feature = "seed_1_2", feature = "patch_sm"))]
        {
            // As the SAI configuration for the PCM3060
            // codec requires the SAI reciever to be in
            // slave mode, the master SAI has to be started
            // as well for the slave SAI to work.
            // As of embassy-stm32 v0.2.0 this can only
            // be done by writing to the transmitter once.
            let write_buf = [0; HALF_DMA_BUFFER_LENGTH];
            self.sai_tx.write(&write_buf).await?;
        }

        self.sai_rx.start()
    }

    // returns (sai_tx, sai_rx, i2c)
    pub async fn setup_and_release(
        mut self,
    ) -> Result<
        (
            Sai<'a, peripherals::SAI1, u32>,
            Sai<'a, peripherals::SAI1, u32>,
            hal::i2c::I2c<'a, hal::mode::Blocking>,
        ),
        sai::Error,
    > {
        self.setup().await?;
        Ok((self.sai_tx, self.sai_rx, unwrap!(self.i2c)))
    }
}

impl Interface<'_, Running> {
    pub async fn start_callback(
        &mut self,
        mut callback: impl FnMut(&[u32], &mut [u32]),
    ) -> Result<Infallible, sai::Error> {
        info!("enter audio callback loop");
        let mut write_buf = [0; HALF_DMA_BUFFER_LENGTH];
        let mut read_buf = [0; HALF_DMA_BUFFER_LENGTH];
        loop {
            self.sai_rx.read(&mut read_buf).await?;
            callback(&read_buf, &mut write_buf);
            self.sai_tx.write(&write_buf).await?;
        }
    }
}

impl<S: InterfaceState> Interface<'_, S> {
    pub fn sai_rx_config(&self) -> &sai::Config {
        &self.sai_rx_config
    }

    pub fn sai_tx_config(&self) -> &sai::Config {
        &self.sai_tx_config
    }
}

#[derive(Clone, Copy)]
pub enum Fs {
    Fs8000,
    Fs32000,
    Fs44100,
    Fs48000,
    Fs88200,
    Fs96000,
}
const CLOCK_RATIO: u32 = 256; //Not yet support oversampling.
impl Fs {
    pub fn into_clock_divider(self) -> MasterClockDivider {
        let fs = match self {
            Fs::Fs8000 => 8000,
            Fs::Fs32000 => 32000,
            Fs::Fs44100 => 44100,
            Fs::Fs48000 => 48000,
            Fs::Fs88200 => 88200,
            Fs::Fs96000 => 96000,
        };
        let kernel_clock = hal::rcc::frequency::<hal::peripherals::SAI1>().0;
        let mclk_div = (kernel_clock / (fs * CLOCK_RATIO)) as u8;
        mclk_div_from_u8(mclk_div)
    }
}

pub struct AudioConfig {
    pub fs: Fs,
}

impl Default for AudioConfig {
    fn default() -> Self {
        AudioConfig { fs: Fs::Fs48000 }
    }
}

//================================================

const fn mclk_div_from_u8(v: u8) -> MasterClockDivider {
    match v {
        1 => MasterClockDivider::Div1,
        2 => MasterClockDivider::Div2,
        3 => MasterClockDivider::Div3,
        4 => MasterClockDivider::Div4,
        5 => MasterClockDivider::Div5,
        6 => MasterClockDivider::Div6,
        7 => MasterClockDivider::Div7,
        8 => MasterClockDivider::Div8,
        9 => MasterClockDivider::Div9,
        10 => MasterClockDivider::Div10,
        11 => MasterClockDivider::Div11,
        12 => MasterClockDivider::Div12,
        13 => MasterClockDivider::Div13,
        14 => MasterClockDivider::Div14,
        15 => MasterClockDivider::Div15,
        16 => MasterClockDivider::Div16,
        17 => MasterClockDivider::Div17,
        18 => MasterClockDivider::Div18,
        19 => MasterClockDivider::Div19,
        20 => MasterClockDivider::Div20,
        21 => MasterClockDivider::Div21,
        22 => MasterClockDivider::Div22,
        23 => MasterClockDivider::Div23,
        24 => MasterClockDivider::Div24,
        25 => MasterClockDivider::Div25,
        26 => MasterClockDivider::Div26,
        27 => MasterClockDivider::Div27,
        28 => MasterClockDivider::Div28,
        29 => MasterClockDivider::Div29,
        30 => MasterClockDivider::Div30,
        31 => MasterClockDivider::Div31,
        32 => MasterClockDivider::Div32,
        33 => MasterClockDivider::Div33,
        34 => MasterClockDivider::Div34,
        35 => MasterClockDivider::Div35,
        36 => MasterClockDivider::Div36,
        37 => MasterClockDivider::Div37,
        38 => MasterClockDivider::Div38,
        39 => MasterClockDivider::Div39,
        40 => MasterClockDivider::Div40,
        41 => MasterClockDivider::Div41,
        42 => MasterClockDivider::Div42,
        43 => MasterClockDivider::Div43,
        44 => MasterClockDivider::Div44,
        45 => MasterClockDivider::Div45,
        46 => MasterClockDivider::Div46,
        47 => MasterClockDivider::Div47,
        48 => MasterClockDivider::Div48,
        49 => MasterClockDivider::Div49,
        50 => MasterClockDivider::Div50,
        51 => MasterClockDivider::Div51,
        52 => MasterClockDivider::Div52,
        53 => MasterClockDivider::Div53,
        54 => MasterClockDivider::Div54,
        55 => MasterClockDivider::Div55,
        56 => MasterClockDivider::Div56,
        57 => MasterClockDivider::Div57,
        58 => MasterClockDivider::Div58,
        59 => MasterClockDivider::Div59,
        60 => MasterClockDivider::Div60,
        61 => MasterClockDivider::Div61,
        62 => MasterClockDivider::Div62,
        63 => MasterClockDivider::Div63,
        _ => panic!(),
    }
}
