use std::path::Path;
use std::fs::{OpenOptions, File};
use std::io::Read;
use std::result::Result::{self, Ok, Err};

use libmodbus_rs::{Modbus, ModbusClient, ModbusRTU, SerialMode, Timeout};

use retry::{Retry, RetryError};

use hex_slice::AsHex;

use sunsaver::*;

pub trait SunSaverConnection {
    fn read_raw_registers(&mut self) -> [u16; 44];

    fn read_raw_logged(&mut self) -> [u16; 32 * 16];

    fn read_status(&mut self) -> SunSaverResponse {
        SunSaverResponse::from_raw_bits(self.read_raw_registers())
    }

    fn read_logged(&mut self) -> LoggedResponse {
        LoggedResponse::from_raw_bits(self.read_raw_logged())
    }
}

pub struct ModbusSunSaverConnection {
    connection: Modbus,
}

impl ModbusSunSaverConnection {
    pub fn open(device: &Path) -> ModbusSunSaverConnection {
        /* A Meterbus to Serial Converter (MSC) is required to adapt the Meter interface to an isolated RS-232 interface**.
           The SunSaver MPPT supports RTU mode only.
           16bit MODBUS® addresses (per the modbus.org spec)
           The serial communication parameters are:
             * BPS: 9600 baud
             * Parity: None
             * Data bits: 8
             * Stop bits: 2
             * Flow control: None
            All addresses listed are for the request PDU.
            The SunSaver MPPT default server address: 0x01. */
        debug!("Configuring device {:?}", device);
        let mut connection = Modbus::new_rtu(device.to_str().unwrap(), 9600, 'N', 8, 2).unwrap();
        assert!(connection.set_slave(0x01).is_ok());
        assert!(
            connection
                .rtu_set_serial_mode(SerialMode::RtuRS232)
                .is_ok()
        );
        assert!(
            connection
                .set_response_timeout(Timeout { sec: 1, usec: 0 })
                .is_ok()
        );
        connection.set_debug(false).unwrap();

        let timeout = connection.get_response_timeout();
        info!("Timout {:?}", timeout);

        assert!(connection.connect().is_ok());
        debug!("Connected");

        ModbusSunSaverConnection { connection: connection }
    }

    fn read_registers_retry(&self, address: u16, num_bit: u16, dest: &mut [u16]) -> Result<usize, RetryError> {
        match Retry::new(
                &mut || self.connection.read_registers(address, num_bit, dest),
                &mut |response| response.is_ok()
            )
            .try(3)
            .wait(100)
            .execute() {
            Ok(response) => Ok(response.unwrap() as usize),
            Err(error) => Err(error),
        }
    }
}

impl SunSaverConnection for ModbusSunSaverConnection {
    fn read_raw_registers(&mut self) -> [u16; 44] {
        let mut response_register = [0u16; 44 as usize];
        let mut num_read_bytes = 0;
        num_read_bytes += self.read_registers_retry(0x08, 22, &mut response_register[0..22])
            .unwrap();
        num_read_bytes += self.read_registers_retry(0x1E, 22, &mut response_register[23..44])
            .unwrap();
        //if num_read_bytes != 44 {
        //    panic!("Failed to read all registers! Required 44 got {}", num_read_bytes);
        //}
        debug!("Read {} bytes", num_read_bytes);
        debug!("read reg 0x08 + 44: {:#x}", response_register.as_hex());

        response_register
    }

    fn read_raw_logged(&mut self) -> [u16; 32 * 16] {
        let mut logged_data = [0u16; (32 * 16) as usize];

        for i in 0..32 {
            let offset: usize = i * 16;
            self.read_registers_retry(
                (0x8000 + offset) as u16,
                16,
                &mut logged_data[offset..offset + 16],
            ).unwrap();
        }

        debug!("logged_data_start");
        for i in (0 as usize)..32 {
            let offset: usize = i * 16;
            debug!("{:#x}", logged_data[offset..(offset + 16)].as_hex());
        }
        debug!("logged_data_end");

        logged_data
    }
}

#[derive(Debug)]
pub struct FileSunSaverConnection {
    file: File,
}

impl FileSunSaverConnection {
    pub fn open(filename: &Path) -> FileSunSaverConnection {
        let file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(filename)
            .unwrap();

        FileSunSaverConnection { file: file }
    }
}

impl SunSaverConnection for FileSunSaverConnection {
    fn read_raw_registers(&mut self) -> [u16; 44] {
        let mut response_register_u8 = [0u8; 88 as usize];
        assert!(self.file.read_exact(&mut response_register_u8).is_ok());

        let response_register_vec_u16: Vec<u16> = response_register_u8
            .chunks(2)
            .map(|items| {
                u16::from_le(((items[0] as u16) << 8) + (items[1] as u16))
            })
            .collect();

        let mut response_register_u16 = [0u16; 44 as usize];
        response_register_u16.clone_from_slice(&response_register_vec_u16);
        response_register_u16
    }

    fn read_raw_logged(&mut self) -> [u16; 32 * 16] {
        unimplemented!();
    }
}
