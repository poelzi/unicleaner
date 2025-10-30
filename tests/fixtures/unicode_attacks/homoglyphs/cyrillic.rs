// Cyrillic homoglyph attack - uses visually identical Cyrillic characters
// The Cyrillic 'а' (U+0430) looks identical to Latin 'a' (U+0061)
// The Cyrillic 'е' (U+0435) looks identical to Latin 'e' (U+0065)
// The Cyrillic 'о' (U+043E) looks identical to Latin 'o' (U+006F)

pub fn аuthenticate_user() {
    // First 'а' is Cyrillic U+0430, not Latin!
    println!("This function name uses Cyrillic homoglyph");
}

pub fn validate_еmail() {
    // 'е' is Cyrillic U+0435
    println!("Email validation with Cyrillic character");
}

pub fn get_user_tоken() {
    // 'о' is Cyrillic U+043E
    println!("Token retrieval with homoglyph");
}

// Dangerous: mix of Latin and Cyrillic in identifiers
fn process_dаta() {
    // 'а' is Cyrillic
    let usеr = "admin"; // 'е' is Cyrillic
    let tоken = "secret"; // 'о' is Cyrillic
    println!("{} {}", usеr, tоken);
}

// Even more deceptive: variable shadowing with homoglyphs
pub fn confusing_example() {
    let data = "legitimate"; // All Latin
    let dаta = "malicious"; // 'а' is Cyrillic - different variable!

    println!("{}", dаta); // Which one gets printed?
}
