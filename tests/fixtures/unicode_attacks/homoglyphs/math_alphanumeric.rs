// Mathematical alphanumeric symbols attack
// Unicode has special mathematical variants: 𝐀-𝐙 (U+1D400-U+1D419) bold capitals
// 𝑎-𝑧 (U+1D44E-U+1D467) italic lowercase
// These look very similar to normal ASCII letters but are different codepoints

// Bold mathematical letters (U+1D400 range)
pub fn 𝐠𝐞𝐭_𝐝𝐚𝐭𝐚() {
    // All letters are mathematical bold (U+1D420-U+1D42D range)
    println!("Function name uses mathematical bold characters");
}

// Italic mathematical letters (U+1D44E range)
pub fn 𝑎𝑢𝑡ℎ𝑒𝑛𝑡𝑖𝑐𝑎𝑡𝑒() {
    // Mathematical italic letters
    println!("Authentication with math italic");
}

// Bold italic (U+1D468 range)
pub fn 𝒗𝒂𝒍𝒊𝒅𝒂𝒕𝒆() {
    println!("Validation with mathematical bold italic");
}

// Script style (U+1D4B6 range)
pub fn 𝓅𝓇ℴ𝒸ℯ𝓈𝓈() {
    println!("Processing with mathematical script");
}

// Double-struck (blackboard bold) (U+1D538 range)
pub fn 𝕔𝕒𝕝𝕔𝕦𝕝𝕒𝕥𝕖() {
    println!("Calculate with double-struck letters");
}

// Sans-serif (U+1D5A0 range)
pub fn 𝗲𝘅𝗲𝗰𝘂𝘁𝗲() {
    println!("Execute with sans-serif mathematical");
}

// Monospace (U+1D670 range)
pub fn 𝚌𝚘𝚖𝚙𝚒𝚕𝚎() {
    println!("Compile with monospace mathematical");
}

// Mixed normal and mathematical - very deceptive
pub fn process_d𝑎ta() {
    // 𝑎 is mathematical italic
    let us𝑒r = "admin"; // 𝑒 is mathematical italic
    println!("{}", us𝑒r);
}

// Numbers can also be mathematical variants
pub fn calculate() {
    let value𝟏 = 100; // 𝟏 is mathematical bold digit
    let value𝟐 = 200; // 𝟐 is mathematical bold digit
    println!("{} + {} = {}", value𝟏, value𝟐, value𝟏 + value𝟐);
}
