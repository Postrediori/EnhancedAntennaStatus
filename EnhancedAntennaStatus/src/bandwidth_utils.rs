use std::time::SystemTime;

pub const SIZE_KB: i64 = 1024;
pub const SIZE_MB: i64 = 1024 * 1024;
pub const SIZE_GB: i64 = 1024 * 1024 * 1024;

pub fn format_bandwidth(bits_per_second: i64) -> String {
    const RATE_BPS: &str = "bit/s";
    const RATE_KBPS: &str = "KBit/s";
    const RATE_MBPS: &str = "MBit/s";
    const RATE_GBPS: &str = "GBit/s";

    if bits_per_second < SIZE_KB {
        format!("{bits_per_second}{RATE_BPS}").to_string()
    }
    else if bits_per_second < SIZE_MB {
        format!("{:.2}{RATE_KBPS}", (bits_per_second as f64) / (SIZE_KB as f64)).to_string()
    }
    else if bits_per_second < SIZE_GB {
        format!("{:.2}{RATE_MBPS}", (bits_per_second as f64) / (SIZE_MB as f64)).to_string()
    }
    else {
        format!("{:.2}{RATE_GBPS}", (bits_per_second as f64) / (SIZE_GB as f64)).to_string()
    }
}

pub fn nearest_fib(x: i64) -> i64 {
    let mut f1: i64 = 0;
    let mut f2: i64 = 1;
    let mut k: i64 = 0;

    for _ in 1..20 {
        k = f1 + f2;
        if x < k {
            break;
        }
        f2 = f1;
        f1 = k;
    }
    k
}

/*
 * Status object that calculates download/upload rates per second based on total dl/ul bytes
 */
pub struct BandwidthCounter {
    dlul_time: SystemTime,
    total_bytes: (i64, i64),
}

impl BandwidthCounter {
    pub fn new() -> Self {
        let dlul_time = SystemTime::now();

        let total_bytes = (0, 0);

        Self {
            dlul_time,
            total_bytes,
        }
    }

    // Update with total values
    pub fn update_with_total_values(&mut self, new_total_bytes: (i64, i64)) -> Option<(i64, i64)> {
        let current_time = SystemTime::now();
        if let Ok(dt) = current_time.duration_since(self.dlul_time) {
            let t = 1000.0 / dt.as_millis() as f64;
            let dl = if self.total_bytes.0 > 0 {
                ((new_total_bytes.0 - self.total_bytes.0) as f64 * t) as i64
            } else {
                0
            };
            let ul = if self.total_bytes.1 > 0 {
                ((new_total_bytes.1 - self.total_bytes.1) as f64 * t) as i64
            } else {
                0
            };

            self.dlul_time = current_time;
            self.total_bytes = new_total_bytes;

            Some((dl, ul))
        }
        else {
            None
        }
    }
}
