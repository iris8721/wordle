use std::fmt::{self, Display, Formatter};

pub const WORD_LEN: usize = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Word([u8; WORD_LEN]);

impl Word {
    pub fn parse_upper_ascii(raw: &str) -> Option<Self> {
        if raw.len() != WORD_LEN {
            return None;
        }

        let bytes = raw.as_bytes();
        if !bytes
            .iter()
            .all(|byte| byte.is_ascii_alphabetic() && byte.is_ascii_uppercase())
        {
            return None;
        }

        let mut letters = [0u8; WORD_LEN];
        letters.copy_from_slice(bytes);
        Some(Self(letters))
    }

    pub fn parse_user_input(raw: &str) -> Option<Self> {
        let trimmed = raw.trim().to_ascii_uppercase();
        Self::parse_upper_ascii(&trimmed)
    }

    pub fn letters(self) -> [u8; WORD_LEN] {
        self.0
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(&self.0) {
            Ok(value) => f.write_str(value),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Word;

    #[test]
    fn parses_valid_words() {
        assert!(Word::parse_upper_ascii("CRANE").is_some());
        assert!(Word::parse_upper_ascii("crane").is_none());
        assert!(Word::parse_user_input("crane").is_some());
    }

    #[test]
    fn rejects_invalid_words() {
        assert!(Word::parse_upper_ascii("ABCD").is_none());
        assert!(Word::parse_upper_ascii("ABCD1").is_none());
    }
}
