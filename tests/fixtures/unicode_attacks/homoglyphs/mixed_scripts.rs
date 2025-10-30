// Mixed script attacks - combining characters from different writing systems
// This is particularly dangerous as it can bypass simple script-based checks

// Mix of Latin, Cyrillic, and Greek
pub fn prοcess_dаta() {
    // 'ο' is Greek, 'а' is Cyrillic, rest is Latin
    println!("Mixed script function name");
}

// Mix in structure names
pub struct Usеr {
    // Cyrillic 'е'
    nаme: String,  // Cyrillic 'а'
    еmail: String, // Cyrillic 'е'
    age: u32,      // Latin
}

// Mix in module paths
pub mod authеntication {
    // Cyrillic 'е'
    pub fn valіdate() {
        // Cyrillic 'і' (U+0456)
        println!("Mixed script module");
    }
}

// Mix of Latin and Mathematical
pub fn calculate_νalue() {
    // Greek 'ν' (nu)
    let result = compute_𝑎rea(); // Mathematical italic 'a'
    println!("{}", result);
}

fn compute_𝑎rea() -> f64 {
    // Mathematical italic 'a'
    let π = 3.14159; // Greek pi (legitimate in math contexts)
    let rаdius = 5.0; // Cyrillic 'а'
    π * rаdius * rаdius
}

// Mix in trait implementations
pub trait Prοcessor {
    // Greek 'ο'
    fn prоcess(&self); // Cyrillic 'о'
}

pub struct DataPrоcessor; // Cyrillic 'о'

impl Prοcessor for DataPrоcessor {
    // Greek and Cyrillic mixed
    fn prоcess(&self) {
        // Cyrillic 'о'
        println!("Processing");
    }
}

// Mix in generics
pub fn transform<Т>(value: Т) -> Т {
    // Cyrillic 'Т' (U+0422)
    value
}

// Mix of Cyrillic, Greek, and Latin in one identifier
pub fn аuthеntіcаtе_usеr_wіth_tοkеn() {
    // 'а', 'е', 'і', 'а', 'е', 'е', 'і', 'ο', 'е' are non-Latin
    println!("Heavily mixed script identifier");
}

// Multiple scripts in variable shadowing
pub fn shadowing_attack() {
    let data = "version1"; // All Latin
    let dаta = "version2"; // Cyrillic 'а'
    let dаtа = "version3"; // Two Cyrillic 'а's
    let ďata = "version4"; // Latin with combining mark

    println!("{}", dаtа); // Which one?
}

// Mixed scripts in string literals and identifiers
pub fn string_confusion() {
    let message = "Hello"; // Normal
    let mеssage = "Hеllo"; // Cyrillic 'е' in both identifier and string

    println!("{}", mеssage);
}

// Armenian, Georgian, and other scripts
pub fn advanced_mixing() {
    let varіable = "test"; // Armenian 'ի' (U+056B) that looks like Latin
    let dataԿ = "value"; // Armenian 'Կ' (U+053F)
    println!("{} {}", varіable, dataԿ);
}

// Combining diacritical marks to make Latin look like other scripts
pub fn combining_confusion() {
    let café = "French"; // Normal combining accent
    let cafė = "Lithuanian"; // Different combining mark
    let cafe̅ = "With macron"; // Yet another combining mark

    // All three look different but are visually similar
    println!("{}", café);
}
