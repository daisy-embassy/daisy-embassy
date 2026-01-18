//! Driver for the IS25LP064 Flash chip connected via QSPI
//! Notes:
//! 1. The Qspi driver in embassy_stm32 can currently only be used in Blocking mode, because Async would require MDMA, which is unsupported.
//! 2. The Daisy bootloader (as of v6.3) Does not use QPI mode, and configuring the flash chip that way would cause problems on reset. So for compatibility's sake, we do not use it here either.
#![allow(unused)]

use crate::hal;
use crate::pins::FlashPins;
use embassy_stm32::qspi::enums::{AddressSize, ChipSelectHighTime, FIFOThresholdLevel, MemorySize};
use hal::{
    mode::Blocking,
    peripherals::QUADSPI,
    qspi::{
        enums::{DummyCycles, QspiWidth},
        Qspi, TransferConfig,
    },
};

// Commands from IS25LP064 datasheet.
const WRITE_CMD: u8 = 0x32; // PPQ
const WRITE_ENABLE_CMD: u8 = 0x06; // WREN
const SECTOR_ERASE_CMD: u8 = 0xD7; // SER
const FAST_READ_QUAD_IO_CMD: u8 = 0xEB; // FRQIO
const RESET_ENABLE_CMD: u8 = 0x66;
const RESET_MEMORY_CMD: u8 = 0x99;

const WRITE_STATUS_REGISTER_CMD: u8 = 0x01; // WRSR
const READ_STATUS_REGISTER_CMD: u8 = 0x05; // RDSR
const STATUS_BIT_WIP: u8 = 1 << 0;
const STATUS_BIT_WEL: u8 = 1 << 1;
const STATUS_BIT_BP0: u8 = 1 << 2;
const STATUS_BIT_BP1: u8 = 1 << 3;
const STATUS_BIT_BP2: u8 = 1 << 4;
const STATUS_BIT_BP3: u8 = 1 << 5;
const STATUS_BIT_QE: u8 = 1 << 6;
const STATUS_BIT_SRWD: u8 = 1 << 7;

const SET_READ_PARAMETERS_CMD: u8 = 0xC0; // SRP
const READ_PARAMS_BIT_BL0: u8 = 1 << 0;
const READ_PARAMS_BIT_BL1: u8 = 1 << 1;
const READ_PARAMS_BIT_WE: u8 = 1 << 2;
const READ_PARAMS_BIT_DC0: u8 = 1 << 3;
const READ_PARAMS_BIT_DC1: u8 = 1 << 4;
const READ_PARAMS_BIT_ODS0: u8 = 1 << 5;
const READ_PARAMS_BIT_ODS1: u8 = 1 << 6;
const READ_PARAMS_BIT_ODS2: u8 = 1 << 7;

// Memory array specifications as defined in the datasheet.
const SECTOR_SIZE: u32 = 4096;
const PAGE_SIZE: u32 = 256;
const MAX_ADDRESS: u32 = 0x7FFFFF;

pub struct FlashBuilder {
    pub pins: FlashPins,
    pub qspi: QUADSPI,
}

impl FlashBuilder {
    pub fn build<'a>(self) -> Flash<'a> {
        let config = hal::qspi::Config {
            memory_size: MemorySize::_8MiB,
            address_size: AddressSize::_24bit,
            prescaler: 1,
            cs_high_time: ChipSelectHighTime::_2Cycle,
            fifo_threshold: FIFOThresholdLevel::_1Bytes,
        };
        let Self { pins, qspi } = self;
        let qspi = Qspi::new_blocking_bank1(
            qspi, pins.IO0, pins.IO1, pins.IO2, pins.IO3, pins.SCK, pins.CS, config,
        );
        let mut result = Flash { qspi };
        result.reset_memory();
        result.reset_status_register();
        result.reset_read_register();
        result
    }
}

pub struct Flash<'a> {
    qspi: Qspi<'a, QUADSPI, Blocking>,
}

