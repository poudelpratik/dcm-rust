// example of a function that runs for a long time and consumes much resources depending on the parameter
/// @mobile
pub fn find_primes(limit: u32) -> Vec<u32> {
    let mut primes = Vec::new();
    'outer: for n in 2..=limit {
        for p in &primes {
            if n % p == 0 {
                continue 'outer;
            }
        }
        primes.push(n);
    }
    primes
}

// // example of a function that has state
// /// @mobile
// pub fn counter() -> u32 {
//     static mut COUNTER: u32 = 0;
//     unsafe {
//         COUNTER += 1;
//         COUNTER
//     }
// }
//
// This one is included in the CFD file just for example
// pub fn add(a: i32, b: i32) -> i32 {
//     a + b
// }
//
// // This one is included in the CFD file just for example
// pub fn multiply(a: i32, b: i32) -> i32 {
//     a * b
// }
//
// pub fn hello_world() -> String {
//     "Hello, world!".to_string()
// }
//
// pub fn reverse_string(s: &str) -> String {
//     s.chars().rev().collect()
// }
//
/// @mobile
pub fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

/// @mobile
pub fn count_vowels(s: &str) -> u32 {
    let vowels = ['a', 'e', 'i', 'o', 'u', 'A', 'E', 'I', 'O', 'U'];
    s.chars().filter(|c| vowels.contains(c)).count() as u32
}

/// @mobile
pub fn factorial(n: u32) -> u32 {
    match n {
        0 => 1,
        _ => n * factorial(n - 1),
    }
}

// // pub fn to_lowercase(s: &str) -> String {
// //     s.to_lowercase()
// // }
// //
// // /// @mobile
// // pub fn is_palindrome(s: &str) -> bool {
// //     let cleaned: String = s.chars().filter(|c| c.is_alphanumeric()).collect();
// //     let reversed: String = cleaned.chars().rev().collect();
// //     cleaned.eq_ignore_ascii_case(&reversed)
// // }
// //
// // /// @mobile(crates = ["itertools"])
// // pub fn invoke_itertools_demo() {
// //     crate::shared::playground1::itertools_demo();
// //     whatever::hello();
// // }
// //
// // mod whatever {
// //     pub fn hello() {
// //         println!("Hello");
// //     }
// // }
//
// use fastrand;
//
// /// @mobile(crates = ["fastrand"])
// pub fn generate_random_number() -> u32 {
//     fastrand::u32(..)
// }
