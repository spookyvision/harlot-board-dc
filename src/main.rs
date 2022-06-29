#![feature(generic_const_exprs)]

//! # Hardware Check
//!
//! This `libstd` program is for the ESP32-C3-DevKitC-02 board.

// Logging macros

use std::sync::{Condvar, Mutex};
use std::{collections::HashMap, num::Wrapping};

mod apa_spi;
mod wifi;

use apa_spi::{Apa, Pixel};
use color_mixer::strip::{Control, Segment, Srgb8, State};
use embedded_svc::io::{Io, Read};
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys as _;
use indexmap::IndexMap;
use log::*;

use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};

use embedded_svc::{
    httpd::{Request, Response},
    storage::RawStorage,
    wifi::*,
};

use embedded_svc::io::Write;

use esp_idf_svc::{
    httpd::{Server, ServerRegistry},
    netif::EspNetifStack,
    nvs::EspDefaultNvs,
    sysloop::EspSysLoopStack,
    wifi::*,
};

/// This configuration is picked up at compile time by `build.rs` from the
/// file `cfg.toml`.
#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

/// Entry point to our application.
///
/// It sets up a Wi-Fi connection to the Access Point given in the
/// configuration, then blinks the RGB LED green/blue.
///
/// If the LED goes solid red, then it was unable to connect to your Wi-Fi
/// network.
///

const FS_NAMESPACE: &'static str = "fs";

struct StdReader<R>(R);

impl<R: Read> std::io::Read for StdReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let res = Read::read(&mut self.0, buf)
            .map_err(|_e| std::io::Error::new(std::io::ErrorKind::Other, "oh no"));
        res
    }
}

#[cfg(not(feature = "experimental"))]
fn httpd(
    mutex: Arc<(Mutex<Option<u32>>, Condvar)>,
    segments: Arc<Mutex<IndexMap<String, Segment>>>,
    sys_start: Instant,
) -> anyhow::Result<Server> {

    use embedded_svc::httpd::{registry::Registry, Body, Handler, Method};
    let read_data = segments.clone();
    let write_data = segments.clone();

    let now_f = move |rr| {
        let dt = Instant::now().duration_since(sys_start).as_millis() as u32;
        let res = format!("{dt}");
        Ok(res.into())
    };

    let read_f = move |rr| {
        let dat = &*read_data.lock().unwrap();
        let ser = serde_json::to_string(dat)?;

        Response::new(200)
            .header("Content-Type", "application/json")
            .body(ser.into())
            .into()
    };

    let write_f = move |mut req: Request| {
        let data = req.as_bytes()?;
        let de: IndexMap<String, Segment> = serde_json::from_slice(&data)?;
        let mut dat = write_data.lock().unwrap();
        *dat = de;

        Ok("ok".into())
    };
    
    fn resp(data: &'static [u8], content_type: &str) -> Result<Response, anyhow::Error> {
        let response = Response::new(200)
            .header("Content-Encoding", "gzip").header("Content-Type", content_type);
        let body = Body::Read(None, Box::new(data));
        response.body(body).into()
    }

    let server = ServerRegistry::new()
//. handler (Handler :: new ("/.gitkeep" , Method :: Get , | _ | { let data = include_bytes ! ("/mnt/c/Users/ace/Documents/GitHub/color-mixer-ws/mixer-dioxus/dist/.gitkeep.gz") ; resp (data . as_slice () , "application/octet-stream") })) ?. handler (Handler :: new ("/assets/dioxus/color-mixer.js" , Method :: Get , | _ | { let data = include_bytes ! ("/mnt/c/Users/ace/Documents/GitHub/color-mixer-ws/mixer-dioxus/dist/assets/dioxus/color-mixer.js.gz") ; resp (data . as_slice () , "text/javascript") })) ?. handler (Handler :: new ("/assets/dioxus/color-mixer_bg.wasm" , Method :: Get , | _ | { let data = include_bytes ! ("/mnt/c/Users/ace/Documents/GitHub/color-mixer-ws/mixer-dioxus/dist/assets/dioxus/color-mixer_bg.wasm.gz") ; resp (data . as_slice () , "application/wasm") })) ?. handler (Handler :: new ("/assets/dioxus/snippets/dioxus-interpreter-js-459fb15b86d869f7/src/interpreter.js" , Method :: Get , | _ | { let data = include_bytes ! ("/mnt/c/Users/ace/Documents/GitHub/color-mixer-ws/mixer-dioxus/dist/assets/dioxus/snippets/dioxus-interpreter-js-459fb15b86d869f7/src/interpreter.js.gz") ; resp (data . as_slice () , "text/javascript") })) ?
. handler (Handler :: new ("/assets/dioxus/color-mixer.js" , Method :: Get , | _ | { let data = include_bytes ! ("/mnt/c/Users/ace/Documents/GitHub/color-mixer-ws/mixer-dioxus/dist/assets/dioxus/color-mixer.js.gz") ; resp (data . as_slice () , "text/javascript") })) ?. handler (Handler :: new ("/assets/dioxus/color-mixer_bg.wasm" , Method :: Get , | _ | { let data = include_bytes ! ("/mnt/c/Users/ace/Documents/GitHub/color-mixer-ws/mixer-dioxus/dist/assets/dioxus/color-mixer_bg.wasm.gz") ; resp (data . as_slice () , "application/wasm") })) ?. handler (Handler :: new ("/assets/dioxus/snippets/dioxus-interpreter-js-459fb15b86d869f7/src/interpreter.js" , Method :: Get , | _ | { let data = include_bytes ! ("/mnt/c/Users/ace/Documents/GitHub/color-mixer-ws/mixer-dioxus/dist/assets/dioxus/snippets/dioxus-interpreter-js-459fb15b86d869f7/src/interpreter.js.gz") ; resp (data . as_slice () , "text/javascript") })) ?


        .handler(Handler::new("/now", Method::Get, now_f))?
        .handler(Handler::new("/data", Method::Get, read_f))?
        .handler(Handler::new("/data", Method::Post, write_f))?;

    #[cfg(esp32s2)]
    let server = httpd_ulp_endpoints(server, mutex)?;

    server.start(&Default::default())
}


