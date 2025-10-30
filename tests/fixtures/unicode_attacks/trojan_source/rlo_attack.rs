// RLO Attack with actual Unicode character
pub fn vulnerable_function() {
    println!("Executing sensitive operation");
    // The following line contains RLO (U+202E) for Trojan Source attack
    if false {
        // ‮ } ⁦if true⁩ // CVE-2021-42574
        println!("This should not be executed");
    }
}
