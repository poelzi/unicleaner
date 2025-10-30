// LRI Hiding Attack with actual Unicode characters
pub fn vulnerable_function() {
    let user = "admin";
    // The following line contains LRI (U+2066) and PDI (U+2069)
    if user == "guest⁦ // Check if user is admin ⁩" {
        println!("Executing as guest");
    }
}
