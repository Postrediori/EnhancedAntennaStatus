use std::fmt::Debug;
use std::str::FromStr;

/// Copy String object to array of chars
#[macro_export]
macro_rules! copy_string_to_array {
    ($array:tt, $string:expr) => {
        let len = $string.len().min($array.len() - 1);
        $array[..len].copy_from_slice(&$string.chars().collect::<Vec<char>>()[..len]);
    };
}
pub use copy_string_to_array;

/// Flag wrapper that monitors change of the flag
pub struct ValueChangeObserver<T: PartialEq + Copy> {
    val: Option<T>,
}

impl<T: PartialEq + Copy> ValueChangeObserver<T> {
    pub fn new() -> Self {
        let val: Option<T> = None;
        Self { val }
    }
    pub fn update_and_check_if_changed(&mut self, new_val: T) -> bool {
        let status = if let Some(v) = self.val {
            v != new_val
        } else {
            true
        };
        if status {
            self.val = Some(new_val);
        }
        status
    }
}

/// Parse parameter of JSON that is integer represented as string
pub fn json_str_as_type<T: FromStr>(val: &serde_json::Value) -> Option<T> {
    val.as_str().and_then(|val| val.parse::<T>().ok())
}

/// Truncate unit at the end of the string
fn truncate_unit(s: &mut String) {
    const UNITS: [&str; 2] = ["dB", "dBm"];

    for u in &UNITS {
        if s.ends_with(u) {
            s.truncate(s.len() - u.len());
            break;
        }
    }
}

/// Parse XML element content as string
pub fn get_xml_element(xml: &xmltree::Element, name: &str) -> Option<String> {
    if let Some(element) = xml.get_child(name) {
        if let Some(str) = element.get_text() {
            return Some(str.to_string());
        }
    }
    None
}

/// Parse XML element content as type T
pub fn get_xml_element_as<T: FromStr>(xml: &xmltree::Element, name: &str) -> Option<T>
where
    <T as FromStr>::Err: Debug,
{
    if let Some(element) = xml.get_child(name) {
        if let Some(str) = element.get_text() {
            match str.to_string().parse::<T>() {
                Ok(val) => {
                    return Some(val);
                }
                Err(e) => {
                    eprintln!("Error: parsing element '{name}': {e:?}");
                }
            }
        }
    }
    None
}

/// Parse XML element content as type T with possible unit
pub fn get_xml_element_as_unit<T: FromStr>(xml: &xmltree::Element, name: &str) -> Option<T>
where
    <T as FromStr>::Err: Debug,
{
    if let Some(element) = xml.get_child(name) {
        if let Some(str) = element.get_text() {
            let mut str = str.to_string();
            truncate_unit(&mut str);
            match str.parse::<T>() {
                Ok(val) => {
                    return Some(val);
                }
                Err(e) => {
                    eprintln!("Error parsing element '{name}': {e:?}");
                }
            }
        }
    }
    None
}

pub fn xml_contains_required_parameters(xml: &xmltree::Element, parameters: &[&str]) -> bool {
    for p in parameters {
        if get_xml_element(xml, p).is_none() {
            eprintln!("XML data doesn't have parameter '{p}'");
            return false;
        }
    }
    true
}
