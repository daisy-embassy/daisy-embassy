use embassy_stm32 as hal;
use embassy_stm32::Peri;
use hal::peripherals::*;

pub type PatchPinA2<'a> = Peri<'a, PA1>; // ADC_9, GPIO, UART4_RX
pub type PatchPinA3<'a> = Peri<'a, PA0>; // ADC_10, GPIO, UART4_TX
pub type PatchPinA8<'a> = Peri<'a, PB14>; // USB_DM, GPIO, USART1_TX
pub type PatchPinA9<'a> = Peri<'a, PB15>; // USB_DP, GPIO, USART1_RX

pub type PatchPinB5<'a> = Peri<'a, PC14>; // GATE_OUT_1 Output Only
pub type PatchPinB6<'a> = Peri<'a, PC13>; // GATE_OUT_2 Output Only
pub type PatchPinB7<'a> = Peri<'a, PB8>; // I2C1_SCL, GPIO, UART4_RX, PWM (TIM4_CH3)
pub type PatchPinB8<'a> = Peri<'a, PB9>; // I2C1_SDA, GPIO, UART4_TX, PWM (TIM4_CH4)
pub type PatchPinB9<'a> = Peri<'a, PG14>; // GATE_IN_2, Input Only
pub type PatchPinB10<'a> = Peri<'a, PG13>; // GATE_IN_1, Input Only

pub type PatchPinC1<'a> = Peri<'a, PA5>; // CV_OUT_2, Output Only
pub type PatchPinC2<'a> = Peri<'a, PA7>; // CV_4, Input Only
pub type PatchPinC3<'a> = Peri<'a, PA2>; // CV_3, Input Only
pub type PatchPinC4<'a> = Peri<'a, PA6>; // CV_2, Input Only
pub type PatchPinC5<'a> = Peri<'a, PA3>; // CV_1, Input Only
pub type PatchPinC6<'a> = Peri<'a, PB1>; // CV_5, Input Only
pub type PatchPinC7<'a> = Peri<'a, PC4>; // CV_6, Input Only
pub type PatchPinC8<'a> = Peri<'a, PC0>; // CV_7, Input Only
pub type PatchPinC9<'a> = Peri<'a, PC1>; // CV_8, Input Only
pub type PatchPinC10<'a> = Peri<'a, PA4>; // CV_OUT_1, Output Only

pub type PatchPinD1<'a> = Peri<'a, PB4>; // SPI2_CS, GPIO
pub type PatchPinD2<'a> = Peri<'a, PC11>; // SDMMC1_D3, GPIO, USART3_RX*
pub type PatchPinD3<'a> = Peri<'a, PC10>; // SDMMC1_D2, GPIO, USART3_TX*
pub type PatchPinD4<'a> = Peri<'a, PC9>; // SDMMC1_D1, GPIO
pub type PatchPinD5<'a> = Peri<'a, PC8>; // SDMMC1_D0, GPIO
pub type PatchPinD6<'a> = Peri<'a, PC12>; // SDMMC1_CLK, GPIO, UART5_TX*
pub type PatchPinD7<'a> = Peri<'a, PD2>; // SDMMC1_CMD, GPIO, UART5_RX*
pub type PatchPinD8<'a> = Peri<'a, PC2>; // ADC_12, GPIO, SPI2_MISO
pub type PatchPinD9<'a> = Peri<'a, PC3>; // ADC_11, GPIO, SPI2_MOSI
pub type PatchPinD10<'a> = Peri<'a, PD3>; // SPI2_SCK, GPIO

pub type Boot<'a> = Peri<'a, PG3>; //on board "BOOT" button

pub struct DaisyPins<'a> {
    pub a2: PatchPinA2<'a>,
    pub a3: PatchPinA3<'a>,
    pub a8: PatchPinA8<'a>,
    pub a9: PatchPinA9<'a>,

    pub b5: PatchPinB5<'a>,
    pub b6: PatchPinB6<'a>,
    pub b7: PatchPinB7<'a>,
    pub b8: PatchPinB8<'a>,
    pub b9: PatchPinB9<'a>,
    pub b10: PatchPinB10<'a>,

    pub c1: PatchPinC1<'a>,
    pub c2: PatchPinC2<'a>,
    pub c3: PatchPinC3<'a>,
    pub c4: PatchPinC4<'a>,
    pub c5: PatchPinC5<'a>,
    pub c6: PatchPinC6<'a>,
    pub c7: PatchPinC7<'a>,
    pub c8: PatchPinC8<'a>,
    pub c9: PatchPinC9<'a>,
    pub c10: PatchPinC10<'a>,

    pub d1: PatchPinD1<'a>,
    pub d2: PatchPinD2<'a>,
    pub d3: PatchPinD3<'a>,
    pub d4: PatchPinD4<'a>,
    pub d5: PatchPinD5<'a>,
    pub d6: PatchPinD6<'a>,
    pub d7: PatchPinD7<'a>,
    pub d8: PatchPinD8<'a>,
    pub d9: PatchPinD9<'a>,
    pub d10: PatchPinD10<'a>,
}

