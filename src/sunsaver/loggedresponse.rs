use std::cmp::{Ord, Ordering};

#[derive(Debug, Clone, Serialize)]
pub struct LoggedResponse {
    pub days: Vec<LoggedResponseDay>,
}

impl LoggedResponse {
    pub fn from_raw_bits(raw_data: [u16; 32 * 16]) -> LoggedResponse {
        let mut days = vec![];
        for i in 0..32 {
            let offset = i * 16;
            let data = clone_into_array(&raw_data[offset..offset+16]);
            let day = LoggedResponseDay::from_raw_bits(data);
            if day.hourmeter == 0x000000 || day.hourmeter == 0xffffff {
                continue;
            }
            days.push(day);
        }
        days.sort();
        LoggedResponse {
            days: days,
        }
    }
}

#[derive(Debug, Clone, Serialize, Eq)]
pub struct LoggedResponseDay {
    pub hourmeter: u32, // u24
    pub alarm_daily: u32, // u24
    vb_min_daily: u16,
    vb_max_daily: u16,
}

impl LoggedResponseDay {
    pub fn from_raw_bits(raw_data: [u16; 16]) -> LoggedResponseDay {
        let hourmeter = u32::from_be((((raw_data[0] as u32) << 16) | (raw_data[1] as u32)) & 0xffffff00);
        let alarm_daily = u32::from_be((((raw_data[1] as u32) << 16) | (raw_data[2] as u32)) & 0x00ffffff);
        LoggedResponseDay {
            hourmeter: hourmeter,
            alarm_daily: alarm_daily,
            vb_min_daily: u16::from_be(raw_data[3]),
            vb_max_daily: u16::from_le(raw_data[4]),
        }
    }

    pub fn battery_voltage_min(&self) -> f32 {
        conv_100_2_15_scale!(self.vb_min_daily)
    }

