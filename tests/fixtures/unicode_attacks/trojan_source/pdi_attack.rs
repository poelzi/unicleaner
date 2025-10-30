// PDI Attack with actual Unicode characters
pub fn vulnerable_function() {
    // The following line contains RLI (U+2067) and PDI (U+2069)
    println!("Access level: ⁧user⁩guest"); // Visually appears as "Access level: guest"
}
