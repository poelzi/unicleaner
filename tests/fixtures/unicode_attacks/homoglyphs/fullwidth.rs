// Fullwidth character attack - East Asian fullwidth forms
// Fullwidth 'Ａ'-'Ｚ' (U+FF21-U+FF3A) look like normal ASCII but are different
// Fullwidth 'ａ'-'ｚ' (U+FF41-U+FF5A)
// Fullwidth digits '０'-'９' (U+FF10-U+FF19)

// Fullwidth Latin letters
pub fn ｇｅｔ＿ｕｓｅｒ() {  // All fullwidth characters
    println!("Function uses fullwidth characters");
}

pub fn ＶＡＬＩＤＡＴＥ() {  // Fullwidth uppercase
    println!("Fullwidth uppercase function");
}

// Mixed halfwidth and fullwidth - very deceptive
pub fn process_ｄata() {  // 'ｄ' is fullwidth, rest is normal
    let useｒ = "admin";  // 'ｒ' is fullwidth
    println!("{}", useｒ);
}

// Fullwidth digits
pub fn calculate() {
    let value１ = 100;  // '１' is fullwidth digit
    let value２ = 200;  // '２' is fullwidth digit
    let total = value１ + value２;
    println!("Total: {}", total);
}

// Fullwidth punctuation and symbols
pub fn check＿status（） {  // '＿' is fullwidth underscore, '（）' are fullwidth parens
    let status ＝ true；  // '＝' is fullwidth equals, '；' is fullwidth semicolon
    println！（"Status： {}"， status）；  // All fullwidth punctuation
}

// Mixed fullwidth in variable names
pub fn confusing_example() {
    let data = "normal";
    let ｄata = "fullwidth_d";  // Different variable!
    let daｔa = "fullwidth_t";  // Another different variable!

    println!("{} {} {}", data, ｄata, daｔa);
}

// Fullwidth spaces (U+3000) - invisible difference
pub fn　space_confusion() {  // Fullwidth space before function name
    let　value　=　42;  // Fullwidth spaces around identifiers
    println!("{}", value);
}
