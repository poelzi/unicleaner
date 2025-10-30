// Greek homoglyph attack - uses Greek letters that look like Latin
// Greek 'ο' (omicron U+03BF) looks identical to Latin 'o' (U+006F)
// Greek 'ρ' (rho U+03C1) looks similar to Latin 'p' (U+0070)
// Greek 'ν' (nu U+03BD) looks similar to Latin 'v' (U+0076)
// Greek 'α' (alpha U+03B1) looks similar to Latin 'a' (U+0061)

pub fn get_user_infο() {  // 'ο' is Greek omicron U+03BF
    println!("User info with Greek omicron");
}

pub fn valida

te_ραssword() {  // 'ρ' is Greek rho U+03C1, 'α' is Greek alpha U+03B1
    println!("Password validation with Greek characters");
}

fn calculate_νalue() {  // 'ν' is Greek nu U+03BD
    let result = 42;
    println!("Value: {}", result);
}

// Mixed Greek and Latin in identifier
pub fn prοcess_transaction() {  // 'ο' is Greek omicron
    let αmount = 1000;  // 'α' is Greek alpha
    let reνenue = 500;  // 'ν' is Greek nu

    println!("Transaction: {} - {}", αmount, reνenue);
}

// Mathematical context where Greek might seem legitimate but is deceptive
pub fn calculate_αrea() {  // 'α' is Greek alpha (might look like math notation)
    let π = 3.14159;  // Legitimate use of Greek pi
    let radius = 5.0;
    let αrea = π * radius * radius;  // 'α' is Greek alpha in variable name

    println!("Area: {}", αrea);
}
