#![feature(generic_const_exprs)]

//! # Hardware Check
//!
//! This `libstd` program is for the ESP32-C3-DevKitC-02 board.

// Logging macros

use std::num::Wrapping;

mod apa_spi;
mod wifi;

use apa_spi::{Apa, Pixel};
use color_mixer::strip::{Control, Segment, Srgb8, State};
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys as _;
use log::*;

use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};

use embedded_svc::wifi::*;

use esp_idf_svc::{netif::EspNetifStack, nvs::EspDefaultNvs, sysloop::EspSysLoopStack, wifi::*};

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
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    println!("Hello, world!");

    // Start the LED off yellow
    // dkc02: gpio8
    // sth else: gpio7

    // The constant `CONFIG` is auto-generated by `toml_config`.
    let app_config = CONFIG;

    let res: thread::JoinHandle<anyhow::Result<()>> = std::thread::spawn(move || {
        let netif_stack = Arc::new(EspNetifStack::new()?);
        let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
        let default_nvs = Arc::new(EspDefaultNvs::new()?);

        let mut wifi = wifi::wifi(
            app_config.wifi_ssid,
            app_config.wifi_psk,
            netif_stack.clone(),
            sys_loop_stack.clone(),
            default_nvs.clone(),
        );
        Ok(())
    });

    let mut apa_config = apa_spi::Config::default();
    apa_config.length = 512;
    const LEN: usize = 32;
    let mut apa: Apa = Apa::new(apa_config);
    let moar_chill = 16;
    let state = State::new(
        [
            Segment::new(
                144,
                false,
                Srgb8::new(255, 150, 0),
                Srgb8::new(255, 30, 20),
                4100 * moar_chill,
            ),
            Segment::new(
                60,
                false,
                Srgb8::new(166, 0, 255),
                Srgb8::new(2, 192, 192),
                6100 * moar_chill,
            ),
            Segment::new(
                30,
                false,
                Srgb8::new(20, 200, 141),
                Srgb8::new(200, 176, 20),
                5400 * moar_chill,
            ),
            Segment::new(
                30,
                false,
                Srgb8::new(200, 20, 30),
                Srgb8::new(200, 200, 10),
                7100 * moar_chill,
            ),
        ]
        .iter()
        .cloned(),
    );

    // apa.set_pixel(0, Pixel::new(200, 20, 30, 99));
    // apa.set_pixel(1, Pixel::new(200, 20, 30, 101));
    // apa.set_pixel(2, Pixel::new(100, 200, 30, 99));
    // apa.set_pixel(3, Pixel::new(100, 200, 30, 101));

    // apa.set_pixel(200, Pixel::new(100, 0, 0, 101));
    // apa.set_pixel(250, Pixel::new(100, 0, 100, 101));
    // apa.set_pixel(300, Pixel::new(0, 100, 0, 101));
    // apa.set_pixel(350, Pixel::new(0, 100, 150, 101));
    // apa.flush();

    let mut bn: u8 = 1;
    let mut now = 0;
    let sys_start = Instant::now();
    loop {
        // for i in 4..254 {
        //     let i = i as u8;
        //     apa.set_pixel((i) as usize, Pixel::new(i * 10, 0, 10 + i * 2, bn));
        //     apa.set_pixel((i * 2) as usize, Pixel::new(i * 2, 0, 10 + i * 10, bn));
        // }

        bn = (bn + 1) % 110;

        let mut led_start = 0;

        let log_f = |s: String| log::warn!("{s}");
        let log_f = |_s| {};

        for (idx, seg) in state.iter().enumerate() {
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