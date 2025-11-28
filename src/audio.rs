use core::convert::Infallible;
use core::marker::PhantomData;

use crate::codec::{Codec, Pins as CodecPins};
use defmt::info;
use embassy_stm32 as hal;
use grounded::uninit::GroundedArrayCell;

use hal::sai::{self, MasterClockDivider};

// - global constants ---------------------------------------------------------

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

        Interface {
            codec: Codec::new(self, audio_config, tx_buffer, rx_buffer).await,
            _state: PhantomData,
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
    codec: Codec<'a>,
    _state: PhantomData<S>,
}

impl<'a> Interface<'a, Idle> {
    /// This has to be called before `Interface::start_callback` can be used to ensure proper setup of the interface.
    /// `Interface::start_callback` should be called immediately afterwards otherwise overruns of the SAI can occur.
    pub async fn start_interface(mut self) -> Result<Interface<'a, Running>, sai::Error> {
        self.codec.start().await?;
        Ok(Interface {
            codec: self.codec,
            _state: PhantomData,
        })
    }

    // returns (sai_tx, sai_rx, i2c)
    #[cfg(any(feature = "seed_1_1", feature = "patch_sm"))]
    pub async fn setup_and_release(
        self,
    ) -> Result<
        (
            sai::Sai<'a, hal::peripherals::SAI1, u32>,
            sai::Sai<'a, hal::peripherals::SAI1, u32>,
            hal::i2c::I2c<'a, hal::mode::Blocking>,
        ),
        sai::Error,
    > {
        self.start_interface().await.map(|i| i.codec.release())
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
            self.codec.read(&mut read_buf).await?;
            callback(&read_buf, &mut write_buf);
            self.codec.write(&write_buf).await?;
        }
    }
}

impl<S: InterfaceState> Interface<'_, S> {
    pub fn sai_rx_config(&self) -> &sai::Config {
        &self.codec.sai_rx_config
    }

    pub fn sai_tx_config(&self) -> &sai::Config {
        &self.codec.sai_tx_config
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
