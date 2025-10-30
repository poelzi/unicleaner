// Zero-Width No-Break Space / BOM (U+FEFF) attack
// BOM at start of file is normal, but in middle is suspicious

// BOM in identifiers
pub fn get﻿user() {  // BOM between 'get' and 'user'
    println!("BOM in function name");
}

pub fn process﻿data() {
    let user﻿name = "admin";
    let pass﻿word = "secret";
    println!("{} {}", user﻿name, pass﻿word);
}

// BOM creating duplicate identifiers
pub fn identifier﻿confusion() {
    let value = "original";
    let value﻿ = "with BOM";  // Different identifier!
    println!("{}", value﻿);
}

// BOM in struct names
pub struct User﻿Data {
    name﻿: String,
    age﻿: u32,
}

impl User﻿Data {
    pub fn new﻿() -> Self {
        User﻿Data {
            name﻿: String::new(),
            age﻿: 0,
        }
    }
}

// Multiple BOMs
pub fn multi﻿﻿﻿ple() {
    let x﻿﻿ = 42;
    println!("{}", x﻿﻿);
}

// BOM in comments
pub fn comment﻿test() {
    // Comment﻿with﻿BOMs
    /* Multi﻿line﻿comment */
    println!("Testing");
}
