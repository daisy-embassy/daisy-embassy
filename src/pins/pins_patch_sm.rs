use embassy_stm32 as hal;
use hal::peripherals::*;

pub type PatchPinA2 = PA1; // ADC_9, GPIO, UART4_RX
pub type PatchPinA3 = PA0; // ADC_10, GPIO, UART4_TX
pub type PatchPinA8 = PB14; // USB_DM, GPIO, USART1_TX
pub type PatchPinA9 = PB15; // USB_DP, GPIO, USART1_RX

pub type PatchPinB5 = PC14; // GATE_OUT_1 Output Only
pub type PatchPinB6 = PC13; // GATE_OUT_2 Output Only
pub type PatchPinB7 = PB8; // I2C1_SCL, GPIO, UART4_RX, PWM (TIM4_CH3)
pub type PatchPinB8 = PB9; // I2C1_SDA, GPIO, UART4_TX, PWM (TIM4_CH4)
pub type PatchPinB9 = PG14; // GATE_IN_2, Input Only
pub type PatchPinB10 = PG13; // GATE_IN_1, Input Only

pub type PatchPinC1 = PA5; // CV_OUT_2, Output Only
pub type PatchPinC2 = PA7; // CV_4, Input Only
pub type PatchPinC3 = PA2; // CV_3, Input Only
pub type PatchPinC4 = PA6; // CV_2, Input Only
pub type PatchPinC5 = PA3; // CV_1, Input Only
pub type PatchPinC6 = PB1; // CV_5, Input Only
pub type PatchPinC7 = PC4; // CV_6, Input Only
pub type PatchPinC8 = PC0; // CV_7, Input Only
pub type PatchPinC9 = PC1; // CV_8, Input Only
pub type PatchPinC10 = PA4; // CV_OUT_1, Output Only

pub type PatchPinD1 = PB4; // SPI2_CS, GPIO
pub type PatchPinD2 = PC11; // SDMMC1_D3, GPIO, USART3_RX*
pub type PatchPinD3 = PC10; // SDMMC1_D2, GPIO, USART3_TX*
pub type PatchPinD4 = PC9; // SDMMC1_D1, GPIO
pub type PatchPinD5 = PC8; // SDMMC1_D0, GPIO
pub type PatchPinD6 = PC12; // SDMMC1_CLK, GPIO, UART5_TX*
pub type PatchPinD7 = PD2; // SDMMC1_CMD, GPIO, UART5_RX*
pub type PatchPinD8 = PC2; // ADC_12, GPIO, SPI2_MISO
pub type PatchPinD9 = PC3; // ADC_11, GPIO, SPI2_MOSI
pub type PatchPinD10 = PD3; // SPI2_SCK, GPIO

pub type Boot = PG3; //on board "BOOT" button

pub struct DaisyPins {
    pub a2: PatchPinA2,
    pub a3: PatchPinA3,
    pub a8: PatchPinA8,
    pub a9: PatchPinA9,

    pub b5: PatchPinB5,
    pub b6: PatchPinB6,
    pub b7: PatchPinB7,
    pub b8: PatchPinB8,
    pub b9: PatchPinB9,
    pub b10: PatchPinB10,

    pub c1: PatchPinC1,
    pub c2: PatchPinC2,
    pub c3: PatchPinC3,
    pub c4: PatchPinC4,
    pub c5: PatchPinC5,
    pub c6: PatchPinC6,
    pub c7: PatchPinC7,
    pub c8: PatchPinC8,
    pub c9: PatchPinC9,
    pub c10: PatchPinC10,

    pub d1: PatchPinD1,
    pub d2: PatchPinD2,
    pub d3: PatchPinD3,
    pub d4: PatchPinD4,
    pub d5: PatchPinD5,
    pub d6: PatchPinD6,
    pub d7: PatchPinD7,
    pub d8: PatchPinD8,
    pub d9: PatchPinD9,
    pub d10: PatchPinD10,
}

pub type LedUserPin = PC7; // LED_USER

#[allow(non_snake_case)]
pub struct USB2Pins {
    pub DN: PA11, // USB2 D-
    pub DP: PA12, // USB2 D+
}

#[allow(non_snake_case)]
pub struct FlashPins {
    // https://github.com/electro-smith/libDaisy/blob/3dda55e9ed55a2f8b6bc4fa6aa2c7ae134c317ab/src/per/qspi.c#L695
    pub IO0: PF8, // (SI)
    pub IO1: PF9, // (SO)
    pub IO2: PF7,
    pub IO3: PF6,
    pub SCK: PF10,
    pub CS: PG6,
}

pub struct SdRamPins {
    pub dd0: PD0,
    pub dd1: PD1,
    pub dd8: PD8,
    pub dd9: PD9,
    pub dd10: PD10,
    pub dd14: PD14,
    pub dd15: PD15,
    pub ee0: PE0,
    pub ee1: PE1,
    pub ee7: PE7,
    pub ee8: PE8,
    pub ee9: PE9,
    pub ee10: PE10,
    pub ee11: PE11,
    pub ee12: PE12,
    pub ee13: PE13,
    pub ee14: PE14,
    pub ee15: PE15,
    pub ff0: PF0,
    pub ff1: PF1,
    pub ff2: PF2,
    pub ff3: PF3,
    pub ff4: PF4,
    pub ff5: PF5,
    pub ff11: PF11,
    pub ff12: PF12,
    pub ff13: PF13,
    pub ff14: PF14,
    pub ff15: PF15,
    pub gg0: PG0,
    pub gg1: PG1,
    pub gg2: PG2,
    pub gg4: PG4,
    pub gg5: PG5,
    pub gg8: PG8,
    pub gg15: PG15,
    pub hh2: PH2,
    pub hh3: PH3,
    pub hh5: PH5,
    pub hh8: PH8,
    pub hh9: PH9,
    pub hh10: PH10,
    pub hh11: PH11,
    pub hh12: PH12,
    pub hh13: PH13,
    pub hh14: PH14,
    pub hh15: PH15,
    pub ii0: PI0,
    pub ii1: PI1,
    pub ii2: PI2,
    pub ii3: PI3,
    pub ii4: PI4,
    pub ii5: PI5,
    pub ii6: PI6,
    pub ii7: PI7,
    pub ii9: PI9,
    pub ii10: PI10,
}
