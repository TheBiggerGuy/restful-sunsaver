use std::convert::From;

use sunsaver::{SunSaverResponse, ChargeState, ArrayFault, LoggedResponse, LoggedResponseDay};

#[derive(Debug, Clone, Serialize)]
pub struct ApiStatusResponse {
    generation: ApiStatusResponseGeneration,
    storage: ApiStatusResponseStorage,
    load: ApiStatusResponseLoad,
    temperature: ApiStatusResponseTemperature,
    faults: ApiStatusResponseFaults,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiStatusResponseGeneration {
    solar_input_voltage_filtered: f32,
    calculated_generation_power: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiStatusResponseStorage {
    battery_voltage_filtered: f32,
    battery_charge_current_filtered: f32,
    battery_charge_power_calculated: f32,
    charge_state: ChargeState,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiStatusResponseLoad {
    load_voltage_filtered: f32,
    load_current_filtered: f32,
    load_power_calculated: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiStatusResponseTemperature {
    heatsink_temperature: i8,
    battery_temperature: i8,
    ambient_temperature: i8,
    remote_temperature: i8,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiStatusResponseFaults {
    array: ArrayFault,
}

impl From<SunSaverResponse> for ApiStatusResponse {
    fn from(response: SunSaverResponse) -> Self {
        let battery_voltage_filtered = response.battery_voltage_filtered();
        let battery_charge_current_filtered = response.battery_charge_current_filtered();
        let load_voltage_filtered = response.load_voltage_filtered();
        let load_current_filtered = response.load_current_filtered();
        let solar_input_voltage_filtered = response.solar_input_voltage_filtered();

        let generation = ApiStatusResponseGeneration {
            solar_input_voltage_filtered: solar_input_voltage_filtered,
            calculated_generation_power: (load_current_filtered + battery_charge_current_filtered) *
                solar_input_voltage_filtered,
        };
        let storage = ApiStatusResponseStorage {
            battery_voltage_filtered: battery_voltage_filtered,
            battery_charge_current_filtered: battery_charge_current_filtered,
            battery_charge_power_calculated: battery_voltage_filtered *
                battery_charge_current_filtered,
            charge_state: response.charge_state(),
        };
        let load = ApiStatusResponseLoad {
            load_voltage_filtered: load_voltage_filtered,
            load_current_filtered: load_current_filtered,
            load_power_calculated: load_voltage_filtered * load_current_filtered,
        };
        let temperature = ApiStatusResponseTemperature {
            heatsink_temperature: response.heatsink_temperature(),
            battery_temperature: response.battery_temperature(),
            ambient_temperature: response.ambient_temperature(),
            remote_temperature: response.remote_temperature(),
        };
        let faults = ApiStatusResponseFaults { array: response.array_fault() };
        ApiStatusResponse {
            generation: generation,
            storage: storage,
            load: load,
            temperature: temperature,
            faults: faults,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiLoggedResponse {
    days: Vec<ApiLoggedDayResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiLoggedDayResponse {
    hourmeter: u32,
    battery_voltage_min: f32,
    battery_voltage_max: f32,
    battery_charge_daily: f32,
    load_charge_daily: f32,
    array_voltage_max: f32,
}

impl From<LoggedResponse> for ApiLoggedResponse {
    fn from(response: LoggedResponse) -> Self {
        let days = response
            .days
            .into_iter()
            .map(ApiLoggedDayResponse::from)
            .collect();
        ApiLoggedResponse { days: days }
    }
}

impl From<LoggedResponseDay> for ApiLoggedDayResponse {
    fn from(response: LoggedResponseDay) -> Self {
        ApiLoggedDayResponse {
            hourmeter: response.hourmeter,
            battery_voltage_min: response.battery_voltage_min(),
            battery_voltage_max: response.battery_voltage_max(),
            battery_charge_daily: response.battery_charge_daily(),
            load_charge_daily: response.load_charge_daily(),
            array_voltage_max: response.array_voltage_max(),
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json;

    use sunsaver::ArrayFault;
    use super::*;

    #[test]
    fn api_statusresponse_faults() {
        let native = ApiStatusResponseFaults { array: ArrayFault::empty() };
        let json = serde_json::to_string(&native).unwrap();
        assert_eq!(
            json,
            "{\"array\":{\"OVERCURENT\":false,\"FETS_SHORTED\":false,\"SOFTWARE_BUGS\":false,\"BATTERY_HVD\":false,\"ARRAY_HVD\":false,\"EEPROM_EDIT\":false,\"RTS_SHORTED\":false,\"RTS_DISCONECTED\":false,\"INTERNAL_TEMP_SENSOR_FAIL\":false}}"
        );
    }
}
