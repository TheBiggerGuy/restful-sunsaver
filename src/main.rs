extern crate clap;
extern crate libmodbus_rs;

use libmodbus_rs::{Modbus, ModbusClient, ModbusRTU, SerialMode};
use clap::{App, Arg};

struct SunSaverResponse {
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
    
    fn battery_voltage_filtered(&self) -> f32 {
        conv_100_2_15_scale!(self.adc_vb_f)
    }

    fn solar_input_voltage_filtered(&self) -> f32 {
        conv_100_2_15_scale!(self.adc_va_f)
    }

    fn load_voltage_filtered(&self) -> f32 {
        conv_100_2_15_scale!(self.adc_vl_f)
    }
}

fn main() {
    let matches = App::new("simple-client")
        .version(env!("CARGO_PKG_VERSION"))
        .about("HTTP RESTful server for SunSaver MPPT ModBus data")
        .author("Guy Taylor <thebiggerguy.co.uk@gmail.com>")
        .arg(Arg::with_name("device")
            .help("Serial device e.g. /dev/ttyUSB0")
            .long("serial_interface")
            .short("d")
            .takes_value(true)
            .required(true))
        .get_matches();

    let serial_interface = matches.value_of("device").unwrap();
    
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
    let mut modbus = Modbus::new_rtu(&serial_interface, 9600, 'N', 8, 2).unwrap();
    assert!(modbus.set_slave(0x01).is_ok());
    assert!(modbus.rtu_set_serial_mode(SerialMode::MODBUS_RTU_RS232).is_ok());

    modbus.set_debug(true).unwrap();
    modbus.connect().unwrap();

    let mut response_register = [0u16; 44 as usize];
    let mut num_read_bytes = 0;
    num_read_bytes += modbus.read_registers(0x08, 22, &mut response_register[0..22]).unwrap();
    num_read_bytes += modbus.read_registers(0x1E, 22, &mut response_register[23..44]).unwrap();
    if num_read_bytes != 44 {
        panic!("Failed to read");
    }

    let response = SunSaverResponse::from_raw_bits(response_register);
    println!("Battery: {}V (filtered)", response.battery_voltage_filtered());
    println!("Solar Input: {}V (filtered)", response.solar_input_voltage_filtered());
    println!("Load: {}V (filtered)", response.load_voltage_filtered());
}
