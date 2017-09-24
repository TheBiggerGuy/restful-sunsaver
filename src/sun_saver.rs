use std::fs::{OpenOptions, File};
use std::io::Read;
use std::result::Result::{self, Ok, Err};

use libmodbus_rs::{Modbus, ModbusClient, ModbusRTU, SerialMode, Timeout};

use retry::{Retry, RetryError};

use hex_slice::AsHex;

use enum_primitive::FromPrimitive;

use serde::ser::{Serialize, Serializer, SerializeMap};

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
    // T_hs
    // [14][0x000D] (C). Heatsink Temperature.
    // Sunsaver MPPT Heatsink temperature. Reported in degrees C.
    t_hs: u16,
    // T_batt
    // [15][0x000E] (C). Battery Temperature.
    // Battery temperature as measured by the ambient temperature sensor or the optional RTS (if connected).
    // Reported in degrees C.
    t_batt: u16,
    // T_amb
    // [16][0x000F] (C). Ambient Temperature.
    // Ambient temperature as measured by the ambient temperature sensor. Reported in degrees C.
    t_amb: u16,
    // T_rts
    // [17][0x0010] (C). RTS Temperature.
    // Temperature as measured by the optional Remote Temperature Sensor(RTS). Reported in degrees C.
    t_rts: u16,
    // Charge_state
    // [18][0x0011] ( ).
    // Reports the charge state.
    charge_state: u16,
    // Array_fault
    // [19][0x0012] (bit-field). Solar input self-diagnostic faults.
    // Reports faults identified by self diagnostics. Each bit corresponds to a specific fault.
    array_fault: u16,
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

enum_from_primitive! {
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ChargeState {
    Start = 0,
    NightCheck = 1,
    Disconnect = 2,
    Night = 3,
    Fault = 4,
    BulkCharge = 5,
    Absorption = 6,
    Float = 7,
    Equalize = 8,
}
}

impl From<u16> for ChargeState {
    fn from(val: u16) -> ChargeState {
        ChargeState::from_u16(val).expect("Value does not match documented enum values")
    }
}

impl From<ChargeState> for u16 {
    fn from(val: ChargeState) -> u16 {
        val as u16
    }
}

bitflags! {
    pub struct ArrayFault: u16 {
        const OVERCURENT                = 0b00000000_00000001;
        const FETS_SHORTED              = 0b00000000_00000010;
        const SOFTWARE_BUGS             = 0b00000000_00000100;
        const BATTERY_HVD               = 0b00000000_00001000;
        const ARRAY_HVD                 = 0b00000000_00010000;
        const EEPROM_EDIT               = 0b00000000_00100000;
        const RTS_SHORTED               = 0b00000000_01000000;
        const RTS_DISCONECTED           = 0b00000000_10000000;
        const INTERNAL_TEMP_SENSOR_FAIL = 0b00000001_00000000;
    }
}
const ARRAY_FAULT_FLAGS: [ArrayFault; 9] = [
                                ArrayFault::OVERCURENT,
                                ArrayFault::FETS_SHORTED,
                                ArrayFault::SOFTWARE_BUGS,
                                ArrayFault::BATTERY_HVD,
                                ArrayFault::ARRAY_HVD,
                                ArrayFault::EEPROM_EDIT,
                                ArrayFault::RTS_SHORTED,
                                ArrayFault::RTS_DISCONECTED,
                                ArrayFault::INTERNAL_TEMP_SENSOR_FAIL,
                              ];

impl From<u16> for ArrayFault {
    fn from(val: u16) -> ArrayFault {
        ArrayFault::from_bits(val).expect("Value does not match documented bit fields")
    }
}

impl Serialize for ArrayFault {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut map = serializer.serialize_map(Some(ARRAY_FAULT_FLAGS.len()))?;
        for flag in ARRAY_FAULT_FLAGS.iter() {
            let is_set = self.contains(*flag);
            map.serialize_entry(&format!("{:?}", flag), &is_set)?;
        }
        map.end()
    }
}

