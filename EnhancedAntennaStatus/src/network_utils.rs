/*
 * HTTP utils
 */
use std::time::Duration;
use http::Uri;

const HTTP_TIMEOUT: Duration = Duration::from_millis(3_000);

pub fn get_url_json(host: &str, query: &str) -> Option<serde_json::Value> {
    if let Ok(path) = Uri::builder()
        .scheme("http")
        .authority(host)
        .path_and_query(query)
        .build() {

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
                eprintln!("HTTP error code={} response={}", code, response.status_text());
            }
            Err(e) => {
                eprintln!("HTTP error={}", &e.to_string());
            }
        }
    }
    return None;
}