pub type LedUserPin = PC7; // LED_USER

#[allow(non_snake_case)]
pub struct USB2Pins<'a> {
    pub DN: Peri<'a, PA11>, // USB2 D-
    pub DP: Peri<'a, PA12>, // USB2 D+
}

#[allow(non_snake_case)]
pub struct FlashPins<'a> {
    // https://github.com/electro-smith/libDaisy/blob/3dda55e9ed55a2f8b6bc4fa6aa2c7ae134c317ab/src/per/qspi.c#L695
    pub IO0: Peri<'a, PF8>, // (SI)
    pub IO1: Peri<'a, PF9>, // (SO)
    pub IO2: Peri<'a, PF7>,
    pub IO3: Peri<'a, PF6>,
    pub SCK: Peri<'a, PF10>,
    pub CS: Peri<'a, PG6>,
}

pub struct SdRamPins<'a> {
    pub dd0: Peri<'a, PD0>,
    pub dd1: Peri<'a, PD1>,
    pub dd8: Peri<'a, PD8>,
    pub dd9: Peri<'a, PD9>,
    pub dd10: Peri<'a, PD10>,
    pub dd14: Peri<'a, PD14>,
    pub dd15: Peri<'a, PD15>,
    pub ee0: Peri<'a, PE0>,
    pub ee1: Peri<'a, PE1>,
    pub ee7: Peri<'a, PE7>,
    pub ee8: Peri<'a, PE8>,
    pub ee9: Peri<'a, PE9>,
    pub ee10: Peri<'a, PE10>,
    pub ee11: Peri<'a, PE11>,
    pub ee12: Peri<'a, PE12>,
    pub ee13: Peri<'a, PE13>,
    pub ee14: Peri<'a, PE14>,
    pub ee15: Peri<'a, PE15>,
    pub ff0: Peri<'a, PF0>,
    pub ff1: Peri<'a, PF1>,
    pub ff2: Peri<'a, PF2>,
    pub ff3: Peri<'a, PF3>,
    pub ff4: Peri<'a, PF4>,
    pub ff5: Peri<'a, PF5>,
    pub ff11: Peri<'a, PF11>,
    pub ff12: Peri<'a, PF12>,
    pub ff13: Peri<'a, PF13>,
    pub ff14: Peri<'a, PF14>,
    pub ff15: Peri<'a, PF15>,
    pub gg0: Peri<'a, PG0>,
    pub gg1: Peri<'a, PG1>,
    pub gg2: Peri<'a, PG2>,
    pub gg4: Peri<'a, PG4>,
    pub gg5: Peri<'a, PG5>,
    pub gg8: Peri<'a, PG8>,
    pub gg15: Peri<'a, PG15>,
    pub hh2: Peri<'a, PH2>,
    pub hh3: Peri<'a, PH3>,
    pub hh5: Peri<'a, PH5>,
    pub hh8: Peri<'a, PH8>,
    pub hh9: Peri<'a, PH9>,
    pub hh10: Peri<'a, PH10>,
    pub hh11: Peri<'a, PH11>,
    pub hh12: Peri<'a, PH12>,
    pub hh13: Peri<'a, PH13>,
    pub hh14: Peri<'a, PH14>,
    pub hh15: Peri<'a, PH15>,
    pub ii0: Peri<'a, PI0>,
    pub ii1: Peri<'a, PI1>,
    pub ii2: Peri<'a, PI2>,
    pub ii3: Peri<'a, PI3>,
    pub ii4: Peri<'a, PI4>,
    pub ii5: Peri<'a, PI5>,
    pub ii6: Peri<'a, PI6>,
    pub ii7: Peri<'a, PI7>,
    pub ii9: Peri<'a, PI9>,
    pub ii10: Peri<'a, PI10>,
}
