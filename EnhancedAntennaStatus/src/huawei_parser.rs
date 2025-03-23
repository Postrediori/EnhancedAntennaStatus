#![allow(clippy::similar_names)]

use std::fmt;
use std::str::FromStr;

use crate::bandwidth_utils::{TrafficMode, TrafficStatistics};
use crate::modem_utils::{
    BatteryStatus, DeviceInformation, LteSignalInfo, ModemError, ModemInfoParser, ModemStatus,
    NetworkMode, PlmnStatus, SignalInfo, WcdmaSignalInfo,
};
use crate::network_utils::{get_url_xml, get_url_xml_with_session_token};
use crate::utils::{
    copy_string_to_array, get_xml_element, get_xml_element_as, get_xml_element_as_unit,
    xml_contains_required_parameters,
};

/// Convert id from 'mode' parameter in XML to `NetworkMode` enum
fn get_mode_by_id(s: &str) -> NetworkMode {
    match s {
        "0" => NetworkMode::Gsm,
        "2" => NetworkMode::Wcdma,
        "7" => NetworkMode::Lte,
        _ => NetworkMode::Unknown,
    }
}

/// REST API error in Huawei format
struct HuaweiError {
    code: Option<String>,
    message: Option<String>,
}

impl fmt::Display for HuaweiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Huawei REST Error{}{}",
            self.code
                .as_ref()
                .map(|code| format!(" code='{code}'"))
                .unwrap_or_default(),
            self.message
                .as_ref()
                .map(|message| format!(" message='{message}'"))
                .unwrap_or_default(),
        )
    }
}

/// Check if XML contains error status
fn check_huawei_error_xml(xml: &xmltree::Element) -> Result<(), HuaweiError> {
    if xml.name.eq("error") {
        let code = get_xml_element(xml, "code");
        let message = get_xml_element(xml, "message");

        Err(HuaweiError { code, message })
    } else {
        Ok(())
    }
}

/// Session info for Huawei web UI: (session info, token info)
pub type SessionInfo = (String, String);

/*
 * Utils for Huawei
 */

pub struct HuaweiParser {}

