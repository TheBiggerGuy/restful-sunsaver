use crate::{ArrayFault, ChargeState};

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

impl SunSaverResponse {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn from_raw_bits(raw_data: [u16; 44]) -> SunSaverResponse {
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
    use super::*;

    #[cfg_attr(rustfmt, rustfmt_skip)]
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

        assert_eq!(response.battery_voltage_filtered(), 12.869_263);
        assert_eq!(response.solar_input_voltage_filtered(), 13.894_653);

        assert_eq!(response.load_voltage_filtered(), 12.854_004);
        assert_eq!(response.battery_charge_current_filtered(), 0.128_035_89);
        assert_eq!(response.load_current_filtered(), 0.372_028_83);

        assert_eq!(response.heatsink_temperature(), 23);
        assert_eq!(response.battery_temperature(), 23);
        assert_eq!(response.ambient_temperature(), 23);
        assert_eq!(response.remote_temperature(), 25);

        assert_eq!(response.charge_state(), ChargeState::BulkCharge);
        assert!(response.array_fault().is_empty());
    }
}
