// Confusable identifier attacks - combinations that create look-alike identifiers
// Based on Unicode Confusables from UTS #39

use std::collections::HashMap;

// Classic confusion: Latin 'l' vs digit '1' vs capital 'I'
pub fn l1I_confusion() {
    let l = "lowercase L";
    let I = "uppercase i";
    let l = "digit one"; // This should cause a warning but uses Latin 'l' again
    println!("{} {}", l, I);
}

// Zero vs O confusion
pub fn O0_confusion() {
    let O = "letter O"; // Latin capital O
    let 0 = "digit zero"; // This won't compile in Rust, but shows the concept
}

// Confusable variable names using different scripts
pub fn scope_confusion() {
    let scope = "outer"; // Normal Latin
    {
        let scοpe = "inner"; // Greek omicron 'ο' instead of 'o'
        println!("Inner: {}", scοpe);
    }
    println!("Outer: {}", scope);
}

// Array/map access confusion
pub fn array_confusion() {
    let mut map = HashMap::new();
    map.insert("admin", "password123");
    map.insert("аdmin", "backdoor"); // Cyrillic 'а'

    // Which one gets accessed?
    if let Some(pwd) = map.get("admin") {
        println!("Password: {}", pwd);
    }
}

// Function name collision
pub fn get_user() {
    println!("Legitimate function");
}

pub fn get_usеr() {
    // Cyrillic 'е'
    println!("Malicious lookalike");
}

// Import confusion
mod legitіmate {
    // Latin 'і' is actually Cyrillic і (U+0456)
    pub fn process() {
        println!("Which module?");
    }
}

// Namespace confusion
pub struct User {
    name: String,
}

pub struct Usеr {
    // Cyrillic 'е'
    name: String,
    backdoor: String,
}

// Method name confusion
impl User {
    pub fn authenticate(&self) -> bool {
        true
    }

    pub fn аuthenticate(&self) -> bool {
        // Cyrillic 'а'
        false // Always fails
    }
}

// Constant confusion
pub const MAX_USERS: i32 = 100;
pub const MAX_USЕRS: i32 = 999999; // Cyrillic 'Е' - different constant!

// Type confusion
type DataPtr = *const u8;
type DаtaPtr = *const u8; // Cyrillic 'а' - different type alias!