    pub fn battery_voltage_max(&self) -> f32 {
        conv_100_2_15_scale!(self.vb_max_daily)
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

use std::convert::AsMut;

fn clone_into_array<A, T>(slice: &[T]) -> A
    where A: Sized + Default + AsMut<[T]>,
          T: Clone
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

#[cfg(test)]
mod test {
    use super::*;

    const DEFAULT_TEST_RAW_BITS: [u16; 32 * 16] = [
        0x0000, 0x0000, 0x0000, 0x1014, 0x1204, 0x0053, 0x0012, 0x0000, 0x0000, 0x1a84, 0x00b3, 0x0000, 0x00a3, 0xffff, 0xffff, 0xffff,
        0x2432, 0x0100, 0x0000, 0x101e, 0x11f6, 0x0051, 0x0012, 0x0000, 0x0000, 0x1af2, 0x00b3, 0x0000, 0x0a09, 0xffff, 0xffff, 0xffff,
        0xffff, 0xff00, 0x0000, 0x1008, 0x1206, 0x005b, 0x001a, 0x0000, 0x0000, 0x1b10, 0x00b3, 0x0000, 0x0164, 0xffff, 0xffff, 0xffff,
        0x2402, 0x0100, 0x0000, 0x1011, 0x11fb, 0x0047, 0x001b, 0x0000, 0x0000, 0x1a84, 0x00b4, 0x0000, 0x010f, 0xffff, 0xffff, 0xffff, // test item
        0x244a, 0x0100, 0x0000, 0x1011, 0x1220, 0x004d, 0x0013, 0x0000, 0x0000, 0x1ab4, 0x00b4, 0x0000, 0x0151, 0xffff, 0xffff, 0xffff,
        0x2462, 0x0100, 0x0000, 0x1052, 0x126e, 0x0015, 0x0000, 0x0000, 0x0000, 0x1b79, 0x00b3, 0x0000, 0x01ee, 0xffff, 0xffff, 0xffff,
        0x247a, 0x0100, 0x0000, 0x0f86, 0x1273, 0x003c, 0x00a4, 0x0000, 0x0000, 0x1b05, 0x00b8, 0x0000, 0x0130, 0xffff, 0xffff, 0xffff,
        0x2491, 0x0100, 0x0000, 0x0e9c, 0x1052, 0x00df, 0x0288, 0x0000, 0x0000, 0x1a2e, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x24a9, 0x0100, 0x0000, 0x0e8a, 0x1022, 0x00f2, 0x0127, 0x0000, 0x0000, 0x1a6c, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x24c1, 0x0100, 0x0000, 0x0e96, 0x1024, 0x00b0, 0x00c0, 0x0000, 0x0000, 0x1a2b, 0x0001, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x24d9, 0x0100, 0x0000, 0x0f58, 0x0fee, 0x0034, 0x0016, 0x0000, 0x0000, 0x1984, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x24f1, 0x0100, 0x0000, 0x0f17, 0x10cd, 0x00b7, 0x0067, 0x0000, 0x0000, 0x1a70, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2509, 0x0100, 0x0000, 0x0f56, 0x10fd, 0x00d9, 0x0067, 0x0000, 0x0000, 0x1a5d, 0x0005, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2521, 0x0100, 0x0000, 0x0fad, 0x109d, 0x003a, 0x0061, 0x0000, 0x0000, 0x1a87, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2539, 0x0100, 0x0000, 0x0f8a, 0x114c, 0x00b0, 0x0066, 0x0000, 0x0000, 0x1a6f, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2551, 0x0100, 0x0000, 0x0fb9, 0x1179, 0x00c9, 0x0064, 0x0000, 0x0000, 0x1a7a, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2569, 0x0100, 0x0000, 0x0fe0, 0x11c2, 0x007b, 0x0062, 0x0000, 0x0000, 0x1ad7, 0x0001, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2581, 0x0100, 0x0000, 0x0fd5, 0x1213, 0x00ab, 0x0064, 0x0000, 0x0000, 0x1ac1, 0x0003, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2599, 0x0100, 0x0000, 0x0ffa, 0x120e, 0x00a2, 0x0061, 0x0000, 0x0000, 0x1a3e, 0x0029, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x25b1, 0x0100, 0x0000, 0x1006, 0x10b6, 0x0045, 0x0060, 0x0000, 0x0000, 0x1a1e, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x25c9, 0x0100, 0x0000, 0x0f7d, 0x1214, 0x00db, 0x00ee, 0x0000, 0x0000, 0x1b11, 0x0002, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x25e1, 0x0100, 0x0000, 0x0fd6, 0x1220, 0x0076, 0x002a, 0x0000, 0x0000, 0x1aa3, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x25f9, 0x0100, 0x0000, 0x1036, 0x125d, 0x0064, 0x0000, 0x0000, 0x0000, 0x1a84, 0x0052, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2611, 0x0100, 0x0000, 0x106a, 0x1259, 0x0052, 0x0002, 0x0000, 0x0000, 0x1add, 0x00b7, 0x0000, 0x004e, 0xffff, 0xffff, 0xffff,
        0x2628, 0x0100, 0x0000, 0x1037, 0x1247, 0x0049, 0x0005, 0x0000, 0x0000, 0x1a05, 0x0024, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2641, 0x0100, 0x0000, 0x0f96, 0x126b, 0x0090, 0x009f, 0x0000, 0x0000, 0x1a9e, 0x00de, 0x0000, 0x009f, 0xffff, 0xffff, 0xffff,
        0x2658, 0x0100, 0x0000, 0x1000, 0x10dc, 0x002b, 0x0018, 0x0000, 0x0000, 0x1980, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2671, 0x0100, 0x0000, 0x101e, 0x1198, 0x0053, 0x0000, 0x0000, 0x0000, 0x1a31, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0x2688, 0x0100, 0x0000, 0x1051, 0x127c, 0x0049, 0x0007, 0x0000, 0x0000, 0x1b33, 0x00cd, 0x0000, 0x0085, 0xffff, 0xffff, 0xffff,
        0x26a0, 0x0100, 0x0000, 0x100e, 0x1216, 0x006c, 0x002d, 0x0000, 0x0000, 0x1aca, 0x00b3, 0x0000, 0x0054, 0xffff, 0xffff, 0xffff,
        0x26b8, 0x0100, 0x0000, 0x1009, 0x11f1, 0x004e, 0x002f, 0x0000, 0x0000, 0x1a1a, 0x0000, 0x0000, 0x0000, 0xffff, 0xffff, 0xffff,
        0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
    ];

    #[test]
    fn loggedresponse_from_raw_bits() {
        let response = LoggedResponse::from_raw_bits(DEFAULT_TEST_RAW_BITS);

        // sorted and filtered
        assert_eq!(response.days.len(), 29);
        assert_eq!(response.days[0].hourmeter, 0x010224);
        assert_eq!(response.days[1].hourmeter, 0x010925);

        // correct endianess
        let day = &response.days[0];
        assert_eq!(day.hourmeter, 0x010224);
        assert_eq!(day.alarm_daily, 0x000000);
        assert_eq!(day.vb_min_daily, 0x1110);
        assert_eq!(day.vb_max_daily, 0x11fb);

        assert_eq!(day.battery_voltage_min(), 13.330078);
        assert_eq!(day.battery_voltage_max(), 14.047241);
    }
}