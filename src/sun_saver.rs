use std::fs::{OpenOptions, File};
use std::io::Read;
use std::result::Result::{self, Ok, Err};

use libmodbus_rs::{Modbus, ModbusClient, ModbusRTU, SerialMode, Timeout};

use retry::{Retry, RetryError};

use hex_slice::AsHex;

pub trait SunSaverConnection {
    fn read_registers(&mut self) ->  [u16; 44];

    fn read_response(&mut self) -> SunSaverResponse {
        SunSaverResponse::from_raw_bits(self.read_registers())
    }
}

pub struct ModbusSunSaverConnection {
    connection: Modbus,
}

impl ModbusSunSaverConnection {
    pub fn open(device: &str) -> ModbusSunSaverConnection {
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
        debug!("Configuring device {}", device);
        let mut connection = Modbus::new_rtu(device, 9600, 'N', 8, 2).unwrap();
        assert!(connection.set_slave(0x01).is_ok());
        assert!(connection.rtu_set_serial_mode(SerialMode::MODBUS_RTU_RS232).is_ok());
        assert!(connection.set_response_timeout( Timeout { sec: 1, usec: 0 } ).is_ok());
        connection.set_debug(false).unwrap();

        let timeout = connection.get_response_timeout();
        info!("Timout {:?}", timeout);

        assert!(connection.connect().is_ok());
        debug!("Connected");
        
        ModbusSunSaverConnection {
            connection: connection,
        }
    }

    fn read_registers_retry(&self, address: i32, num_bit: i32, dest: &mut [u16]) -> Result<usize, RetryError> {
        match Retry::new(
            &mut || self.connection.read_registers(address, num_bit, dest),
            &mut |response| response.is_ok()
            ).try(3).wait(100).execute() {
            Ok(response) => Ok(response.unwrap() as usize),
            Err(error) => Err(error),
        }
    }
}

impl SunSaverConnection for ModbusSunSaverConnection {
    fn read_registers(&mut self) -> [u16; 44] {
        let mut response_register = [0u16; 44 as usize];
        let mut num_read_bytes = 0;
        num_read_bytes += self.read_registers_retry(0x08, 22, &mut response_register[0..22]).unwrap();
        num_read_bytes += self.read_registers_retry(0x1E, 22, &mut response_register[23..44]).unwrap();
        //if num_read_bytes != 44 {
        //    panic!("Failed to read all registers! Required 44 got {}", num_read_bytes);
        //}
        debug!("Read {} bytes", num_read_bytes);
        debug!("read reg 0x08 + 44: {:#x}", response_register.as_hex());

        response_register
    }
}

#[derive(Debug)]
pub struct FileSunSaverConnection {
    file: File,
}

impl FileSunSaverConnection {
    pub fn open(filename: &str) -> FileSunSaverConnection {
        let file = OpenOptions::new().read(true).write(false).open(filename).unwrap();

        FileSunSaverConnection {
            file: file,
        }
    }
}

impl SunSaverConnection for FileSunSaverConnection {
    fn read_registers(&mut self) -> [u16; 44] {
        let mut response_register_u8 = [0u8; 88 as usize];
        assert!(self.file.read_exact(&mut response_register_u8).is_ok());

        let response_register_vec_u16: Vec<u16> = response_register_u8.chunks(2)
        .map(|items| {
            u16::from_le(((items[0] as u16) << 8) + (items[1] as u16))
        })
        .collect();

        let mut response_register_u16 = [0u16; 44 as usize];
        response_register_u16.clone_from_slice(&response_register_vec_u16);
        response_register_u16
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SunSaverResponse {
    // Adc_vb_f
    // [09][0x0008] (V). battery voltage, filtered.
    // Voltage measured directly at the battery connection on the SunSaver MPPT.
    adc_vb_f: u16,
    // Adc_va_f
    // [10][0x0009] (V). solar input voltage.
    // Va is the terminal voltage of the solar input connection.
    adc_va_f: u16,
    // Adc_vl_f
    // [11][0x000A] (V). load voltage.
    // Vl is the terminal voltage of the load output connection.
    adc_vl_f: u16,
    // Adc_ic_f
    // [12][0x000B] (A). battery charge current, filtered.
    // Charging current to the battery as measured by on-board shunt.
    adc_ic_f: u16,
    // Adc_il_f
    // [13][0x000C] (A). load current, filtered.
    // Load current to the systems loads as measured by on-board shunt.
    adc_il_f: u16,
}

macro_rules! conv_100_2_15_scale {
    ($expression:expr) => (
        (($expression as f32) * 100.0) / 32768.0
    )
}

macro_rules! conv_7916_2_15_scale {
    ($expression:expr) => (
        (($expression as f32) * 79.16) / 32768.0
    )
}

impl SunSaverResponse {
    fn from_raw_bits(raw_data: [u16; 44]) -> SunSaverResponse {
        SunSaverResponse {
            adc_vb_f: raw_data[0],
            adc_va_f: raw_data[1],
            adc_vl_f: raw_data[2],
            adc_ic_f: raw_data[3],
            adc_il_f: raw_data[4],
        }
    }
    
    pub fn battery_voltage_filtered(&self) -> f32 {
        conv_100_2_15_scale!(self.adc_vb_f)
    }

    pub fn solar_input_voltage_filtered(&self) -> f32 {
        conv_100_2_15_scale!(self.adc_va_f)
    }

    pub fn load_voltage_filtered(&self) -> f32 {
        conv_100_2_15_scale!(self.adc_vl_f)
    }

    pub fn battery_charge_current_filtered(&self) -> f32 {
        conv_7916_2_15_scale!(self.adc_ic_f)
    }

    pub fn load_current_filtered(&self) -> f32 {
        conv_7916_2_15_scale!(self.adc_il_f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const DEFAULT_TEST_RAW_BITS: [u16; 44] = [
        0x1079, 0x11c9, 0x1074, 0x0035, 0x009a, 0x0017, 0x0017, 0x0017,
        0x0019, 0x0005, 0x0000, 0x1079, 0x1200, 0x0000, 0x1712, 0x0000,
        0x1712, 0x004e, 0x0001, 0x0000, 0x0e13, 0x0000, 0x0000, 0x0f2a,
        0x0000, 0x0f2a, 0x0000, 0x26b6, 0x0000, 0x0001, 0x000b, 0x0006,
        0x006b, 0x10b1, 0x0123, 0x1640, 0x1009, 0x11f1, 0x004e, 0x002b,
        0x0000, 0x0000, 0x0000, 0x0001
    ];

    #[test]
    fn sunsaverresponse_from_raw_bits() {
        let response = SunSaverResponse::from_raw_bits(DEFAULT_TEST_RAW_BITS);

        assert_eq!(response.adc_vb_f, 0x1079);
        assert_eq!(response.adc_va_f, 0x11c9);
        assert_eq!(response.adc_vl_f, 0x1074);
        assert_eq!(response.adc_ic_f, 0x0035);
        assert_eq!(response.adc_il_f, 0x009a);
    }

    #[test]
    fn sunsaverresponse_from_raw_bits_converted() {
        let response = SunSaverResponse::from_raw_bits(DEFAULT_TEST_RAW_BITS);

        assert_eq!(response.battery_voltage_filtered(), 12.869263);
        assert_eq!(response.solar_input_voltage_filtered(), 13.894653);
        assert_eq!(response.load_voltage_filtered(), 12.854004);
        assert_eq!(response.battery_charge_current_filtered(), 0.12803589);
        assert_eq!(response.load_current_filtered(), 0.37202883);
    }
}
