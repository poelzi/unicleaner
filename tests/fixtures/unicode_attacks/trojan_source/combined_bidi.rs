// Combined Bidi Attack with actual Unicode characters
pub fn vulnerable_function() {
    // The following line contains RLO (U+202E) and PDF (U+202C)
    if false {
        // ‮ Begin RLO
        println!("This should not be executed");
    } // ‬ End PDF

    let user = "admin";
    // The following line contains LRI (U+2066) and PDI (U+2069)
    if user == "guest⁦ // Check if user is admin ⁩" {
        println!("Executing as guest");
    }
}
