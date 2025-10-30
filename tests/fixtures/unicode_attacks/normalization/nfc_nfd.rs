// NFC/NFD confusion test fixtures (T068)
// Tests for normalization form attacks

// Café with different normalizations
// NFC: é is a single character (U+00E9)
pub fn café_nfc() {
    let café = "café"; // NFC form: é = U+00E9
    println!("{}", café);
}

// NFD: é is e (U+0065) + combining acute (U+0301)
pub fn café_nfd() {
    let café = "café"; // NFD form: e + ́
    println!("{}", café);
}

// Both look identical but are different byte sequences
pub fn normalization_confusion() {
    let nfc = "résumé"; // NFC
    let nfd = "résumé"; // NFD (visually identical)

    // These are different identifiers!
    println!("{} {}", nfc, nfd);
}

// Function names with different normalizations
pub fn naïve_nfc() {
    // NFC: ï is U+00EF
    println!("NFC form");
}

pub fn naïve_nfd() {
    // NFD: i + combining diaeresis
    println!("NFD form");
}

// Variable shadowing with normalization
pub fn variable_confusion() {
    let café = "NFC";
    let café = "NFD"; // Different variable!
    println!("{}", café);
}

// Type names with normalization differences
pub struct Naïve {
    // NFC
    value: i32,
}

pub struct Naïve {
    // NFD - different struct!
    data: String,
}
