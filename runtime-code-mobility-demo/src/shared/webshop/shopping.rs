use std::cmp::Ordering;
use std::thread;
use std::time::Duration;

/// @mobile
pub fn sort_strings(sort_mode: &str, arr: Vec<String>) -> Vec<String> {
    let mut sorted_slice = arr;

    if sort_mode == "asc" && is_sorted(&sorted_slice) {
        return sorted_slice;
    }

    match sort_mode {
        "asc" => sorted_slice.sort(),
        "desc" => sorted_slice.sort_by(|a, b| b.cmp(a)),
        _ => sorted_slice.sort(),
    }

    sorted_slice
}

/// @mobile
pub fn sort_floats(sort_mode: &str, arr: Vec<f64>) -> Vec<f64> {
    let mut sorted_slice = arr;

    if sort_mode == "asc" && sorted_slice.windows(2).all(|w| w[0] <= w[1]) {
        return sorted_slice;
    }

    match sort_mode {
        "asc" => sorted_slice.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)),
        "desc" => sorted_slice.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal)),
        _ => sorted_slice.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)),
    }

    sorted_slice
}

/// @mobile
pub fn increment_number(num: i32) -> i32 {
    num + 1
}

/// @mobile
pub fn decrement_number(num: i32) -> i32 {
    num - 1
}

/// @mobile
pub fn add_floats(floats: Vec<f64>) -> f64 {
    floats.iter().sum()
}

/// @mobile
pub fn check() -> i32 {
    let count = 5;
    for _ in 0..count {
        thread::sleep(Duration::from_secs(2));
        println!("Check");
    }
    1
}

/// @mobile
pub fn validate_coupon_input(input: &str) -> bool {
    if input.is_empty() || input.len() != 11 {
        return false;
    }
    true
}

static VALID_COUPONS: [&str; 5] = [
    "491-314-740",
    "780-989-453",
    "301-502-258",
    "337-745-466",
    "285-799-970",
];

/// @mobile
pub fn validate_coupon(input: &str) -> bool {
    VALID_COUPONS.contains(&input)
}

/// @mobile
pub fn calc_discount(total: f64, discount: f64) -> f64 {
    if total <= 0.0 || discount <= 0.0 {
        0.0
    } else {
        (total / 100.0) * discount
    }
}

/// @mobile
pub fn round_float(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// Helper function to check if a vector is sorted
fn is_sorted<T: Ord>(data: &[T]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}
