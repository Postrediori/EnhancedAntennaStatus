use std::fmt;

use http::Uri;
// use serde_json::Value;
// use ureq::get;

#[macro_export]
macro_rules! copy_string_to_array {
    ($array:tt, $string:expr) => {
        let len = $string.len().min($array.len() - 1);
        $array[..len].copy_from_slice(&$string.chars().collect::<Vec<char>>()[..len]);
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum NetworkMode {
    Lte = 7,
    Wcdma = 2,
    Gsm = 0,
    Unknown = -1,
}

#[derive(Copy, Clone)]
pub struct LteSignalInfo {
    pub rsrq: i64,
    pub rsrp: i64,
    pub sinr: i64,
    pub ca_count: i64,
    pub enb: i64,
    pub id: i64,
    pub pci: i64,
}

#[derive(Copy, Clone)]
pub struct WcdmaSignalInfo {
    pub rscp: i64,
    pub ecio: i64,
    pub nb: i64,
    pub cc: i64,
    pub rnc: i64,
    pub psc: i64,
}

#[derive(Copy, Clone)]
pub enum SignalInfo {
    Lte(LteSignalInfo),
    Wcdma(WcdmaSignalInfo),
    None,
}

#[derive(Copy, Clone)]
pub struct ModemStatus {
    pub mode: NetworkMode,
    pub plmn: [char; 6],
    pub rssi: i64,
    pub cell_id: i64,
    pub signal_info: SignalInfo,
    pub band: [char; 20],

    pub manufacturer: [char; 40],
    pub model: [char; 40],

    pub battery_percent: i64,
    pub battery_status: [char; 20],

    pub device_temp: i64,
    pub battery_temp: i64,

    pub dl: i64,
    pub ul: i64,
}

impl ModemStatus {
    pub fn get_ca_count(&self) -> i64 {
        if let SignalInfo::Lte(lte_info) = self.signal_info {
            lte_info.ca_count
        } else {
            0
        }
    }
    pub fn get_mode(&self) -> String {
        match self.mode {
            NetworkMode::Lte => {
                format!("LTE{}", if self.get_ca_count() > 0 { "-A" } else { "" })
            },
            NetworkMode::Wcdma => "WCDMA".to_string(),
            NetworkMode::Gsm => "GSM".to_string(),
            NetworkMode::Unknown => "Unknown".to_string(),
        }
    }
    pub fn get_plmn(&self) -> String {
        String::from_iter(self.plmn.iter())
    }
    pub fn get_band(&self) -> String {
        let ca_count = self.get_ca_count();
        let band = format!("{}{}",
            String::from_iter(self.band.iter()).trim_matches(char::from(0)).to_string(),
            if ca_count > 0 { format!("+{ca_count}CA") } else { "".to_string() }
        );
        band
    }
    pub fn get_cell_id_hex_and_dec(&self) -> (String, String) {
        let cell_id = self.cell_id.to_string();
        let cell_id_hex = format!("{:X}", self.cell_id);
        (cell_id_hex, cell_id)
    }
    pub fn get_manufacturer_and_model(&self) -> (String, String) {
        (
            String::from_iter(self.manufacturer.iter()).trim_matches(char::from(0)).to_string(),
            String::from_iter(self.model.iter()).trim_matches(char::from(0)).to_string()
        )
    }
    pub fn get_battery_percent_and_status(&self) -> (i64, String) {
        (
            self.battery_percent,
            String::from_iter(self.battery_status.iter()).trim_matches(char::from(0)).to_string()
        )
    }
}

impl fmt::Display for ModemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = self.get_mode();
        
        let plmn = self.get_plmn();

        let band = self.get_band();

        let (cell_id_hex, cell_id) = self.get_cell_id_hex_and_dec();

        let mode_info = match self.signal_info {
            SignalInfo::Wcdma(wcdma_info) => {
                format!("\nRSCP : {}dBm EC/IO : {}dB",
                    wcdma_info.rscp, wcdma_info.ecio)
            },
            SignalInfo::Lte(lte_info) => {
                format!("\nRSRQ/RSRP/SINR : {}dB/{}dBm/{}dB",
                    lte_info.rsrq, lte_info.rsrp, lte_info.sinr)
            },
            _=> { "".to_string() }
        };

        write!(f,
            "Network mode : {}\nRSSI : {} dBm\nPLMN : {}\nBand : {}\nCell ID : {} / {}{}",
            mode, self.rssi,
            plmn, band,
            cell_id_hex, cell_id,
            mode_info)
    }
}

fn get_mode_by_description(s: &str) -> NetworkMode {
    match s {
        "GsmService" => NetworkMode::Gsm,
        "WcdmaService" => NetworkMode::Wcdma,
        "LteService" => NetworkMode::Lte,
        _ => NetworkMode::Unknown,
    }
}

