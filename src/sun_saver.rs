use std::fs::{OpenOptions, File};
use std::io::Read;

use libmodbus_rs::{Modbus, ModbusClient, ModbusRTU, SerialMode};

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
        let mut connection = Modbus::new_rtu(device, 9600, 'N', 8, 2).unwrap();
        assert!(connection.set_slave(0x01).is_ok());
        assert!(connection.rtu_set_serial_mode(SerialMode::MODBUS_RTU_RS232).is_ok());

        connection.set_debug(true).unwrap();
        connection.connect().unwrap();
        
        ModbusSunSaverConnection {
            connection: connection,
        }
    }
}

impl SunSaverConnection for ModbusSunSaverConnection {
    fn read_registers(&mut self) -> [u16; 44] {
        let mut response_register = [0u16; 44 as usize];
        let mut num_read_bytes = 0;
        num_read_bytes += self.connection.read_registers(0x08, 22, &mut response_register[0..22]).unwrap();
        num_read_bytes += self.connection.read_registers(0x1E, 22, &mut response_register[23..44]).unwrap();
        if num_read_bytes != 44 {
            panic!("Failed to read");
        }

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
}

macro_rules! conv_100_2_15_scale {
    ($expression:expr) => (
        (($expression as f32) * 100.0) / 32768.0
    )
}

impl SunSaverResponse {
    fn from_raw_bits(raw_data: [u16; 44]) -> SunSaverResponse {
        SunSaverResponse {
            adc_vb_f: raw_data[0],
            adc_va_f: raw_data[1],
            adc_vl_f: raw_data[2],
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
}