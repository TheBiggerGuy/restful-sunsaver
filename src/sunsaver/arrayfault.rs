use std::convert::From;

use std::result::Result::{self, Ok};

use serde::ser::{Serialize, SerializeMap, Serializer};

bitflags! {
    pub struct ArrayFault: u16 {
        const OVERCURENT                = 0b0000_0000_0000_0001;
        const FETS_SHORTED              = 0b0000_0000_0000_0010;
        const SOFTWARE_BUGS             = 0b0000_0000_0000_0100;
        const BATTERY_HVD               = 0b0000_0000_0000_1000;
        const ARRAY_HVD                 = 0b0000_0000_0001_0000;
        const EEPROM_EDIT               = 0b0000_0000_0010_0000;
        const RTS_SHORTED               = 0b0000_0000_0100_0000;
        const RTS_DISCONECTED           = 0b0000_0000_1000_0000;
        const INTERNAL_TEMP_SENSOR_FAIL = 0b0000_0001_0000_0000;
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(ARRAY_FAULT_FLAGS.len()))?;
        for flag in ARRAY_FAULT_FLAGS.iter() {
            let is_set = self.contains(*flag);
            map.serialize_entry(&format!("{:?}", flag), &is_set)?;
        }
        map.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn sunsaverresponse_array_fault() {
        assert_eq!(ArrayFault::from(0b0000_0000_0000_0000), ArrayFault::empty());
        assert_eq!(ArrayFault::from(0b0000_0000_0000_0001), ArrayFault::OVERCURENT);
        assert_eq!(ArrayFault::from(0b0000_0000_0000_0010), ArrayFault::FETS_SHORTED);
        assert_eq!(ArrayFault::from(0b0000_0000_0000_0011), ArrayFault::OVERCURENT | ArrayFault::FETS_SHORTED);
    }

    #[test]
    fn sunsaverresponse_array_fault_serialize() {
        let native = ArrayFault::empty();
        let json = serde_json::to_string(&native).unwrap();
        assert_eq!(
            json,
            "{\
             \"OVERCURENT\":false,\
             \"FETS_SHORTED\":false,\
             \"SOFTWARE_BUGS\":false,\
             \"BATTERY_HVD\":false,\
             \"ARRAY_HVD\":false,\
             \"EEPROM_EDIT\":false,\
             \"RTS_SHORTED\":false,\
             \"RTS_DISCONECTED\":false,\
             \"INTERNAL_TEMP_SENSOR_FAIL\":false\
             }"
        );

        let native = ArrayFault::OVERCURENT;
        let json = serde_json::to_string(&native).unwrap();
        assert!(
            json.starts_with(
                "{\"OVERCURENT\":true,\
                 \"FETS_SHORTED\":false,\
                 \"SOFTWARE_BUGS\":false,",
            ),
            json
        );

        let native = ArrayFault::FETS_SHORTED;
        let json = serde_json::to_string(&native).unwrap();
        assert!(
            json.starts_with(
                "{\"OVERCURENT\":false,\
                 \"FETS_SHORTED\":true,\
                 \"SOFTWARE_BUGS\":false,",
            ),
            json
        );

        let native = ArrayFault::OVERCURENT | ArrayFault::FETS_SHORTED;
        let json = serde_json::to_string(&native).unwrap();
        assert!(
            json.starts_with(
                "{\
                 \"OVERCURENT\":true,\
                 \"FETS_SHORTED\":true,\
                 \"SOFTWARE_BUGS\":false,",
            ),
            json
        );
    }
}
