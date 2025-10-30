// Invisible separator characters attack
// Various Unicode separators that are invisible: line separator, paragraph separator, etc.

// Line Separator (U+2028) in identifiers
pub fn get user() {  // Line separator between 'get' and 'user'
    println!("Line separator in function");
}

// Paragraph Separator (U+2029) in identifiers
pub fn process data() {  // Paragraph separator
    println!("Paragraph separator");
}

// Multiple invisible separators
pub fn multi ple sep arators() {
    let val ue = 42;
    println!("{}", val ue);
}

// Invisible separators creating confusion
pub fn confusion() {
    let data = "original";
    let data  = "with separator";  // Different due to invisible char
    println!("{}", data );
}

// Other invisible/whitespace characters
// Mongolian Vowel Separator (U+180E)
pub fn mongolian᠎separator() {
    println!("Mongolian separator");
}

// Ogham Space Mark (U+1680)
pub fn ogham space() {
    println!("Ogham space");
}

// En Quad (U+2000), Em Quad (U+2001), etc.
pub fn various spaces() {
    let en space = "value";
    let em space = "value";
    let thin space = "value";
    println!("{} {} {}", en space, em space, thin space);
}

// Non-breaking spaces (U+00A0)
pub fn non breaking() {
    let no break = "value";
    println!("{}", no break);
}

// Narrow no-break space (U+202F)
pub fn narrow space() {
    let narrow = "value";
    println!("{}", narrow);
}

// Medium mathematical space (U+205F)
pub fn math space() {
    let medium = "value";
    println!("{}", medium);
}

// Ideographic space (U+3000) - fullwidth space
pub fn ideographic　space() {  // Fullwidth space
    let　value　=　42;  // Fullwidth spaces
    println!("{}", value);
}

// Mix of different invisible separators
pub fn mixed separators() {
    let x  ᠎   = 42;  // Multiple different invisible chars
    println!("{}", x);
}
