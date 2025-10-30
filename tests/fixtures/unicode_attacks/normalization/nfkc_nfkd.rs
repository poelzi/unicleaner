// NFKC/NFKD compatibility normalization fixtures (T069)
// Tests for compatibility normalization attacks

// Compatibility characters that normalize differently
pub fn compat_numbers() {
    let normal = "123"; // ASCII digits
    let super = "¹²³"; // Superscript (compatibility)
    let circled = "①②③"; // Circled (compatibility)

    println!("{} {} {}", normal, super, circled);
}

// Fullwidth vs halfwidth compatibility
pub fn compat_latin() {
    let half = "test"; // Halfwidth ASCII
    let full = "ｔｅｓｔ"; // Fullwidth (compatibility chars)

    // NFKC normalizes full to half
    println!("{} {}", half, full);
}

// Ligatures and compatibility
pub fn compat_ligatures() {
    let normal = "fi"; // f + i
    let ligature = "ﬁ"; // fi ligature (U+FB01)

    // NFKC normalizes ligature to f + i
    println!("{} {}", normal, ligature);
}

// Roman numerals compatibility
pub fn compat_roman() {
    let ascii = "IV"; // I + V
    let compat = "Ⅳ"; // Roman numeral four (U+2163)

    println!("{} {}", ascii, compat);
}

// Fractions compatibility
pub fn compat_fractions() {
    let text = "1/2"; // Text representation
    let compat = "½"; // Vulgar fraction (U+00BD)

    println!("{} {}", text, compat);
}

// Mathematical operators compatibility
pub fn compat_math() {
    let ascii = "<=";
    let compat = "≤"; // Less than or equal (U+2264)

    println!("{} {}", ascii, compat);
}

// Identifier confusion via compatibility
pub fn process_data() {
    // ASCII
    println!("ASCII version");
}

pub fn ｐｒｏｃｅｓｓ_data() {
    // Fullwidth 'p'
    println!("Fullwidth version");
}
