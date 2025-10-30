// Zero-Width Joiner (ZWJ U+200D) attack
// ZWJ is used for emoji sequences and some scripts, but can be abused

// ZWJ in identifiers
pub fn process‍data() {
    // ZWJ between 'process' and 'data'
    println!("ZWJ in function name");
}

pub fn get‍user‍info() {
    // Multiple ZWJs
    let user‍name = "admin";
    let pass‍word = "secret";
    println!("{} {}", user‍name, pass‍word);
}

// ZWJ creating identifier confusion
pub fn variable‍confusion() {
    let value = "first";
    let value‍ = "second"; // ZWJ makes this different
    println!("{}", value‍);
}

// ZWJ in struct and field names
pub struct Data‍Processor {
    state‍: i32,
    result‍: String,
}

impl Data‍Processor {
    pub fn process‍() {
        println!("Processing with ZWJ");
    }
}

// Multiple ZWJs in sequence
pub fn heavy‍‍‍usage() {
    let x‍‍‍ = 42;
    println!("{}", x‍‍‍);
}

// ZWJ in comments
pub fn comment‍test() {
    // This‍comment‍has‍ZWJs
    /* ZWJ‍in‍multiline */
    println!("Comments");
}