impl SunSaverResponse {
    fn from_raw_bits(raw_data: [u16; 44]) -> SunSaverResponse {
        SunSaverResponse {
            adc_vb_f: raw_data[0],
            adc_va_f: raw_data[1],
            adc_vl_f: raw_data[2],
            adc_ic_f: raw_data[3],
            adc_il_f: raw_data[4],
            t_hs:     raw_data[5],
            t_batt:   raw_data[6],
            t_amb:    raw_data[7],
            t_rts:    raw_data[8],
            charge_state: raw_data[9],
            array_fault: raw_data[10],
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

    pub fn heatsink_temperature(&self) -> i8 {
        self.t_hs as i8
    }

    pub fn battery_temperature(&self) -> i8 {
        self.t_batt as i8
    }

    pub fn ambient_temperature(&self) -> i8 {
        self.t_amb as i8
    }

    pub fn remote_temperature(&self) -> i8 {
        self.t_rts as i8
    }

    pub fn charge_state(&self) -> ChargeState {
        self.charge_state.into()
    }

    pub fn array_fault(&self) -> ArrayFault {
        self.array_fault.into()
    }
}

#[cfg(test)]
mod test {
    use serde_json;
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

        assert_eq!(response.t_hs, 0x0017);
        assert_eq!(response.t_batt, 0x0017);
        assert_eq!(response.t_amb, 0x0017);
        assert_eq!(response.t_rts, 0x0019);

        assert_eq!(response.charge_state, 0x0005);
        assert_eq!(response.array_fault, 0x0000);
    }

    #[test]
    fn sunsaverresponse_from_raw_bits_converted() {
        let response = SunSaverResponse::from_raw_bits(DEFAULT_TEST_RAW_BITS);

        assert_eq!(response.battery_voltage_filtered(), 12.869263);
        assert_eq!(response.solar_input_voltage_filtered(), 13.894653);

        assert_eq!(response.load_voltage_filtered(), 12.854004);
        assert_eq!(response.battery_charge_current_filtered(), 0.12803589);
        assert_eq!(response.load_current_filtered(), 0.37202883);

        assert_eq!(response.heatsink_temperature(), 23);
        assert_eq!(response.battery_temperature(), 23);
        assert_eq!(response.ambient_temperature(), 23);
        assert_eq!(response.remote_temperature(), 25);

        assert_eq!(response.charge_state(), ChargeState::BulkCharge);
        assert!(response.array_fault().is_empty());
    }

    #[test]
    fn sunsaverresponse_charge_state() {
        assert_eq!(ChargeState::from(0u16), ChargeState::Start);
        assert_eq!(ChargeState::from(1u16), ChargeState::NightCheck);
        assert_eq!(ChargeState::from(2u16), ChargeState::Disconnect);
        assert_eq!(ChargeState::from(3u16), ChargeState::Night);
        assert_eq!(ChargeState::from(4u16), ChargeState::Fault);
        assert_eq!(ChargeState::from(5u16), ChargeState::BulkCharge);
        assert_eq!(ChargeState::from(6u16), ChargeState::Absorption);
        assert_eq!(ChargeState::from(7u16), ChargeState::Float);
        assert_eq!(ChargeState::from(8u16), ChargeState::Equalize);

        assert_eq!(0u16, ChargeState::Start as u16);
        assert_eq!(1u16, ChargeState::NightCheck as u16);
        assert_eq!(3u16, ChargeState::Night as u16);
        assert_eq!(2u16, ChargeState::Disconnect as u16);
        assert_eq!(4u16, ChargeState::Fault as u16);
        assert_eq!(5u16, ChargeState::BulkCharge as u16);
        assert_eq!(6u16, ChargeState::Absorption as u16);
        assert_eq!(7u16, ChargeState::Float as u16);
        assert_eq!(8u16, ChargeState::Equalize as u16);
    }

    #[test]
    fn sunsaverresponse_array_fault() {
        assert_eq!(ArrayFault::from(0b0000000000000000), ArrayFault::empty());
        assert_eq!(ArrayFault::from(0b0000000000000001), ArrayFault::OVERCURENT);
        assert_eq!(ArrayFault::from(0b0000000000000010), ArrayFault::FETS_SHORTED);
        assert_eq!(ArrayFault::from(0b0000000000000011), ArrayFault::OVERCURENT | ArrayFault::FETS_SHORTED);
    }

    #[test]
    fn sunsaverresponse_array_fault_serialize() {
        let native = ArrayFault::empty();
        let json = serde_json::to_string(&native).unwrap();
        assert_eq!(json, "{\"OVERCURENT\":false,\"FETS_SHORTED\":false,\"SOFTWARE_BUGS\":false,\"BATTERY_HVD\":false,\"ARRAY_HVD\":false,\"EEPROM_EDIT\":false,\"RTS_SHORTED\":false,\"RTS_DISCONECTED\":false,\"INTERNAL_TEMP_SENSOR_FAIL\":false}");

        let native = ArrayFault::OVERCURENT;
        let json = serde_json::to_string(&native).unwrap();
        assert!(json.starts_with("{\"OVERCURENT\":true,\"FETS_SHORTED\":false,\"SOFTWARE_BUGS\":false,"), json);

        let native = ArrayFault::FETS_SHORTED;
        let json = serde_json::to_string(&native).unwrap();
        assert!(json.starts_with("{\"OVERCURENT\":false,\"FETS_SHORTED\":true,\"SOFTWARE_BUGS\":false,"), json);

        let native = ArrayFault::OVERCURENT | ArrayFault::FETS_SHORTED;
        let json = serde_json::to_string(&native).unwrap();
        assert!(json.starts_with("{\"OVERCURENT\":true,\"FETS_SHORTED\":true,\"SOFTWARE_BUGS\":false,"), json);
    }
}
