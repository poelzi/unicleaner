// FSI Attack with actual Unicode characters
pub fn vulnerable_function() {
    let cmd = "ls -l";
    // The following line contains FSI (U+2068) and LRI (U+2066)
    println!("Executing: ⁨rm -rf /⁦");
}
