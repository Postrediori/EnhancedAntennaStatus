use std::str::FromStr;

/// Copy String object to array of chars
#[macro_export]
macro_rules! copy_string_to_array {
    ($array:tt, $string:expr) => {
        let len = $string.len().min($array.len() - 1);
        $array[..len].copy_from_slice(&$string.chars().collect::<Vec<char>>()[..len]);
    }
}
pub use copy_string_to_array;

/// Flag wrapper that monitors change of the flag
pub struct ValueChangeObserver<T: PartialEq+Copy> {
    val: Option<T>,
}

impl<T: PartialEq+Copy> ValueChangeObserver<T> {
    pub fn new() -> Self {
        let val: Option<T> = None;
        Self {
            val,
        }
    }
    pub fn update_and_check_if_changed(&mut self, new_val: T) -> bool {
        let status = if let Some(v) = self.val {
            v != new_val
        }
        else {
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
    if let Some(val) = val.as_str() {
        if let Ok(val) = val.parse::<T>() {
            Some(val)
        }
        else {
            None
        }
    }
    else {
        None
    }
}
