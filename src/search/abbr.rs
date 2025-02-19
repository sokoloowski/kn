use std::cmp::Ordering;

use strsim::levenshtein as str_distance;

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub enum Abbr {
    Literal(String),
    Wildcard,
}

impl Abbr {
    pub fn from_string(pattern: String) -> Result<Self> {
        // Invalid characters: /, \, any whitespace.
        let is_invalid = |s: &str| {
            s.contains('/')
                || s.contains('\\')
                || s.contains(char::is_whitespace)
        };

        if pattern == "-" {
            Ok(Self::Wildcard)
        } else if is_invalid(&pattern) {
            Err(Error::InvalidAbbr(pattern))
        } else if pattern.is_empty() {
            Err(Error::InvalidAbbr(pattern))
        } else {
            Ok(Self::Literal(pattern.to_ascii_lowercase()))
        }
    }

    pub fn compare(&self, component: &str) -> Option<Congruence> {
        use Congruence::*;

        let component = component.to_ascii_lowercase();

        match self {
            Self::Literal(literal) =>
                if *literal == component.to_ascii_lowercase() {
                    Some(Complete)
                } else {
                    let mut abbr_chars = literal.chars().peekable();

                    for component_c in component.chars() {
                        match abbr_chars.peek() {
                            Some(abbr_c) =>
                                if *abbr_c == component_c.to_ascii_lowercase() {
                                    abbr_chars.next(); // Consume char.
                                },
                            None => break,
                        }
                    }

                    let whole_abbr_consumed = abbr_chars.peek().is_none();

                    if whole_abbr_consumed {
                        let distance = str_distance(literal, &component);

                        Some(Partial(distance))
                    } else {
                        None
                    }
                },
            Self::Wildcard => Some(Wildcard),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Congruence {
    Partial(usize),
    Wildcard,
    Complete,
}

impl PartialOrd for Congruence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Congruence {
    fn cmp(&self, other: &Self) -> Ordering {
        use Congruence::*;
        use Ordering::*;

        match (self, other) {
            (Complete, Complete) => Equal,
            (Complete, _) => Less,

            (Wildcard, Wildcard) => Equal,
            (Wildcard, _) => Greater,

            (Partial(_), Wildcard) => Less,
            (Partial(a), Partial(b)) => a.cmp(&b),
            (Partial(_), Complete) => Greater,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Congruence::*;

    #[test]
    fn test_congruence_ordering() {
        assert!(Partial(1) < Partial(2));
        assert!(Complete < Partial(1));
        assert!(Complete < Wildcard);
        assert!(Partial(1000) < Wildcard);
    }

    #[test]
    fn test_from_string() {
        let abbr = |s: &str| Abbr::from_string(s.to_string());

        assert!(abbr("one two three").is_err());
        assert!(abbr("du\tpa").is_err());
        assert!(abbr("\n").is_err());
        assert!(abbr(r"a\b").is_err());
        assert!(abbr("a/b").is_err());

        let abbr = |s: &str| Abbr::from_string(s.to_string()).unwrap();

        assert_variant!(abbr("-"), Abbr::Wildcard);
        assert_variant!(abbr("mOcKiNgBiRd"), Abbr::Literal(literal) if literal == "mockingbird");
        assert_variant!(abbr("X.ae.A-12"), Abbr::Literal(literal) if literal == "x.ae.a-12");
        assert_variant!(abbr("zażółć"), Abbr::Literal(literal) if literal == "zażółć")
    }

    #[test]
    fn test_wildcard_match() {
        let abbr = Abbr::Wildcard;

        assert_variant!(abbr.compare("iks"), Some(Wildcard));
        assert_variant!(abbr.compare("eh--ehe123"), Some(Wildcard));
    }

    #[test]
    fn test_literal_match() {
        let abbr = Abbr::Literal("mi".to_string());

        assert_variant!(abbr.compare("mi"), Some(Complete));
        assert_variant!(abbr.compare("Mi"), Some(Complete));
        assert_variant!(abbr.compare("ooo..oo---mi-ooooo"), Some(Partial(_)));
        assert_variant!(abbr.compare("ooo..oo---mI-ooooo"), Some(Partial(_)));
        assert_variant!(abbr.compare("xxxxxx"), None);
    }

    #[test]
    fn test_subseries_match() {
        let abbr = Abbr::Literal("mi".to_string());

        let dist_a = assert_variant!(abbr.compare("m-----i"), Some(Partial(dist_a)) => dist_a);
        let dist_b = assert_variant!(abbr.compare("M--i"), Some(Partial(dist_b)) => dist_b);
        assert!(dist_a > dist_b);

        assert_variant!(abbr.compare("im"), None);
    }
}