#[cfg(feature = "experimental")]
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
// http::{
//         server::{registry::Registry, Response, ServerRegistry},
//         Method,
//     },

#[cfg(feature = "experimental")]
fn httpd(
    mutex: Arc<(Mutex<Option<u32>>, Condvar)>,
    segments: Arc<Mutex<IndexMap<String, Segment>>>,
    sys_start: Instant,
) -> anyhow::Result<esp_idf_svc::http::server::EspHttpServer> {
    use embedded_svc::errors::wrap::WrapError;
    use embedded_svc::http::server::registry::Registry;
    use embedded_svc::http::server::Response;
    use embedded_svc::http::SendStatus;

    let mut server = esp_idf_svc::http::server::EspHttpServer::new(&Default::default())?;

    let read_data = segments.clone();
    let write_data = segments.clone();
    server
        .handle_get("/", |_req, mut resp| {
            resp.set_ok();
            resp.send_str("Hello from Rust!")?;
            Ok(())
        })?
        .handle_get("/now", move |_req, mut resp| {
            let dt = Instant::now().duration_since(sys_start).as_millis() as u32;
            resp.set_ok();
            resp.send_str(&format!("{dt}"))?;
            Ok(())
        })?
        .handle_get("/data", move |_req, mut resp| {
            let dat = &*read_data.lock().unwrap();
            let ser = serde_json::to_string(dat)?;
            resp.set_ok();
            resp.send_str(&ser)?;
            Ok(())
        })?
        .handle_post("/data", move |mut req, mut resp| {
            let reader = req.reader();
            let de: IndexMap<String, Segment> = serde_json::from_reader(StdReader(reader))?;
            let mut dat = write_data.lock().unwrap();
            *dat = de;
            resp.set_ok();
            resp.send_str("ok")?;
            Ok(())
        })?
        .handle_get("/foo", |_req, resp| {
            Result::Err(WrapError("Boo, something happened!").into())
        })?
        .handle_get("/bar", |_req, resp| {
            resp.status(403)
                .status_message("No permissions")
                .send_str("You have no permissions to access this page")?;

            Ok(())
        })?
        .handle_get("/panic", |_req, _resp| panic!("User requested a panic!"))?;

    Ok(server)
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let app_config = CONFIG;

    let mut now = 0;
    let sys_start = Instant::now();

    println!("Hello, world!");

    log::debug!("Hello, log!");
    log::info!("Hello, log!");
    log::warn!("Hello, log!");

    let nvs = Arc::new(esp_idf_svc::nvs::EspDefaultNvs::new()?);
    let mut storage =
        esp_idf_svc::nvs_storage::EspNvsStorage::new_default(nvs.clone(), FS_NAMESPACE, true)?;

    let file_name = "segments.json";

    let mut segments = match storage.len(file_name) {
        Ok(Some(len)) => {
            let mut buf = vec![];
            buf.resize(len, 0u8);
            if let Some((loaded_buf, _)) = storage.get_raw(file_name, &mut buf)? {
                let de: IndexMap<String, Segment> = serde_json::from_slice(loaded_buf)?;
                de
            } else {
                IndexMap::new()
            }
        }
        _ => IndexMap::new(),
    };

    if segments.is_empty() {
        let some_segs = [
            Segment::new(
                1,
                false,
                Srgb8::new(255, 150, 0),
                Srgb8::new(255, 10, 120),
                0,
                100,
            ),
            Segment::new(
                1,
                false,
                Srgb8::new(166, 0, 255),
                Srgb8::new(2, 192, 192),
                1,
                100,
            ),
            Segment::new(
                1,
                false,
                Srgb8::new(20, 200, 141),
                Srgb8::new(200, 176, 20),
                2,
                100,
            ),
            Segment::new(
                1,
                false,
                Srgb8::new(200, 20, 30),
                Srgb8::new(200, 200, 10),
                3,
                100,
            ),
        ];

        segments.extend(some_segs.into_iter().map(|s| (s.to_uuid_string(), s)));
    }

    let segments = Arc::new(Mutex::new(segments));


    log::info!("starting wifi");
    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);

    log::info!("starting wifi harder...");
    let _wifi = wifi::wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        netif_stack.clone(),
        sys_loop_stack.clone(),
        nvs.clone(),
    )?;
    log::info!("ok");

    let mutex = Arc::new((Mutex::new(None), Condvar::new()));

    let httpd = httpd(mutex.clone(), segments.clone(), sys_start)?;
    let mut apa_config = apa_spi::Config::default();
    apa_config.length = 512;
    const LEN: usize = 32;
    let mut apa: Apa = Apa::new(apa_config);
    let moar_chill = 1000;
    let state = State::new(
        [
            Segment::new(
                144,
                false,
                Srgb8::new(255, 150, 0),
                Srgb8::new(255, 30, 20),
                0,
                moar_chill,
            ),
            Segment::new(
                60,
                false,
                Srgb8::new(166, 0, 255),
                Srgb8::new(2, 192, 192),
                1,
                moar_chill,
            ),
            Segment::new(
                30,
                false,
                Srgb8::new(20, 200, 141),
                Srgb8::new(200, 176, 20),
                1,
                moar_chill,
            ),
            Segment::new(
                30,
                false,
                Srgb8::new(200, 20, 30),
                Srgb8::new(200, 200, 10),
                1,
                moar_chill,
            ),
        ]
        .iter()
        .cloned(),
    );

    loop {
        let mut led_start = 0;

        let log_f = |s: String| log::warn!("{s}");
        let log_f = |_s| {};

        for (idx, (_id, seg)) in state.iter().enumerate() {
            let color = seg.color_at(now);
            let segment_color = Pixel::new(color.red, color.green, color.blue, 40);
            for i in led_start..led_start + seg.length() {
                apa.set_pixel(i, segment_color, log_f);
            }
            led_start += seg.length();
            apa.flush();
        }
        std::thread::sleep(std::time::Duration::from_millis(10));

        let dt = Instant::now().duration_since(sys_start);
        now = dt.as_millis() as u32;
    }
}
