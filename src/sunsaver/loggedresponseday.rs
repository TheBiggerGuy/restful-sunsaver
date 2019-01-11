use std::cmp::{Ord, Ordering};

#[derive(Debug, Clone, Serialize, Eq)]
pub struct LoggedResponseDay {
    pub hourmeter: u32,   // u24
    pub alarm_daily: u32, // u24
    vb_min_daily: u16,
    vb_max_daily: u16,
    ahc_daily: u16,
    ahl_daily: u16,
    va_max_daily: u16,
}

impl LoggedResponseDay {
    pub fn from_raw_bits(raw_data: [u16; 16]) -> LoggedResponseDay {
        let hourmeter = u32::from_be(((u32::from(raw_data[0]) << 16) | u32::from(raw_data[1])) & 0xffff_ff00);
        let alarm_daily = u32::from_be(((u32::from(raw_data[1]) << 16) | u32::from(raw_data[2])) & 0x00ff_ffff);
        LoggedResponseDay {
            hourmeter,
            alarm_daily,
            vb_min_daily: raw_data[3],
            vb_max_daily: raw_data[4],
            ahc_daily: raw_data[5],
            ahl_daily: raw_data[6],
            va_max_daily: raw_data[9],
        }
    }

    pub fn battery_voltage_min(&self) -> f32 {
        conv_100_2_15_scale!(self.vb_min_daily)
    }

    pub fn battery_voltage_max(&self) -> f32 {
        conv_100_2_15_scale!(self.vb_max_daily)
    }

    pub fn battery_charge_daily(&self) -> f32 {
        f32::from(self.ahc_daily) * 0.1
    }

    pub fn load_charge_daily(&self) -> f32 {
        f32::from(self.ahl_daily) * 0.1
    }

    pub fn array_voltage_max(&self) -> f32 {
        conv_100_2_15_scale!(self.va_max_daily)
    }
}

impl Ord for LoggedResponseDay {
    fn cmp(&self, other: &LoggedResponseDay) -> Ordering {
        self.hourmeter.cmp(&other.hourmeter)
    }
}

impl PartialOrd for LoggedResponseDay {
    fn partial_cmp(&self, other: &LoggedResponseDay) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for LoggedResponseDay {
    fn eq(&self, other: &LoggedResponseDay) -> bool {
        self.hourmeter == other.hourmeter
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    const DEFAULT_TEST_RAW_BITS: [u16; 16] = [
        0x2402, 0x0100, 0x0000, 0x1011, 0x11fb, 0x0047, 0x001b, 0x0000,
        0x0000, 0x1a84, 0x00b4, 0x0000, 0x010f, 0xffff, 0xffff, 0xffff,
    ];

    #[test]
    fn loggedresponse_from_raw_bits() {
        let day = LoggedResponseDay::from_raw_bits(DEFAULT_TEST_RAW_BITS);

        assert_eq!(day.hourmeter, 0x01_0224);
        assert_eq!(day.alarm_daily, 0x00_0000);
        assert_eq!(day.vb_min_daily, 0x1011);
        assert_eq!(day.vb_max_daily, 0x11fb);

        assert_eq!(day.battery_voltage_min(), 12.551_88);
        assert_eq!(day.battery_voltage_max(), 14.047_241);
        assert_eq!(day.battery_charge_daily(), 7.1);
        assert_eq!(day.load_charge_daily(), 2.7);
        assert_eq!(day.array_voltage_max(), 20.715_332);
    }
}
