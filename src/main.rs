#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate clap;
extern crate libmodbus_rs;
extern crate iron;
extern crate router;
extern crate staticfile;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate ctrlc;
extern crate retry;
extern crate hex_slice;
#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate bitflags;

use std::fs;
use std::path::Path;
use std::os::unix::fs::FileTypeExt;
use std::sync::{Arc, Mutex};
use std::convert::From;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::{App, Arg};

use iron::Iron;
use iron::prelude::*;
use iron::status;
use iron::middleware::Handler;
use router::Router;
use staticfile::Static;

mod sun_saver;
use sun_saver::{SunSaverConnection, FileSunSaverConnection, ModbusSunSaverConnection, SunSaverResponse, ChargeState, ArrayFault};

#[derive(Debug, Clone, Serialize)]
struct ApiResponse {
    generation: ApiResponseGeneration,
    storage: ApiResponseStorage,
    load: ApiResponseLoad,
    temperature: ApiResponseTemperature,
    faults: ApiResponseFaults,
}

#[derive(Debug, Clone, Serialize)]
struct ApiResponseGeneration {
    solar_input_voltage_filtered: f32,
    calculated_generation_power: f32,
}

#[derive(Debug, Clone, Serialize)]
struct ApiResponseStorage {
    battery_voltage_filtered: f32,
    battery_charge_current_filtered: f32,
    battery_charge_power_calculated: f32,
    charge_state: ChargeState,
}

#[derive(Debug, Clone, Serialize)]
struct ApiResponseLoad {
    load_voltage_filtered: f32,
    load_current_filtered: f32,
    load_power_calculated: f32,
}

#[derive(Debug, Clone, Serialize)]
struct ApiResponseTemperature {
    heatsink_temperature: i8,
    battery_temperature: i8,
    ambient_temperature: i8,
    remote_temperature: i8,
}

#[derive(Debug, Clone, Serialize)]
struct ApiResponseFaults {
    array_fault: ArrayFault,
}

impl From<SunSaverResponse> for ApiResponse {
    fn from(response: SunSaverResponse) -> Self {
        let battery_voltage_filtered = response.battery_voltage_filtered();
        let battery_charge_current_filtered = response.battery_charge_current_filtered();
        let load_voltage_filtered = response.load_voltage_filtered();
        let load_current_filtered = response.load_current_filtered();
        let solar_input_voltage_filtered = response.solar_input_voltage_filtered();
        
        let generation = ApiResponseGeneration {
            solar_input_voltage_filtered: solar_input_voltage_filtered,
            calculated_generation_power: (load_current_filtered + battery_charge_current_filtered) * solar_input_voltage_filtered,
        };
        let storage = ApiResponseStorage {
            battery_voltage_filtered: battery_voltage_filtered,
            battery_charge_current_filtered: battery_charge_current_filtered,
            battery_charge_power_calculated: battery_voltage_filtered * battery_charge_current_filtered,
            charge_state: response.charge_state(),
        };
        let load = ApiResponseLoad {
            load_voltage_filtered: load_voltage_filtered,
            load_current_filtered: load_current_filtered,
            load_power_calculated: load_voltage_filtered * load_current_filtered,
        };
        let temperature = ApiResponseTemperature {
            heatsink_temperature: response.heatsink_temperature(),
            battery_temperature: response.battery_temperature(),
            ambient_temperature: response.ambient_temperature(),
            remote_temperature: response.remote_temperature(),
        };
        let faults = ApiResponseFaults {
            array_fault: response.array_fault(),
        };
        ApiResponse {
            generation: generation,
            storage: storage,
            load: load,
            temperature: temperature,
            faults: faults,
        }
    }
}

#[derive(Clone)]
struct ApiHandler {
    connection: Arc<Mutex<Box<SunSaverConnection>>>,
}

impl ApiHandler {
    fn new(connection: Box<SunSaverConnection>) -> ApiHandler {
        ApiHandler {
            connection: Arc::new(Mutex::new(connection)),
        }
    }
}

unsafe impl Send for ApiHandler {}
unsafe impl Sync for ApiHandler {}

impl Handler for ApiHandler {
    fn handle(&self, _req: &mut Request) -> IronResult<Response> {
        let connection = self.connection.clone();
        let mut unlocked_connection = connection.lock().unwrap();
        let a = ApiResponse::from(unlocked_connection.read_response());
        let b = serde_json::to_string_pretty(&a).unwrap();
        Ok(Response::with((status::Ok, b)))
    }
}

fn is_socket(path: &str) -> bool {
    let metadata = fs::metadata(path).unwrap();
    let file_type = metadata.file_type();
    debug!("is_socket for {} had metadata {:?}", path, metadata);
    debug!("is_socket for {} is_block_device: {}, is_char_device: {}, is_fifo: {}, is_socket: {}", path, file_type.is_block_device(), file_type.is_char_device(), file_type.is_fifo(), file_type.is_socket());
    metadata.file_type().is_char_device()
}

fn main() {
    assert!(pretty_env_logger::init().is_ok());

    let matches = App::new("simple-client")
        .version(env!("CARGO_PKG_VERSION"))
        .about("HTTP RESTful server for SunSaver MPPT ModBus data")
        .author("Guy Taylor <thebiggerguy.co.uk@gmail.com>")
        .arg(Arg::with_name("device")
            .help("Serial device e.g. /dev/ttyUSB0")
            .long("device")
            .short("d")
            .takes_value(true)
            .required(true))
        .get_matches();

    let serial_interface = matches.value_of("device").unwrap();

    let mut connection: Box<SunSaverConnection> = if is_socket(serial_interface) {
        info!("Device is a socket. Using Modbus");
        Box::new(ModbusSunSaverConnection::open(serial_interface))
    } else {
        info!("Device is not a socket. Using File");
        Box::new(FileSunSaverConnection::open(serial_interface))
    };

    debug!("Response: {:?}", connection.read_response());
    
    let api_handler = ApiHandler::new(connection);

    let mut router = Router::new();
    router.get("/", Static::new(Path::new("web")), "index");
    router.get("/api", api_handler, "api");


    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        info!("Cought Ctrl-C");
    }).expect("Error setting Ctrl-C handler");
    
    info!("Starting server ...");
    let bind_address = format!("0.0.0.0:{}", option_env!("PORT").unwrap_or("8080"));
    let mut listening = Iron::new(router).http(&bind_address).unwrap();
    info!("Started server {}", bind_address);
    
    info!("Use Ctrl-C to stop");
    while running.load(Ordering::SeqCst) {}
    listening.close().unwrap();
}
