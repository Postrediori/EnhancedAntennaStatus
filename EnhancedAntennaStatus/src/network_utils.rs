/*
 * HTTP utils
 */
use http::Uri;
use std::time::Duration;

const HTTP_TIMEOUT: Duration = Duration::from_millis(3_000);

pub fn get_url_json(host: &str, query: &str) -> Option<serde_json::Value> {
    if let Ok(path) = Uri::builder()
        .scheme("http")
        .authority(host)
        .path_and_query(query)
        .build()
    {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(HTTP_TIMEOUT)
            .build();

        let req = agent.get(&path.to_string());
        match req.call() {
            Ok(response) => {
                if let Ok(json) = response.into_json::<serde_json::Value>() {
                    return Some(json);
                }
            }
            Err(ureq::Error::Status(code, response)) => {
                eprintln!(
                    "HTTP error code={} response={}",
                    code,
                    response.status_text()
                );
            }
            Err(e) => {
                eprintln!("HTTP error={}", &e.to_string());
            }
        }
    }
    return None;
}

pub fn get_url_xml(host: &str, query: &str) -> Option<xmltree::Element> {
    if let Ok(path) = Uri::builder()
        .scheme("http")
        .authority(host)
        .path_and_query(query)
        .build()
    {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(HTTP_TIMEOUT)
            .build();

        let req = agent.get(&path.to_string()).set("Accept", "*/*");

        match req.call() {
            Ok(response) => {
                if let Ok(xml) = response.into_string() {
                    match xmltree::Element::parse(xml.as_bytes()) {
                        Ok(parsed_xml) => {
                            return Some(parsed_xml);
                        }
                        Err(e) => {
                            eprintln!("XML DOM error={}", &e.to_string());
                        }
                    }
                }
            }
            Err(ureq::Error::Status(code, response)) => {
                eprintln!(
                    "HTTP error code={} response={}",
                    code,
                    response.status_text()
                );
            }
            Err(e) => {
                eprintln!("HTTP error={}", &e.to_string());
            }
        }
    }
    return None;
}

pub fn get_url_xml_with_session_token(
    host: &str,
    sesion_token: &Option<(String, String)>,
    query: &str,
) -> Option<xmltree::Element> {
    if let Ok(path) = Uri::builder()
        .scheme("http")
        .authority(host)
        .path_and_query(query)
        .build()
    {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(HTTP_TIMEOUT)
            .build();

        let mut req = agent.get(&path.to_string()).set("Accept", "*/*");
        if let Some((session_info, token_info)) = sesion_token {
            req = req
                .set("Host", host)
                .set("X-Requested-With", "XMLHttpRequest")
                .set("Cookie", session_info.as_str())
                .set("__RequestVerificationToken", token_info.as_str());
        }

        match req.call() {
            Ok(response) => {
                if let Ok(xml) = response.into_string() {
                    match xmltree::Element::parse(xml.as_bytes()) {
                        Ok(parsed_xml) => {
                            return Some(parsed_xml);
                        }
                        Err(e) => {
                            eprintln!("XML DOM error={}", &e.to_string());
                        }
                    }
                }
            }
            Err(ureq::Error::Status(code, response)) => {
                eprintln!(
                    "HTTP error code={} response={}",
                    code,
                    response.status_text()
                );
            }
            Err(e) => {
                eprintln!("HTTP error={}", &e.to_string());
            }
        }
    }
    return None;
}
