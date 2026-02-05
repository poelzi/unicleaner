//! Unicode block registry for resolving block names to code point ranges

use crate::unicode::ranges::UnicodeRange;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;

/// Entry representing a Unicode block with its name and code point boundaries
#[derive(Debug, Clone)]
pub struct BlockEntry {
    pub name: String,
    pub start: u32,
    pub end: u32,
}

/// Information about a Unicode block for listing purposes
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub aliases: Vec<String>,
}

/// Error type for block resolution failures
#[derive(Debug)]
pub enum BlockError {
    /// Block name not found, includes similar name suggestions
    UnknownBlock {
        name: String,
        suggestions: Vec<String>,
    },
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockError::UnknownBlock { name, suggestions } => {
                write!(f, "Unknown Unicode block \"{}\"", name)?;
                if !suggestions.is_empty() {
                    write!(f, "\n  Did you mean: \"{}\"?", suggestions[0])?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for BlockError {}

/// Macro to register all unicode-blocks crate constants into the registry HashMap.
/// Each constant is a `unicode_blocks::UnicodeBlock` with `name()`, `start()`, `end()`.
macro_rules! register_blocks {
    ($map:ident, $( $const_name:ident ),* $(,)?) => {
        $(
            let block = unicode_blocks::$const_name;
            $map.insert(
                block.name().to_lowercase(),
                BlockEntry {
                    name: block.name().to_string(),
                    start: block.start(),
                    end: block.end(),
                },
            );
        )*
    };
}

/// The global block registry: lowercase name/alias → BlockEntry
static REGISTRY: Lazy<HashMap<String, BlockEntry>> = Lazy::new(|| {
    let mut map = HashMap::new();

    register_blocks!(
        map,
        BASIC_LATIN,
        LATIN_1_SUPPLEMENT,
        LATIN_EXTENDED_A,
        LATIN_EXTENDED_B,
        IPA_EXTENSIONS,
        SPACING_MODIFIER_LETTERS,
        COMBINING_DIACRITICAL_MARKS,
        GREEK_AND_COPTIC,
        CYRILLIC,
        CYRILLIC_SUPPLEMENT,
        ARMENIAN,
        HEBREW,
        ARABIC,
        SYRIAC,
        ARABIC_SUPPLEMENT,
        THAANA,
        NKO,
        SAMARITAN,
        MANDAIC,
        SYRIAC_SUPPLEMENT,
        ARABIC_EXTENDED_B,
        ARABIC_EXTENDED_A,
        DEVANAGARI,
        BENGALI,
        GURMUKHI,
        GUJARATI,
        ORIYA,
        TAMIL,
        TELUGU,
        KANNADA,
        MALAYALAM,
        SINHALA,
        THAI,
        LAO,
        TIBETAN,
        MYANMAR,
        GEORGIAN,
        HANGUL_JAMO,
        ETHIOPIC,
        ETHIOPIC_SUPPLEMENT,
        CHEROKEE,
        UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS,
        OGHAM,
        RUNIC,
        TAGALOG,
        HANUNOO,
        BUHID,
        TAGBANWA,
        KHMER,
        MONGOLIAN,
        UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS_EXTENDED,
        LIMBU,
        TAI_LE,
        NEW_TAI_LUE,
        KHMER_SYMBOLS,
        BUGINESE,
        TAI_THAM,
        COMBINING_DIACRITICAL_MARKS_EXTENDED,
        BALINESE,
        SUNDANESE,
        BATAK,
        LEPCHA,
        OL_CHIKI,
        CYRILLIC_EXTENDED_C,
        GEORGIAN_EXTENDED,
        SUNDANESE_SUPPLEMENT,
        VEDIC_EXTENSIONS,
        PHONETIC_EXTENSIONS,
        PHONETIC_EXTENSIONS_SUPPLEMENT,
        COMBINING_DIACRITICAL_MARKS_SUPPLEMENT,
        LATIN_EXTENDED_ADDITIONAL,
        GREEK_EXTENDED,
        GENERAL_PUNCTUATION,
        SUPERSCRIPTS_AND_SUBSCRIPTS,
        CURRENCY_SYMBOLS,
        COMBINING_DIACRITICAL_MARKS_FOR_SYMBOLS,
        LETTERLIKE_SYMBOLS,
        NUMBER_FORMS,
        ARROWS,
        MATHEMATICAL_OPERATORS,
        MISCELLANEOUS_TECHNICAL,
        CONTROL_PICTURES,
        OPTICAL_CHARACTER_RECOGNITION,
        ENCLOSED_ALPHANUMERICS,
        BOX_DRAWING,
        BLOCK_ELEMENTS,
        GEOMETRIC_SHAPES,
        MISCELLANEOUS_SYMBOLS,
        DINGBATS,
        MISCELLANEOUS_MATHEMATICAL_SYMBOLS_A,
        SUPPLEMENTAL_ARROWS_A,
        BRAILLE_PATTERNS,
        SUPPLEMENTAL_ARROWS_B,
        MISCELLANEOUS_MATHEMATICAL_SYMBOLS_B,
        SUPPLEMENTAL_MATHEMATICAL_OPERATORS,
        MISCELLANEOUS_SYMBOLS_AND_ARROWS,
        GLAGOLITIC,
        LATIN_EXTENDED_C,
        COPTIC,
        GEORGIAN_SUPPLEMENT,
        TIFINAGH,
        ETHIOPIC_EXTENDED,
        CYRILLIC_EXTENDED_A,
        SUPPLEMENTAL_PUNCTUATION,
        CJK_RADICALS_SUPPLEMENT,
        KANGXI_RADICALS,
        IDEOGRAPHIC_DESCRIPTION_CHARACTERS,
        CJK_SYMBOLS_AND_PUNCTUATION,
        HIRAGANA,
        KATAKANA,
        BOPOMOFO,
        HANGUL_COMPATIBILITY_JAMO,
        KANBUN,
        BOPOMOFO_EXTENDED,
        CJK_STROKES,
        KATAKANA_PHONETIC_EXTENSIONS,
        ENCLOSED_CJK_LETTERS_AND_MONTHS,
        CJK_COMPATIBILITY,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_A,
        YIJING_HEXAGRAM_SYMBOLS,
        CJK_UNIFIED_IDEOGRAPHS,
        YI_SYLLABLES,
        YI_RADICALS,
        LISU,
        VAI,
        CYRILLIC_EXTENDED_B,
        BAMUM,
        MODIFIER_TONE_LETTERS,
        LATIN_EXTENDED_D,
        SYLOTI_NAGRI,
        COMMON_INDIC_NUMBER_FORMS,
        PHAGS_PA,
        SAURASHTRA,
        DEVANAGARI_EXTENDED,
        KAYAH_LI,
        REJANG,
        HANGUL_JAMO_EXTENDED_A,
        JAVANESE,
        MYANMAR_EXTENDED_B,
        CHAM,
        MYANMAR_EXTENDED_A,
        TAI_VIET,
        MEETEI_MAYEK_EXTENSIONS,
        ETHIOPIC_EXTENDED_A,
        LATIN_EXTENDED_E,
        CHEROKEE_SUPPLEMENT,
        MEETEI_MAYEK,
        HANGUL_SYLLABLES,
        HANGUL_JAMO_EXTENDED_B,
        HIGH_SURROGATES,
        HIGH_PRIVATE_USE_SURROGATES,
        LOW_SURROGATES,
        PRIVATE_USE_AREA,
        CJK_COMPATIBILITY_IDEOGRAPHS,
        ALPHABETIC_PRESENTATION_FORMS,
        ARABIC_PRESENTATION_FORMS_A,
        VARIATION_SELECTORS,
        VERTICAL_FORMS,
        COMBINING_HALF_MARKS,
        CJK_COMPATIBILITY_FORMS,
        SMALL_FORM_VARIANTS,
        ARABIC_PRESENTATION_FORMS_B,
        HALFWIDTH_AND_FULLWIDTH_FORMS,
        SPECIALS,
        LINEAR_B_SYLLABARY,
        LINEAR_B_IDEOGRAMS,
        AEGEAN_NUMBERS,
        ANCIENT_GREEK_NUMBERS,
        ANCIENT_SYMBOLS,
        PHAISTOS_DISC,
        LYCIAN,
        CARIAN,
        COPTIC_EPACT_NUMBERS,
        OLD_ITALIC,
        GOTHIC,
        OLD_PERMIC,
        UGARITIC,
        OLD_PERSIAN,
        DESERET,
        SHAVIAN,
        OSMANYA,
        OSAGE,
        ELBASAN,
        CAUCASIAN_ALBANIAN,
        VITHKUQI,
        LINEAR_A,
        LATIN_EXTENDED_F,
        CYPRIOT_SYLLABARY,
        IMPERIAL_ARAMAIC,
        PALMYRENE,
        NABATAEAN,
        HATRAN,
        PHOENICIAN,
        LYDIAN,
        MEROITIC_HIEROGLYPHS,
        MEROITIC_CURSIVE,
        KHAROSHTHI,
        OLD_SOUTH_ARABIAN,
        OLD_NORTH_ARABIAN,
        MANICHAEAN,
        AVESTAN,
        INSCRIPTIONAL_PARTHIAN,
        INSCRIPTIONAL_PAHLAVI,
        PSALTER_PAHLAVI,
        OLD_TURKIC,
        OLD_HUNGARIAN,
        HANIFI_ROHINGYA,
        RUMI_NUMERAL_SYMBOLS,
        YEZIDI,
        ARABIC_EXTENDED_C,
        OLD_SOGDIAN,
        SOGDIAN,
        OLD_UYGHUR,
        CHORASMIAN,
        ELYMAIC,
        BRAHMI,
        KAITHI,
        SORA_SOMPENG,
        CHAKMA,
        MAHAJANI,
        SHARADA,
        SINHALA_ARCHAIC_NUMBERS,
        KHOJKI,
        MULTANI,
        KHUDAWADI,
        GRANTHA,
        NEWA,
        TIRHUTA,
        SIDDHAM,
        MODI,
        MONGOLIAN_SUPPLEMENT,
        TAKRI,
        AHOM,
        DOGRA,
        WARANG_CITI,
        DIVES_AKURU,
        NANDINAGARI,
        ZANABAZAR_SQUARE,
        SOYOMBO,
        UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS_EXTENDED_A,
        PAU_CIN_HAU,
        DEVANAGARI_EXTENDED_A,
        BHAIKSUKI,
        MARCHEN,
        MASARAM_GONDI,
        GUNJALA_GONDI,
        MAKASAR,
        KAWI,
        LISU_SUPPLEMENT,
        TAMIL_SUPPLEMENT,
        CUNEIFORM,
        CUNEIFORM_NUMBERS_AND_PUNCTUATION,
        EARLY_DYNASTIC_CUNEIFORM,
        CYPRO_MINOAN,
        EGYPTIAN_HIEROGLYPHS,
        EGYPTIAN_HIEROGLYPH_FORMAT_CONTROLS,
        ANATOLIAN_HIEROGLYPHS,
        BAMUM_SUPPLEMENT,
        MRO,
        TANGSA,
        BASSA_VAH,
        PAHAWH_HMONG,
        MEDEFAIDRIN,
        MIAO,
        IDEOGRAPHIC_SYMBOLS_AND_PUNCTUATION,
        TANGUT,
        TANGUT_COMPONENTS,
        KHITAN_SMALL_SCRIPT,
        TANGUT_SUPPLEMENT,
        KANA_EXTENDED_B,
        KANA_SUPPLEMENT,
        KANA_EXTENDED_A,
        SMALL_KANA_EXTENSION,
        NUSHU,
        DUPLOYAN,
        SHORTHAND_FORMAT_CONTROLS,
        ZNAMENNY_MUSICAL_NOTATION,
        BYZANTINE_MUSICAL_SYMBOLS,
        MUSICAL_SYMBOLS,
        ANCIENT_GREEK_MUSICAL_NOTATION,
        KAKTOVIK_NUMERALS,
        MAYAN_NUMERALS,
        TAI_XUAN_JING_SYMBOLS,
        COUNTING_ROD_NUMERALS,
        MATHEMATICAL_ALPHANUMERIC_SYMBOLS,
        SUTTON_SIGNWRITING,
        LATIN_EXTENDED_G,
        GLAGOLITIC_SUPPLEMENT,
        CYRILLIC_EXTENDED_D,
        NYIAKENG_PUACHUE_HMONG,
        TOTO,
        WANCHO,
        NAG_MUNDARI,
        ETHIOPIC_EXTENDED_B,
        MENDE_KIKAKUI,
        ADLAM,
        INDIC_SIYAQ_NUMBERS,
        OTTOMAN_SIYAQ_NUMBERS,
        ARABIC_MATHEMATICAL_ALPHABETIC_SYMBOLS,
        MAHJONG_TILES,
        DOMINO_TILES,
        PLAYING_CARDS,
        ENCLOSED_ALPHANUMERIC_SUPPLEMENT,
        ENCLOSED_IDEOGRAPHIC_SUPPLEMENT,
        MISCELLANEOUS_SYMBOLS_AND_PICTOGRAPHS,
        EMOTICONS,
        ORNAMENTAL_DINGBATS,
        TRANSPORT_AND_MAP_SYMBOLS,
        ALCHEMICAL_SYMBOLS,
        GEOMETRIC_SHAPES_EXTENDED,
        SUPPLEMENTAL_ARROWS_C,
        SUPPLEMENTAL_SYMBOLS_AND_PICTOGRAPHS,
        CHESS_SYMBOLS,
        SYMBOLS_AND_PICTOGRAPHS_EXTENDED_A,
        SYMBOLS_FOR_LEGACY_COMPUTING,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_B,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_C,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_D,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_E,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_F,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_I,
        CJK_COMPATIBILITY_IDEOGRAPHS_SUPPLEMENT,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_G,
        CJK_UNIFIED_IDEOGRAPHS_EXTENSION_H,
        TAGS,
        VARIATION_SELECTORS_SUPPLEMENT,
        SUPPLEMENTARY_PRIVATE_USE_AREA_A,
        SUPPLEMENTARY_PRIVATE_USE_AREA_B,
    );

    // Add short aliases (FR-004)
    let aliases: &[(&str, &str)] = &[
        ("ascii", "basic latin"),
        ("latin-1", "latin-1 supplement"),
        ("latin-extended-a", "latin extended-a"),
        ("latin-extended-b", "latin extended-b"),
        ("greek", "greek and coptic"),
        ("cyrillic", "cyrillic"),
        ("hebrew", "hebrew"),
        ("arabic", "arabic"),
        ("cjk", "cjk unified ideographs"),
        ("hangul", "hangul syllables"),
        ("hiragana", "hiragana"),
        ("katakana", "katakana"),
        ("emoji", "emoticons"),
    ];

    for (alias, official_lower) in aliases {
        if let Some(entry) = map.get(*official_lower) {
            let alias_entry = entry.clone();
            map.insert(alias.to_string(), alias_entry);
        }
    }

    map
});

/// Reverse map: lowercase name → official name (for suggestions)
static ALL_NAMES: Lazy<Vec<String>> = Lazy::new(|| {
    let mut names: Vec<String> = REGISTRY.values().map(|entry| entry.name.clone()).collect();
    names.sort();
    names.dedup();
    names
});

/// Registry for looking up Unicode blocks by name
pub struct BlockRegistry;

impl BlockRegistry {
    /// Resolve a block name (official or alias) to a UnicodeRange.
    /// Lookup is case-insensitive.
    pub fn resolve(name: &str) -> Result<UnicodeRange, BlockError> {
        let key = name.to_lowercase();
        match REGISTRY.get(&key) {
            Some(entry) => Ok(UnicodeRange::with_description(
                entry.start,
                entry.end,
                entry.name.clone(),
            )),
            None => {
                let suggestions = Self::suggest(name);
                Err(BlockError::UnknownBlock {
                    name: name.to_string(),
                    suggestions,
                })
            }
        }
    }

    /// Suggest similar block names for a given input using Jaro-Winkler distance
    pub fn suggest(name: &str) -> Vec<String> {
        let input = name.to_lowercase();
        let mut scored: Vec<(f64, &str)> = ALL_NAMES
            .iter()
            .map(|n| (strsim::jaro_winkler(&input, &n.to_lowercase()), n.as_str()))
            .filter(|(score, _)| *score > 0.7)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(3)
            .map(|(_, n)| n.to_string())
            .collect()
    }

    /// List all available blocks, optionally filtered by case-insensitive substring match.
    /// Returns blocks sorted by start code point.
    pub fn list_blocks(filter: Option<&str>) -> Vec<BlockInfo> {
        // Collect unique blocks (skip alias duplicates)
        let mut seen = std::collections::HashSet::new();
        let mut blocks: Vec<BlockInfo> = Vec::new();

        for entry in REGISTRY.values() {
            if seen.contains(&entry.start) {
                continue;
            }
            seen.insert(entry.start);

            // Collect aliases for this block
            let aliases: Vec<String> = REGISTRY
                .iter()
                .filter(|(key, e)| {
                    e.start == entry.start && key.as_str() != entry.name.to_lowercase()
                })
                .map(|(key, _)| key.clone())
                .collect();

            blocks.push(BlockInfo {
                name: entry.name.clone(),
                start: entry.start,
                end: entry.end,
                aliases,
            });
        }

        blocks.sort_by_key(|b| b.start);

        if let Some(filter) = filter {
            let filter_lower = filter.to_lowercase();
            blocks.retain(|b| b.name.to_lowercase().contains(&filter_lower));
        }

        blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_official_name() {
        let range = BlockRegistry::resolve("Basic Latin").unwrap();
        assert_eq!(range.start, 0x0000);
        assert_eq!(range.end, 0x007F);
    }

    #[test]
    fn test_resolve_case_insensitive_lowercase() {
        let range = BlockRegistry::resolve("basic latin").unwrap();
        assert_eq!(range.start, 0x0000);
        assert_eq!(range.end, 0x007F);
    }

    #[test]
    fn test_resolve_case_insensitive_uppercase() {
        let range = BlockRegistry::resolve("BASIC LATIN").unwrap();
        assert_eq!(range.start, 0x0000);
        assert_eq!(range.end, 0x007F);
    }

    #[test]
    fn test_resolve_hebrew() {
        let range = BlockRegistry::resolve("Hebrew").unwrap();
        assert_eq!(range.start, 0x0590);
        assert_eq!(range.end, 0x05FF);
    }

    #[test]
    fn test_resolve_unknown_block_returns_error() {
        let result = BlockRegistry::resolve("Nonexistent");
        assert!(result.is_err());
        if let Err(BlockError::UnknownBlock { name, suggestions }) = result {
            assert_eq!(name, "Nonexistent");
            assert!(!suggestions.is_empty(), "Should suggest similar names");
        }
    }

    #[test]
    fn test_suggest_similar_names() {
        let suggestions = BlockRegistry::suggest("Hewbrew");
        assert!(!suggestions.is_empty());
        assert!(
            suggestions.iter().any(|s| s == "Hebrew"),
            "Should suggest 'Hebrew' for 'Hewbrew'"
        );
    }

    #[test]
    fn test_block_error_display() {
        let err = BlockError::UnknownBlock {
            name: "Hewbrew".to_string(),
            suggestions: vec!["Hebrew".to_string()],
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Hewbrew"));
        assert!(msg.contains("Hebrew"));
    }

    #[test]
    fn test_registry_has_all_blocks() {
        // The unicode-blocks crate has 328 constants
        // Our registry should have at least that many entries
        assert!(
            REGISTRY.len() >= 320,
            "Registry should have 320+ blocks, got {}",
            REGISTRY.len()
        );
    }

    // T017: Alias tests
    #[test]
    fn test_resolve_alias_ascii() {
        let range = BlockRegistry::resolve("ascii").unwrap();
        assert_eq!(range.start, 0x0000);
        assert_eq!(range.end, 0x007F);
    }

    #[test]
    fn test_resolve_alias_latin1() {
        let range = BlockRegistry::resolve("latin-1").unwrap();
        assert_eq!(range.start, 0x0080);
        assert_eq!(range.end, 0x00FF);
    }

    #[test]
    fn test_resolve_alias_hebrew() {
        let range = BlockRegistry::resolve("hebrew").unwrap();
        // Should resolve to same range as official "Hebrew"
        let official = BlockRegistry::resolve("Hebrew").unwrap();
        assert_eq!(range.start, official.start);
        assert_eq!(range.end, official.end);
    }

    #[test]
    fn test_resolve_alias_cjk() {
        let range = BlockRegistry::resolve("cjk").unwrap();
        assert_eq!(range.start, 0x4E00);
        assert_eq!(range.end, 0x9FFF);
    }

    #[test]
    fn test_resolve_alias_emoji() {
        let range = BlockRegistry::resolve("emoji").unwrap();
        assert_eq!(range.start, 0x1F600);
        assert_eq!(range.end, 0x1F64F);
    }

    // T021: list_blocks tests
    #[test]
    fn test_list_blocks_returns_all_sorted() {
        let blocks = BlockRegistry::list_blocks(None);
        assert!(
            blocks.len() >= 320,
            "Should list 320+ blocks, got {}",
            blocks.len()
        );
        // Verify sorted by start code point
        for window in blocks.windows(2) {
            assert!(
                window[0].start <= window[1].start,
                "Blocks should be sorted by start: {} (U+{:04X}) > {} (U+{:04X})",
                window[0].name,
                window[0].start,
                window[1].name,
                window[1].start
            );
        }
    }

    #[test]
    fn test_list_blocks_filter_hebrew() {
        let blocks = BlockRegistry::list_blocks(Some("hebrew"));
        assert!(
            !blocks.is_empty(),
            "Filter 'hebrew' should return at least one block"
        );
        for block in &blocks {
            assert!(
                block.name.to_lowercase().contains("hebrew"),
                "Filtered block '{}' should contain 'hebrew'",
                block.name
            );
        }
    }

    #[test]
    fn test_list_blocks_filter_no_match() {
        let blocks = BlockRegistry::list_blocks(Some("zzzzz"));
        assert!(blocks.is_empty(), "Filter 'zzzzz' should return no blocks");
    }

    #[test]
    fn test_list_blocks_first_is_basic_latin() {
        let blocks = BlockRegistry::list_blocks(None);
        assert_eq!(blocks[0].name, "Basic Latin");
        assert_eq!(blocks[0].start, 0x0000);
        assert_eq!(blocks[0].end, 0x007F);
    }
}
