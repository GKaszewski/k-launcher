pub use nucleo_matcher::Matcher;
use nucleo_matcher::{
    Config, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};

pub fn new_matcher() -> Matcher {
    Matcher::new(Config::DEFAULT)
}

pub fn parse_pattern(query: &str) -> Pattern {
    Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart)
}

fn matches_initials(name_lowercase: &str, query_lowercase: &str) -> bool {
    let mut initials = name_lowercase
        .split_whitespace()
        .filter_map(|w| w.chars().next());
    let mut query_chars = query_lowercase.chars();

    for expected in &mut query_chars {
        match initials.next() {
            Some(initial) if initial == expected => continue,
            _ => return false,
        }
    }
    true
}

const INITIALS_BONUS: u32 = 20;

pub fn score_match(
    matcher: &mut Matcher,
    pattern: &Pattern,
    name: &str,
    char_buf: &mut Vec<char>,
    name_lowercase: &str,
    query_lowercase: &str,
) -> Option<u32> {
    let haystack = Utf32Str::new(name, char_buf);
    let score = pattern.score(haystack, matcher)?;

    let bonus = if matches_initials(name_lowercase, query_lowercase) {
        INITIALS_BONUS
    } else {
        0
    };
    Some(score.saturating_add(bonus))
}

pub fn humanize_category(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch.is_uppercase() && !result.is_empty() {
            result.push(' ');
        }
        result.push(ch);
    }
    result
}
