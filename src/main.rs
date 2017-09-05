#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
extern crate libmodbus_rs;
extern crate iron;
extern crate router;
extern crate staticfile;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::fs;
use std::path::Path;
use std::os::unix::fs::FileTypeExt;
use std::sync::{Arc, Mutex};
use std::convert::From;

use clap::{App, Arg};

use iron::Iron;
use iron::prelude::*;
use iron::status;
use iron::middleware::Handler;
use router::Router;
use staticfile::Static;

mod sun_saver;
use sun_saver::{SunSaverConnection, FileSunSaverConnection, ModbusSunSaverConnection, SunSaverResponse};

#[derive(Debug, Clone, Serialize)]
struct ApiResponse {
    battery_voltage_filtered: f32,
    solar_input_voltage_filtered: f32,
    load_voltage_filtered: f32,
}

impl From<SunSaverResponse> for ApiResponse {
    fn from(response: SunSaverResponse) -> Self {
        ApiResponse {
            battery_voltage_filtered: response.battery_voltage_filtered(),
            solar_input_voltage_filtered: response.solar_input_voltage_filtered(),
            load_voltage_filtered: response.load_voltage_filtered(),
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

fn main() {
    assert!(env_logger::init().is_ok());

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

    let metadata = fs::metadata(serial_interface).unwrap();
    let connection: Box<SunSaverConnection> = if metadata.file_type().is_socket() {
        info!("Device is a socket. Using Modbus");
        Box::new(ModbusSunSaverConnection::open(serial_interface))
    } else {
        info!("Device is not a socket. Using File");
        Box::new(FileSunSaverConnection::open(serial_interface))
    };
    
    let api_handler = ApiHandler::new(connection);

    let mut router = Router::new();
    router.get("/", Static::new(Path::new("web")), "index");
    router.get("/api", api_handler, "api");

    Iron::new(router).http("0.0.0.0:4000").unwrap();
}