impl HuaweiParser {
    fn parse_signal_xml(xml: &xmltree::Element) -> Option<ModemStatus> {
        const REQUIRED_PARAMETERS: [&str; 3] = ["mode", "rssi", "cell_id"];
        if !xml_contains_required_parameters(xml, &REQUIRED_PARAMETERS) {
            return None;
        }

        let mode = get_mode_by_id(get_xml_element(xml, "mode").unwrap().as_str());

        let rssi = get_xml_element_as_unit::<i64>(xml, "rssi").unwrap();

        let plmn = PlmnStatus::from_str("00000").expect("Unable to convert PLMN from string"); // PLMN is set by a different request
        let band = ['\0'; 20]; // TODO: Band on Huawei?

        let cell_id = get_xml_element_as::<i64>(xml, "cell_id").unwrap();

        let signal_info: SignalInfo = match mode {
            NetworkMode::Wcdma => {
                const WCDMA_PARAMETERS: [&str; 2] = ["rscp", "ecio"];
                if !xml_contains_required_parameters(xml, &WCDMA_PARAMETERS) {
                    return None;
                }

                let rscp = get_xml_element_as_unit::<i64>(xml, "rscp").unwrap();
                let ecio = get_xml_element_as_unit::<i64>(xml, "ecio").unwrap();

                let psc = get_xml_element_as_unit::<i64>(xml, "sc").unwrap_or(0);

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
                const LTE_PARAMETERS: [&str; 3] = ["rsrp", "rsrq", "sinr"];
                if !xml_contains_required_parameters(xml, &LTE_PARAMETERS) {
                    return None;
                }

                let rsrp = get_xml_element_as_unit::<i64>(xml, "rsrp").unwrap();
                let rsrq = get_xml_element_as_unit::<i64>(xml, "rsrq").unwrap();
                let sinr = get_xml_element_as_unit::<i64>(xml, "sinr").unwrap();

                let pci = get_xml_element_as_unit::<i64>(xml, "pci").unwrap_or(0);

                let (enb, id) = (cell_id >> 8, cell_id & 0xFF);

                SignalInfo::Lte(LteSignalInfo {
                    rsrq,
                    rsrp,
                    sinr,
                    ca_count: 0,
                    enb,
                    id,
                    pci, // TODO: ca_count on Huawei?
                })
            }
            _ => SignalInfo::None,
        };

        Some(ModemStatus {
            mode,
            plmn,
            rssi,
            cell_id,
            signal_info,
            band,
            device_info: DeviceInformation::from("HUAWEI", ""),
            battery_status: None,
            device_temp: None,
            traffic_statistics: None,
            traffic_mode: TrafficMode::Absolute,
        })
    }
    fn parse_session_token_xml(xml: &xmltree::Element) -> Option<SessionInfo> {
        if let (Some(ses_info), Some(tok_info)) = (
            get_xml_element(xml, "SesInfo"),
            get_xml_element(xml, "TokInfo"),
        ) {
            Some((ses_info, tok_info))
        } else {
            None
        }
    }
    fn get_session_token(host: &str) -> Option<SessionInfo> {
        get_url_xml(host, "/api/webserver/SesTokInfo")
            .and_then(|xml| HuaweiParser::parse_session_token_xml(&xml))
    }
    fn parse_traffic_statistics_xml(xml: &xmltree::Element) -> TrafficStatistics {
        let dl = if let Some(dl) = get_xml_element_as_unit::<i64>(xml, "CurrentDownloadRate") {
            dl * 8
        } else {
            0
        };
        let ul = if let Some(ul) = get_xml_element_as_unit::<i64>(xml, "CurrentUploadRate") {
            ul * 8
        } else {
            0
        };
        TrafficStatistics { dl, ul }
    }
    fn get_traffic_statistics(
        host: &str,
        session_token: Option<&SessionInfo>,
    ) -> Option<TrafficStatistics> {
        let xml = get_url_xml_with_session_token(
            host,
            session_token,
            "/api/monitoring/traffic-statistics",
        );

        xml.map(|xml| HuaweiParser::parse_traffic_statistics_xml(&xml))
    }
    fn parse_battery_status_xml(xml: &xmltree::Element) -> Option<BatteryStatus> {
        let battery_percent = get_xml_element_as_unit::<i64>(xml, "BatteryPercent")?;

        let battery_status_str = if let Some(status) = get_xml_element(xml, "BatteryStatus") {
            match status.as_str() {
                "0" => "No Charge",
                "1" => "Charging",
                "-1" => "Low",
                "2" => "No Battery",
                _ => "Unknown status",
            }
        } else {
            return None;
        };

        let mut battery_status = ['\0'; 20];
        copy_string_to_array!(battery_status, battery_status_str);

        Some(BatteryStatus {
            percent: battery_percent,
            status: battery_status,
        })
    }
    fn get_battery_status(
        host: &str,
        session_token: Option<&SessionInfo>,
    ) -> Option<BatteryStatus> {
        let xml = get_url_xml_with_session_token(host, session_token, "/api/monitoring/status");

        if let Some(xml) = xml {
            HuaweiParser::parse_battery_status_xml(&xml)
        } else {
            None
        }
    }
    fn parse_plmn_xml(xml: &xmltree::Element) -> PlmnStatus {
        let plmn_str = get_xml_element(xml, "Numeric").unwrap_or_default();

        PlmnStatus::from_str(&plmn_str).expect("Unable to convert PLMN from string")
    }
    fn get_plmn_status(host: &str, session_token: Option<&SessionInfo>) -> Option<PlmnStatus> {
        let xml = get_url_xml_with_session_token(host, session_token, "/api/net/current-plmn");

        xml.map(|xml| HuaweiParser::parse_plmn_xml(&xml))
    }
    fn parse_device_model_xml(xml: &xmltree::Element) -> String {
        if let Some(model_str) = get_xml_element(xml, "devicename") {
            model_str
        } else {
            get_xml_element(xml, "DeviceName").unwrap_or_default()
        }
    }
    fn get_device_model_by_query(
        host: &str,
        session_token: Option<&SessionInfo>,
        query: &str,
    ) -> Result<String, ModemError> {
        let query = format!("/api/device/{query}");
        let xml = get_url_xml_with_session_token(host, session_token, &query);

        if let Some(xml) = xml {
            match check_huawei_error_xml(&xml) {
                Err(e) => {
                    eprintln!("Device Information Access error: {e}");
                    Err(ModemError::Access)
                }
                Ok(()) => Ok(HuaweiParser::parse_device_model_xml(&xml)),
            }
        } else {
            Err(ModemError::HttpConnection)
        }
    }
    fn get_device_information(
        host: &str,
        session_token: Option<&SessionInfo>,
    ) -> Result<DeviceInformation, ModemError> {
        let manufacturer = "HUAWEI"; // Hardcoded
        let mut model = String::new();

        for query in ["basic_information", "information"] {
            if let Ok(model_str) =
                HuaweiParser::get_device_model_by_query(host, session_token, query)
            {
                if !model_str.is_empty() {
                    model = model_str;
                    break;
                }
            } else {
                /* Ignore errors from getting device data */
            }
        }

        if model.is_empty() {
            Err(ModemError::Access)
        } else {
            Ok(DeviceInformation::from(manufacturer, model.as_str()))
        }
    }
}

impl ModemInfoParser for HuaweiParser {
    fn get_info(host: &str) -> Result<ModemStatus, ModemError> {
        let session_token = HuaweiParser::get_session_token(host);

        let xml =
            get_url_xml_with_session_token(host, session_token.as_ref(), "/api/device/signal");

        if let Some(xml) = xml {
            if let Some(mut modem_status) = HuaweiParser::parse_signal_xml(&xml) {
                if let Some(plmn) = HuaweiParser::get_plmn_status(host, session_token.as_ref()) {
                    modem_status.plmn = plmn;
                }

                modem_status.traffic_statistics =
                    HuaweiParser::get_traffic_statistics(host, session_token.as_ref());
                modem_status.battery_status =
                    HuaweiParser::get_battery_status(host, session_token.as_ref());

                // Get model info
                if let Ok(device_info) =
                    HuaweiParser::get_device_information(host, session_token.as_ref())
                {
                    modem_status.device_info = device_info;
                } else {
                    /* Ignore errors from getting device data */
                }

                Ok(modem_status)
            } else {
                eprintln!("Cannot parse signal data");
                Err(ModemError::DataParsing)
            }
        } else {
            eprintln!("Cannot get signal data from host {host}");
            Err(ModemError::HttpConnection)
        }
    }
}
