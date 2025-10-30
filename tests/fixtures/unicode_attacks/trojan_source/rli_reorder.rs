// RLI Reordering Attack with actual Unicode characters
pub fn vulnerable_function() {
    // The following line contains RLI (U+2067) and LRI (U+2066)
    println!("Access level: ⁧user⁦guest"); // Visually appears as "Access level: guestuser"
}
