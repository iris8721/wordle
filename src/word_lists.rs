use crate::word::Word;
use std::collections::BTreeSet;

pub struct WordLists {
    pub answers: Vec<Word>,
    pub guesses: Vec<Word>,
}

pub fn load_word_lists() -> WordLists {
    const OFFICIAL_ANSWERS_TXT: &str = include_str!("../word_lists/officialanswers.txt");
    const OFFICIAL_GUESSES_TXT: &str = include_str!("../word_lists/officialguesses.txt");

    let answers = extract_word_set(OFFICIAL_ANSWERS_TXT);
    let mut guesses = extract_word_set(OFFICIAL_GUESSES_TXT);

    for answer in &answers {
        guesses.insert(*answer);
    }

    WordLists {
        answers: answers.into_iter().collect(),
        guesses: guesses.into_iter().collect(),
    }
}

fn extract_word_set(raw_text: &str) -> BTreeSet<Word> {
    raw_text
        .lines()
        .map(str::trim)
        .filter_map(Word::parse_upper_ascii)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::load_word_lists;

    #[test]
    fn loads_non_empty_lists() {
        let lists = load_word_lists();
        assert!(!lists.answers.is_empty());
        assert!(!lists.guesses.is_empty());
        assert!(lists.guesses.len() >= lists.answers.len());
    }
}
