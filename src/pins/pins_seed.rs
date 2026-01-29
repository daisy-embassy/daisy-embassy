use embassy_stm32::{self as hal, Peri};
use hal::peripherals::*;

// - types --------------------------------------------------------------------

pub type SeedPin0<'a> = Peri<'a, PB12>; // PIN_01, USB OTG ID, I2C3 SCL
pub type SeedPin1<'a> = Peri<'a, PC11>; // PIN_02, SD Data3, USART3 Rx
pub type SeedPin2<'a> = Peri<'a, PC10>; // PIN_03, SD Data2, USART3 Tx
pub type SeedPin3<'a> = Peri<'a, PC9>; // PIN_04, SD Data1, I2C3 SDA
pub type SeedPin4<'a> = Peri<'a, PC8>; // PIN_05, SD Data0
pub type SeedPin5<'a> = Peri<'a, PD2>; // PIN_06, SD CMD, UART5 Rx
pub type SeedPin6<'a> = Peri<'a, PC12>; // PIN_07, SD CLK, UART5 Tx
pub type SeedPin7<'a> = Peri<'a, PG10>; // PIN_08, SPI1 CS
pub type SeedPin8<'a> = Peri<'a, PG11>; // PIN_09, SPI1 SCK, SPDIFRX1
pub type SeedPin9<'a> = Peri<'a, PB4>; // PIN_10, SPI1 MISO
pub type SeedPin10<'a> = Peri<'a, PB5>; // PIN_11, SPI1 MOSI
pub type SeedPin11<'a> = Peri<'a, PB8>; // PIN_12, I2C1 SCL, UART4 Rx
pub type SeedPin12<'a> = Peri<'a, PB9>; // PIN_13, I2C1 SDA, UART4 Tx
pub type SeedPin13<'a> = Peri<'a, PB6>; // PIN_14, USART1 Tx, I2C4 SCL
pub type SeedPin14<'a> = Peri<'a, PB7>; // PIN_15, USART1 Rx, I2C4 SDA
pub type SeedPin15<'a> = Peri<'a, PC0>; // PIN_22, ADC 0
pub type SeedPin16<'a> = Peri<'a, PA3>; // PIN_23, ADC 1
pub type SeedPin17<'a> = Peri<'a, PB1>; // PIN_24, ADC 2
pub type SeedPin18<'a> = Peri<'a, PA7>; // PIN_25, ADC 3
pub type SeedPin19<'a> = Peri<'a, PA6>; // PIN_26, ADC 4
pub type SeedPin20<'a> = Peri<'a, PC1>; // PIN_27, ADC 5
pub type SeedPin21<'a> = Peri<'a, PC4>; // PIN_28, ADC 6
pub type SeedPin22<'a> = Peri<'a, PA5>; // PIN_29, DAC OUT 2, ADC 7
pub type SeedPin23<'a> = Peri<'a, PA4>; // PIN_30, DAC OUT 1, ADC 8
pub type SeedPin24<'a> = Peri<'a, PA1>; // PIN_31, SAI2 MCLK, ADC 9
pub type SeedPin25<'a> = Peri<'a, PA0>; // PIN_32, SAI2 SD B, ADC 10
pub type SeedPin26<'a> = Peri<'a, PD11>; // PIN_33, SAI2 SD A
pub type SeedPin27<'a> = Peri<'a, PG9>; // PIN_34, SAI2 SD FS
pub type SeedPin28<'a> = Peri<'a, PA2>; // PIN_35, SAI2 SCK, ADC 11
pub type SeedPin29<'a> = Peri<'a, PB14>; // PIN_36, USB1 D-, USART1 Tx
pub type SeedPin30<'a> = Peri<'a, PB15>; // PIN_37, USB1 D+, USART1 Rx

pub type Boot<'a> = Peri<'a, PG3>; //on board "BOOT" button

pub struct DaisyPins<'a> {
    pub d0: SeedPin0<'a>,
    pub d1: SeedPin1<'a>,
    pub d2: SeedPin2<'a>,
    pub d3: SeedPin3<'a>,
    pub d4: SeedPin4<'a>,
    pub d5: SeedPin5<'a>,
    pub d6: SeedPin6<'a>,
    pub d7: SeedPin7<'a>,
    pub d8: SeedPin8<'a>,
    pub d9: SeedPin9<'a>,
    pub d10: SeedPin10<'a>,
    pub d11: SeedPin11<'a>,
    pub d12: SeedPin12<'a>,
    pub d13: SeedPin13<'a>,
    pub d14: SeedPin14<'a>,
    pub d15: SeedPin15<'a>,
    pub d16: SeedPin16<'a>,
    pub d17: SeedPin17<'a>,
    pub d18: SeedPin18<'a>,
    pub d19: SeedPin19<'a>,
    pub d20: SeedPin20<'a>,
    pub d21: SeedPin21<'a>,
    pub d22: SeedPin22<'a>,
    pub d23: SeedPin23<'a>,
    pub d24: SeedPin24<'a>,
    pub d25: SeedPin25<'a>,
    pub d26: SeedPin26<'a>,
    pub d27: SeedPin27<'a>,
    pub d28: SeedPin28<'a>,
    pub d29: SeedPin29<'a>,
    pub d30: SeedPin30<'a>,
}

pub type LedUserPin<'a> = Peri<'a, PC7>; // LED_USER

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
