// Combining character abuse - stacking diacriticals to hide code
// Combining marks (U+0300-U+036F) can be stacked excessively

// Excessive combining marks
pub fn get_ủs̴̗̃ẽ̵̢r̷̰̎() {
    // Multiple combining marks on letters
    println!("Combining character abuse");
}

// Combining marks creating confusable identifiers
pub fn normal_function() {
    println!("Normal");
}

pub fn nőrm̃ãl̂_fưnc̈tïön() {
    // Looks similar but has combining marks
    println!("With combining marks");
}

// Stacked combining marks (Zalgo text style)
pub fn p̴̧̛̗̝̱̩̈́͋͝r̸̡̢̰̪͙̿̀̈́͝o̷̧̱͔̪̐̄̍̕c̵̢̨̛̗͇̈́̾̕e̸̛͕̰̫̋̔̕͜s̷̨̧̛̰̀̄̚s̵̢̰̗̝͐̈́̄̕() {
    let d̸̛̗̰͐̕a̸̧̛͙̿̚t̷̢̝̎̕a̸̧̛̗͐ = "zalgo";
    println!("{}", d̸̛̗̰͐̕a̸̧̛͙̿̚t̷̢̝̎̕a̸̧̛̗͐);
}

// Combining marks in variable names
pub fn variable_confusion() {
    let value = "original";
    let vãlue = "with tilde"; // Combining tilde
    let v̈alue = "with umlaut"; // Combining diaeresis
    let v̄alue = "with macron"; // Combining macron

    println!("{}", vãlue);
}

// Combining marks in struct fields
pub struct Ūsër_Dätä {
    ñamë: String,
    ägę: u32,
}

impl Ūsër_Dätä {
    pub fn nëw() -> Self {
        Ūsër_Dätä {
            ñamë: String::new(),
            ägę: 0,
        }
    }
}

// Invisible combining marks (combining grapheme joiner, etc.)
pub fn invisible͏_combining() {
    // U+034F combining grapheme joiner
    let data͏ = "value"; // Invisible combining mark
    println!("{}", data͏);
}

// Multiple combining marks on single character
pub fn stacked() {
    let e̷̡̨̛̛̗̰̱̝̪̿̀̈́̐̄͋̾̍̕͝ = "extreme"; // Many marks on 'e'
    println!("{}", e̷̡̨̛̛̗̰̱̝̪̿̀̈́̐̄͋̾̍̕͝);
}
