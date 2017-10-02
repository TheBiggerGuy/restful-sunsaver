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
use std::sync::atomic::{AtomicBool, Ordering};

use clap::{App, Arg};

use iron::Iron;
use iron::prelude::*;
use iron::status;
use iron::middleware::Handler;
use iron::headers::{AccessControlAllowMethods, AccessControlAllowOrigin};
use iron::method::Method;
use iron::mime::Mime;
use router::Router;
use staticfile::Static;

mod sunsaver_connection;
use sunsaver_connection::{SunSaverConnection, FileSunSaverConnection, ModbusSunSaverConnection};
mod sunsaver;
use sunsaver::{ChargeState, ArrayFault, LoggedResponseDay};
mod api;
use api::*;

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
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        debug!("{:?}", req.url);

        let mut response = Response::new();
        response.headers.set(AccessControlAllowMethods(vec![Method::Get]));
        response.headers.set(AccessControlAllowOrigin::Any);
        let mime: Mime = "application/json".parse().unwrap();
        response = response.set((mime));

        let path = req.url.path();
        let last_path = path.clone().pop().unwrap();
        match last_path {
            "status" => {
                let connection = self.connection.clone();
                let mut unlocked_connection = connection.lock().unwrap();
                let a = ApiStatusResponse::from(unlocked_connection.read_status());
                let b = serde_json::to_string_pretty(&a).unwrap();
                response = response.set((status::Ok, b));
            },
            "logged" => {
                let connection = self.connection.clone();
                let mut unlocked_connection = connection.lock().unwrap();
                let a = ApiLoggedResponse::from(unlocked_connection.read_logged());
                let b = serde_json::to_string_pretty(&a).unwrap();
                response = response.set((status::Ok, b));
            },
            _ => {
                response = response.set((status::NotFound, String::new()));
            },
        };

        Ok(response)
    }
}

fn is_socket(path: &str) -> bool {
    let metadata = fs::metadata(path).unwrap();
    let file_type = metadata.file_type();
    debug!("is_socket for {} had metadata {:?}", path, metadata);
    debug!("is_socket for {} is_block_device: {}, is_char_device: {}, is_fifo: {}, is_socket: {}", path, file_type.is_block_device(), file_type.is_char_device(), file_type.is_fifo(), file_type.is_socket());
    metadata.file_type().is_char_device()
}

static CLI_ARG_DEVICE: &'static str = "DEVICE";
static CLI_ARG_PORT: &'static str = "PORT";

fn is_port_number(v: String) -> Result<(), String> {
    v.parse::<u16>().map(|_| ()).map_err(|_| format!("Invalid port number: {}", v))
}

fn main() {
    assert!(pretty_env_logger::init().is_ok());

    let matches = App::new("simple-client")
        .version(env!("CARGO_PKG_VERSION"))
        .about("HTTP RESTful server for SunSaver MPPT ModBus data")
        .author("Guy Taylor <thebiggerguy.co.uk@gmail.com>")
        .arg(Arg::with_name(CLI_ARG_DEVICE)
            .help("Serial device e.g. /dev/ttyUSB0")
            .long("device")
            .short("d")
            .takes_value(true)
            .empty_values(false)
            .required(true)
            )
        .arg(Arg::with_name(CLI_ARG_PORT)
            .help("HTTP server port")
            .long("port")
            .short("p")
            .takes_value(true)
            .empty_values(false)
            .required(false)
            .default_value("8080")
            .validator(is_port_number)
            )
        .get_matches();

    let serial_interface = matches.value_of(CLI_ARG_DEVICE).unwrap();
    let port_number = matches.value_of(CLI_ARG_PORT).unwrap().parse::<u16>().unwrap();

    let connection: Box<SunSaverConnection> = if is_socket(serial_interface) {
        info!("Device is a socket. Using Modbus");
        Box::new(ModbusSunSaverConnection::open(serial_interface))
    } else {
        info!("Device is not a socket. Using File");
        Box::new(FileSunSaverConnection::open(serial_interface))
    };

    let api_handler = ApiHandler::new(connection);
    let static_handler = Static::new(Path::new("web"));

    let mut router = Router::new();
    router.get("/", static_handler.clone(), "index");
    router.get("/:filepath(*)", static_handler.clone(), "static");
    router.get("/api/v1/status", api_handler.clone(), "api_v1_status");
    router.get("/api/v1/logged", api_handler.clone(), "api_v2_logger");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        info!("Cought Ctrl-C");
    }).expect("Error setting Ctrl-C handler");
    
    info!("Starting server ...");
    let bind_address = format!("0.0.0.0:{}", port_number);
    let mut listening = Iron::new(router).http(&bind_address).unwrap();
    info!("Started server {}", bind_address);
    
    info!("Use Ctrl-C to stop");
    while running.load(Ordering::SeqCst) {}
    listening.close().unwrap();
}
