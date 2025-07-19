#![allow(clippy::similar_names)]

use std::str::FromStr;

use crate::bandwidth_utils::{SIZE_TB, TrafficMode, TrafficStatistics};
use crate::modem_utils::{
    BatteryStatus, DeviceInformation, DeviceTemperature, LteSignalInfo, ModemError,
    ModemInfoParser, ModemStatus, NetworkMode, PlmnStatus, SignalInfo, WcdmaSignalInfo,
};
use crate::network_utils::get_url_json;
use crate::utils::{copy_string_to_array, json_str_as_type};

fn get_mode_by_description(s: &str) -> NetworkMode {
    match s {
        "GsmService" => NetworkMode::Gsm,
        "WcdmaService" => NetworkMode::Wcdma,
        "LteService" => NetworkMode::Lte,
        _ => NetworkMode::Unknown,
    }
}

/*
 * Utils for Netgear
 */

pub struct NetgearParser {}

impl NetgearParser {
    fn get_info_json(host: &str) -> Option<serde_json::Value> {
        get_url_json(host, "/model.json?internalapi=1")
    }

    fn parse_info_json(json: &serde_json::Value) -> ModemStatus {
        let ca_count = json["wwan"]["ca"]["SCCcount"].as_i64().unwrap_or(0);

        let mode = get_mode_by_description(json["wwan"]["currentNWserviceType"].as_str().unwrap());

        let rssi = json["wwan"]["signalStrength"]["rssi"].as_i64().unwrap();

        let plmn_str = format!(
            "{}{}",
            json["wwanadv"]["MCC"].as_str().unwrap(),
            json["wwanadv"]["MNC"].as_str().unwrap()
        );
        let plmn = PlmnStatus::from_str(&plmn_str).expect("Unable to convert PLMN from string");

        let band_str = json["wwanadv"]["curBand"].as_str().unwrap().to_string();
        let mut band = ['\0'; 20];
        copy_string_to_array!(band, band_str);

        let cell_id = json["wwanadv"]["cellId"].as_i64().unwrap();

        let signal_info: SignalInfo = match mode {
            NetworkMode::Wcdma => {
                let rscp = json["wwan"]["signalStrength"]["rscp"].as_i64().unwrap();
                let ecio = json["wwan"]["signalStrength"]["ecio"].as_i64().unwrap();

                let psc = json["wwanadv"]["primScode"].as_i64().unwrap();

                let (rnc, id) = (cell_id >> 16, cell_id & 0xFFFF);

                let (nb, cc) = (id / 10, id % 10);

                SignalInfo::Wcdma(WcdmaSignalInfo {
                    rscp,
                    ecio,
                    nb,
                    cc,
                    rnc,
                    psc,
                })
            }
            NetworkMode::Lte => {
                let rsrq = json["wwan"]["signalStrength"]["rsrq"].as_i64().unwrap();
                let rsrp = json["wwan"]["signalStrength"]["rsrp"].as_i64().unwrap();
                let sinr = json["wwan"]["signalStrength"]["sinr"].as_i64().unwrap();

                let pci = json["wwanadv"]["primScode"].as_i64().unwrap();

                let (enb, id) = (cell_id >> 8, cell_id & 0xFF);

                SignalInfo::Lte(LteSignalInfo {
                    rsrq,
                    rsrp,
                    sinr,
                    ca_count,
                    enb,
                    id,
                    pci,
                })
            }
            _ => SignalInfo::None,
        };

        // Modem model
        let manufacturer_str = json["general"]["companyName"].as_str().unwrap().to_string();
        let model_str = json["general"]["deviceName"].as_str().unwrap().to_string();
        let device_info = DeviceInformation::from(&manufacturer_str, &model_str);

        // Battery info
        let battery_percent = json["power"]["battChargeLevel"].as_i64().unwrap();

        let battery_status_str = json["power"]["battChargeSource"]
            .as_str()
            .unwrap()
            .to_string();
        let mut battery_status = ['\0'; 20];
        copy_string_to_array!(battery_status, battery_status_str);

        let battery_status = BatteryStatus {
            percent: battery_percent,
            status: battery_status,
        };

        // Temperature
        let device_temp = DeviceTemperature {
            device_temp: json["general"]["devTemperature"].as_i64().unwrap(),
            battery_temp: json["power"]["batteryTemperature"].as_i64().unwrap(),
        };

        // Bandwidth
        let dl = if let Some(dl) = json_str_as_type::<i64>(&json["wwan"]["dataTransferredRx"]) {
            if dl <= SIZE_TB { dl * 8 } else { 0 }
        } else {
            0
        };
        let ul = if let Some(ul) = json_str_as_type::<i64>(&json["wwan"]["dataTransferredTx"]) {
            if ul <= SIZE_TB { ul * 8 } else { 0 }
        } else {
            0
        };
        let traffic_statistics = TrafficStatistics { dl, ul };

        ModemStatus {
            mode,
            plmn,
            rssi,
            cell_id,
            signal_info,
            band,
            device_info,
            battery_status: Some(battery_status),
            device_temp: Some(device_temp),
            traffic_statistics: Some(traffic_statistics),
            traffic_mode: TrafficMode::Cumulative,
        }
    }
}

impl ModemInfoParser for NetgearParser {
    fn get_info(host: &str) -> Result<ModemStatus, ModemError> {
        if let Some(json) = NetgearParser::get_info_json(host) {
            let modem_info = NetgearParser::parse_info_json(&json);

            Ok(modem_info)
        } else {
            eprintln!("Cannot access info JSON from host {host}");
            Err(ModemError::HttpConnection)
        }
    }
}
