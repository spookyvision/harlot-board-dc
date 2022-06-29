// based on https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs

use std::{sync::Arc, time::Duration};

use anyhow::bail;
use embedded_svc::{wifi::{
    self, AuthMethod, ClientConfiguration, AccessPointConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus,
    Wifi as _,
}, ipv4::{self, DHCPClientSettings}};
use esp_idf_svc::{
    netif::EspNetifStack, nvs::EspDefaultNvs, sysloop::EspSysLoopStack, wifi::EspWifi,
};
use log::info;

pub fn wifi(
    ssid: &str,
    psk: &str,
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> anyhow::Result<Box<EspWifi>> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.len() == 0 {
        anyhow::bail!("missing WiFi name")
    }
    if psk.len() == 0 {
        auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Searching for Wifi network {}", ssid);

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    info!("setting Wifi configuration");
    let hostname = Some(heapless::String::from("harharlot"));
    let ip_conf = Some(ipv4::ClientConfiguration::DHCP(DHCPClientSettings{hostname}));
    wifi.set_configuration(&wifi::Configuration::Mixed(
        ClientConfiguration {
            ssid: ssid.into(),
            password: psk.into(),
            channel,
            // auth_method,
            ip_conf,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;
    
    info!("Wifi: waiting to settle");

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

        info!("Wifi: getting status");
    let status = wifi.get_status();

    if let wifi::Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(_))),
        _,
    ) = status
    {
        info!("Wifi connected!");
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}