fn get_url_json(host: &str, query: &str) -> Option<serde_json::Value> {
    if let Ok(path) = Uri::builder()
        .scheme("http")
        .authority(host)
        .path_and_query(query)
        .build() {

        let req = ureq::get(&path.to_string());
        match req.call() {
            Ok(response) => {
                if let Ok(json) = response.into_json::<serde_json::Value>() {
                    return Some(json);
                }
            }
            Err(ureq::Error::Status(code, response)) => {
                eprintln!("HTTP error code={} response={}", code, response.status_text());
            }
            Err(e) => {
                eprintln!("HTTP error={}", &e.to_string());
            }
        }
    }
    return None;
}

/*
 * General trait for getting modem info by hostname
 */

pub trait ModemInfoParser {
    fn get_info(host: &str) -> Option<ModemStatus>;
}

/*
 * Utils for Netgear 
 */

pub struct NetgearParser { }

impl NetgearParser {
    fn get_info_json(host: &str) -> Option<serde_json::Value> {
        get_url_json(host, "/model.json?internalapi=1")
    }
    
    fn parse_info_json(json: &serde_json::Value) -> ModemStatus {
        let ca_count = match json["wwan"]["ca"]["SCCcount"].as_i64() {
            Some(i) => {i},
            None => {0},
        };
    
        let mode = get_mode_by_description(json["wwan"]["currentNWserviceType"].as_str().unwrap());
    
        let rssi = json["wwan"]["signalStrength"]["rssi"].as_i64().unwrap();
    
        let plmn_str = format!("{}{}",
            json["wwanadv"]["MCC"].as_str().unwrap(),
            json["wwanadv"]["MNC"].as_str().unwrap());
        let mut plmn = ['\0'; 6];
        copy_string_to_array!(plmn, plmn_str);
    
        let band_str = json["wwanadv"]["curBand"].as_str().unwrap().to_string();
        let mut band = ['\0'; 20];
        copy_string_to_array!(band, band_str);
    
        let cell_id = json["wwanadv"]["cellId"].as_i64().unwrap();
    
        let signal_info: SignalInfo = match mode {
            NetworkMode::Wcdma => {
                let rscp = json["wwan"]["signalStrength"]["rscp"].as_i64().unwrap();
                let ecio = json["wwan"]["signalStrength"]["ecio"].as_i64().unwrap();
    
                let psc = json["wwanadv"]["primScode"].as_i64().unwrap();
    
                let rnc = cell_id >> 16;
                let id = cell_id & 0xFFFF;
    
                let nb = id / 10;
                let cc = id % 10;
    
                SignalInfo::Wcdma(WcdmaSignalInfo {
                    rscp, ecio, nb, cc, rnc, psc
                })
            },
            NetworkMode::Lte => {
                let rsrq = json["wwan"]["signalStrength"]["rsrq"].as_i64().unwrap();
                let rsrp = json["wwan"]["signalStrength"]["rsrp"].as_i64().unwrap();
                let sinr = json["wwan"]["signalStrength"]["sinr"].as_i64().unwrap();
    
                let pci = json["wwanadv"]["primScode"].as_i64().unwrap();
    
                let enb = cell_id >> 8;
                let id = cell_id & 0xFF;
    
                SignalInfo::Lte(LteSignalInfo {
                    rsrq, rsrp, sinr, ca_count, enb, id, pci
                })
            },
            _=> { SignalInfo::None }
        };

        // Modem model
        let manufacturer_str = json["general"]["companyName"].as_str().unwrap().to_string();
        let mut manufacturer = ['\0'; 40];
        copy_string_to_array!(manufacturer, manufacturer_str);

        let model_str = json["general"]["deviceName"].as_str().unwrap().to_string();
        let mut model = ['\0'; 40];
        copy_string_to_array!(model, model_str);

        // Battery info
        let battery_percent = json["power"]["battChargeLevel"].as_i64().unwrap();

        let battery_status_str = json["power"]["battChargeSource"].as_str().unwrap().to_string();
        let mut battery_status = ['\0'; 20];
        copy_string_to_array!(battery_status, battery_status_str);

        // Temperature
        let device_temp = json["general"]["devTemperature"].as_i64().unwrap();
        let battery_temp = json["power"]["batteryTemperature"].as_i64().unwrap();

        // Bandwidth
        let dl =
            if let Some(dl) = json["wwan"]["dataTransferredRx"].as_str() {
                dl.parse::<i64>().unwrap() * 8
            }
            else {
                0
            };
        let ul = 
            if let Some(ul) = json["wwan"]["dataTransferredTx"].as_str() {
                ul.parse::<i64>().unwrap() * 8
            }
            else {
                0
            };

        ModemStatus {
            mode, plmn, rssi, cell_id, signal_info, band,
            manufacturer, model,
            battery_percent, battery_status,
            device_temp, battery_temp,
            dl, ul,
        }
    }    
}

impl ModemInfoParser for NetgearParser {
    fn get_info(host: &str) -> Option<ModemStatus> {
        if let Some(json) = NetgearParser::get_info_json(host) {
            let modem_info = NetgearParser::parse_info_json(&json);

            return Some(modem_info);
        }
        return None;
    }
}
