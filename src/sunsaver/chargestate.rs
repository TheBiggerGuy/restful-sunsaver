use std::convert::From;

use enum_primitive::FromPrimitive;

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

#[cfg(test)]
mod test {
    use super::*;

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
}