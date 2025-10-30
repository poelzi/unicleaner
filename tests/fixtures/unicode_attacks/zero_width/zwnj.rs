// Zero-Width Non-Joiner (ZWNJ U+200C) attack
// ZWNJ is used in some languages but can be abused in code

// ZWNJ in identifiers
pub fn get‌user() {
    // ZWNJ between 'get' and 'user'
    println!("ZWNJ in function name");
}

pub fn authenticate‌user() {
    // ZWNJ in middle
    let pass‌word = "secret"; // ZWNJ in variable
    println!("{}", pass‌word);
}

// ZWNJ creating duplicate identifiers
pub fn confusion() {
    let data = "original";
    let data‌ = "duplicate"; // ZWNJ makes this different!
    println!("{}", data‌);
}

// ZWNJ in type names
pub struct User‌Info {
    name‌: String, // ZWNJ in field name
}

impl User‌Info {
    pub fn new‌() -> Self {
        User‌Info {
            name‌: String::new(),
        }
    }
}

// Multiple ZWNJs
pub fn multi‌‌‌ple() {
    // Three ZWNJs
    let val‌‌ue = 42;
    println!("{}", val‌‌ue);
}
