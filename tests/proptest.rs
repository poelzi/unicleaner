// Property-based tests entry point

#[path = "proptest/bidi_properties.rs"]
mod bidi_properties;

#[path = "proptest/config_properties.rs"]
mod config_properties;

#[path = "proptest/encoding_properties.rs"]
mod encoding_properties;

#[path = "proptest/homoglyph_properties.rs"]
mod homoglyph_properties;

#[path = "proptest/normalization.rs"]
mod normalization;

#[path = "proptest/scanner_determinism.rs"]
mod scanner_determinism;

#[path = "proptest/scanner_stability.rs"]
mod scanner_stability;

#[path = "proptest/unicode_categories.rs"]
mod unicode_categories;

#[path = "proptest/unicode_ranges.rs"]
mod unicode_ranges;