impl Flash<'_> {
    pub fn read(&mut self, address: u32, buffer: &mut [u8]) {
        assert!(address + buffer.len() as u32 <= MAX_ADDRESS);

        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::QUAD,
            dwidth: QspiWidth::QUAD,
            instruction: FAST_READ_QUAD_IO_CMD,
            address: Some(address),
            dummy: DummyCycles::_8,
        };
        self.qspi.blocking_read(buffer, transaction);
    }

    pub fn read_uuid(&mut self) -> [u8; 16] {
        let mut buffer = [0; 16];
        let transaction: TransferConfig = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::SING,
            dwidth: QspiWidth::QUAD,
            instruction: 0x4B,
            address: Some(0x00),
            dummy: DummyCycles::_8,
        };
        self.qspi.blocking_read(&mut buffer, transaction);
        buffer
    }

    pub fn write(&mut self, mut address: u32, data: &[u8]) {
        assert!(address <= MAX_ADDRESS);
        assert!(!data.is_empty());
        self.erase(address, data.len() as u32);

        let mut length = data.len() as u32;
        let mut start_cursor = 0;

        //WRITE_CMD(or PPQ) allows to write up to 256 bytes, which is as much as PAGE_SIZE.
        //Let's divide the data into chunks of page size to write to flash
        loop {
            // Calculate number of bytes between address and end of the page.
            let page_remainder = PAGE_SIZE - (address & (PAGE_SIZE - 1));
            let size = page_remainder.min(length) as usize;
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::SING,
                dwidth: QspiWidth::QUAD,
                instruction: WRITE_CMD,
                address: Some(address),
                dummy: DummyCycles::_0,
            };

            self.qspi
                .blocking_write(&data[start_cursor..start_cursor + size], transaction);
            self.wait_for_write();
            start_cursor += size;

            // Stop if this was the last needed page.
            if length <= page_remainder {
                break;
            }
            length -= page_remainder;

            // Jump to the next page.
            address += page_remainder;
            address %= MAX_ADDRESS;
        }
    }

    pub fn erase(&mut self, mut address: u32, mut length: u32) {
        assert!(address <= MAX_ADDRESS);
        assert!(length > 0);

        loop {
            // Erase the sector.
            self.enable_write();
            let transaction = TransferConfig {
                iwidth: QspiWidth::SING,
                awidth: QspiWidth::SING,
                dwidth: QspiWidth::NONE,
                instruction: SECTOR_ERASE_CMD,
                address: Some(address),
                dummy: DummyCycles::_0,
            };

            self.qspi.blocking_command(transaction);
            self.wait_for_write();

            // Calculate number of bytes between address and end of the sector.
            let sector_remainder = SECTOR_SIZE - (address & (SECTOR_SIZE - 1));

            // Stop if this was the last affected sector.
            if length <= sector_remainder {
                break;
            }
            length -= sector_remainder;

            // Jump to the next sector.
            address += sector_remainder;
            address %= MAX_ADDRESS;
        }
    }

    fn enable_write(&mut self) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: WRITE_ENABLE_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_command(transaction);
    }

    fn wait_for_write(&mut self) {
        loop {
            if self.read_status() & STATUS_BIT_WIP == 0 {
                break;
            }
        }
    }

    fn read_status(&mut self) -> u8 {
        let mut status: [u8; 1] = [0xFF; 1];
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: READ_STATUS_REGISTER_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_read(&mut status, transaction);
        status[0]
    }

    fn reset_memory(&mut self) {
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: RESET_ENABLE_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_command(transaction);

        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::NONE,
            instruction: RESET_MEMORY_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_command(transaction);
    }

    /// Reset status registers into driver's defaults. This makes sure that the
    /// peripheral is configured as expected.
    fn reset_status_register(&mut self) {
        self.enable_write();
        let value = STATUS_BIT_QE;
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: WRITE_STATUS_REGISTER_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_write(&[value], transaction);
        self.wait_for_write();
    }

    /// Reset read registers into driver's defaults. This makes sure that the
    /// peripheral is configured as expected.
    fn reset_read_register(&mut self) {
        let value = READ_PARAMS_BIT_ODS2
            | READ_PARAMS_BIT_ODS1
            | READ_PARAMS_BIT_ODS0
            | READ_PARAMS_BIT_DC1;
        let transaction = TransferConfig {
            iwidth: QspiWidth::SING,
            awidth: QspiWidth::NONE,
            dwidth: QspiWidth::SING,
            instruction: SET_READ_PARAMETERS_CMD,
            address: None,
            dummy: DummyCycles::_0,
        };
        self.qspi.blocking_write(&[value], transaction);
        self.wait_for_write();
    }
}
