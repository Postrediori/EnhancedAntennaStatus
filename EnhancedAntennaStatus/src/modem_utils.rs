use std::fmt::{self, Display};
use std::str::FromStr;

use crate::bandwidth_utils::{TrafficMode, TrafficStatistics};
use crate::utils::copy_string_to_array;

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
pub struct PlmnStatus {
    pub plmn: [char; 6],
}

impl Display for PlmnStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.plmn.iter().collect::<String>())
    }
}

impl FromStr for PlmnStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut plmn = ['\0'; 6];
        copy_string_to_array!(plmn, s);
        Ok(Self { plmn })
    }
}

#[derive(Copy, Clone)]
pub struct BatteryStatus {
    pub percent: i64,
    pub status: [char; 20],
}

impl BatteryStatus {
    fn get_battery_percent_and_status(&self) -> (i64, String) {
        (
            self.percent,
            self.status
                .iter()
                .collect::<String>()
                .trim_matches(char::from(0))
                .to_string(),
        )
    }
}

#[derive(Copy, Clone)]
pub struct DeviceTemperature {
    pub device_temp: i64,
    pub battery_temp: i64,
}

#[derive(Copy, Clone)]
pub struct DeviceInformation {
    pub manufacturer: [char; 40],
    pub model: [char; 40],
}

impl DeviceInformation {
    pub fn from(manufacturer_str: &str, model_str: &str) -> Self {
        let mut manufacturer = ['\0'; 40];
        copy_string_to_array!(manufacturer, manufacturer_str);

        let mut model = ['\0'; 40];
        copy_string_to_array!(model, model_str);

        Self {
            manufacturer,
            model,
        }
    }
    pub fn get_manufacturer_and_model(&self) -> (String, String) {
        (
            self.manufacturer
                .iter()
                .collect::<String>()
                .trim_matches(char::from(0))
                .to_string(),
            self.model
                .iter()
                .collect::<String>()
                .trim_matches(char::from(0))
                .to_string(),
        )
    }
}

#[derive(Copy, Clone)]
pub struct ModemStatus {
    pub mode: NetworkMode,
    pub plmn: PlmnStatus,
    pub rssi: i64,
    pub cell_id: i64,
    pub signal_info: SignalInfo,
    pub band: [char; 20],

    pub device_info: DeviceInformation,
    pub battery_status: Option<BatteryStatus>,
    pub device_temp: Option<DeviceTemperature>,
    pub traffic_statistics: Option<TrafficStatistics>,
    pub traffic_mode: TrafficMode,
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
            }
            NetworkMode::Wcdma => "WCDMA".to_string(),
            NetworkMode::Gsm => "GSM".to_string(),
            NetworkMode::Unknown => "Unknown".to_string(),
        }
    }
    pub fn get_plmn(&self) -> String {
        self.plmn.to_string()
    }
    pub fn get_band(&self) -> String {
        let ca_count = self.get_ca_count();
        let band = format!(
            "{}{}",
            self.band
                .iter()
                .collect::<String>()
                .trim_matches(char::from(0)),
            if ca_count > 0 {
                format!("+{ca_count}CA")
            } else {
                String::new()
            }
        );
        band
    }
    pub fn get_cell_id_hex_and_dec(&self) -> (String, String) {
        let cell_id = self.cell_id.to_string();
        let cell_id_hex = format!("{:X}", self.cell_id);
        (cell_id_hex, cell_id)
    }
    pub fn get_battery_percent_and_status(&self) -> Option<(i64, String)> {
        self.battery_status
            .map(|battery_status| battery_status.get_battery_percent_and_status())
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
                format!(
                    "\nRSCP : {}dBm EC/IO : {}dB",
                    wcdma_info.rscp, wcdma_info.ecio
                )
            }
            SignalInfo::Lte(lte_info) => {
                format!(
                    "\nRSRQ/RSRP/SINR : {}dB/{}dBm/{}dB",
                    lte_info.rsrq, lte_info.rsrp, lte_info.sinr
                )
            }
            SignalInfo::None => String::new(),
        };

        write!(
            f,
            "Network mode : {}\nRSSI : {} dBm\nPLMN : {}\nBand : {}\nCell ID : {} / {}{}",
            mode, self.rssi, plmn, band, cell_id_hex, cell_id, mode_info
        )
    }
}

/// Modem Error
#[derive(Clone, Copy, Debug)]
pub enum ModemError {
    /// Low-level HTTP connection error
    HttpConnection,
    /// Resource access error
    Access,
    /// Parsing of data error
    DataParsing,
    /// All other errors
    Unknown,
}

/*
 * General trait for getting modem info by hostname
 */

pub trait ModemInfoParser {
    fn get_info(host: &str) -> Result<ModemStatus, ModemError>;
}
