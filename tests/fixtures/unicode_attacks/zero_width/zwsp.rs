// Zero-Width Space (ZWSP U+200B) attack
// ZWSP is invisible but can hide code or create confusion

// ZWSP in function names
pub fn get​_user() {  // Zero-Width Space between 'get' and '_user'
    println!("Function with hidden ZWSP");
}

pub fn process​data() {  // ZWSP before 'data'
    println!("Processing");
}

// ZWSP in variable names
pub fn variable_confusion() {
    let user = "Alice";  // Normal
    let user​ = "Bob";  // ZWSP after 'user' - different variable!

    println!("{}", user​);  // Which one?
}

// ZWSP in identifiers
pub fn authenticate() {
    let admin​_token = "secret";  // ZWSP in identifier
    let pass​word = "12345";  // ZWSP in identifier

    println!("{} {}", admin​_token, pass​word);
}

// Multiple ZWSPs
pub fn multi​ple​​spaces() {  // Multiple ZWSPs
    let val​​​ue = 42;  // Three ZWSPs
    println!("{}", val​​​ue);
}

// ZWSP in string literals (less dangerous but still detectable)
pub fn string_zwsp() {
    let message = "Hello​World";  // ZWSP between words
    let data = "test​​data";  // Multiple ZWSPs
    println!("{} {}", message, data);
}

// ZWSP in comments (can hide malicious intent)
pub fn comment_zwsp() {
    // This is a normal​comment with ZWSP
    /* Multi-line​comment​with​ZWSPs */
    println!("Comments with hidden characters");
}

// ZWSP in type names
pub struct User​Data {  // ZWSP in struct name
    name: String,
    value​: i32,  // ZWSP in field name
}

// ZWSP in impl blocks
impl User​Data {
    pub fn new​() -> Self {  // ZWSP in method name
        User​Data {
            name: String::new(),
            value​: 0,
        }
    }
}

// ZWSP creating invisible tokens
pub fn​ invisible​_tokens​() {
    let​ x​ =​ 42​;  // ZWSPs everywhere
    println!("{}", x​);
}
