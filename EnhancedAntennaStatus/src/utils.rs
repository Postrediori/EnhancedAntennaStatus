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
