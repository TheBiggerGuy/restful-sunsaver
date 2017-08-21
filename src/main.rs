extern crate clap;
extern crate libmodbus_rs;

use libmodbus_rs::{Modbus, ModbusClient, ModbusRTU};
use clap::{App, Arg};


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
    let mut modbus = Modbus::new_rtu(&serial_interface, 9600, 'N', 8, 2).unwrap();
    modbus.set_slave(0x01).unwrap();

    modbus.set_debug(true).unwrap();
    modbus.connect().unwrap();

    let mut response_register = vec![0u16; 44 as usize];
    modbus.read_registers(0x08, 22, &mut response_register[0..22]).unwrap();
    modbus.read_registers(0x1E, 22, &mut response_register[23..44]).unwrap();

    let adc_vb_f = (response_register[0] as f32) * (100.0 / 32768.0);
    println!("Adc_vb_f={}", adc_vb_f);
}
