use crate::word::{WORD_LEN, Word};
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};

const NUM_PATTERNS: usize = 243; // 3^5 (B/Y/G per tile)
const SOLVED_PATTERN: u16 = 242; // GGGGG in base-3 encoding

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Pattern(u16);

impl Pattern {
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() != WORD_LEN {
            return Err("Pattern must be exactly 5 chars using B/Y/G.".to_string());
        }

        let mut value = 0u16;
        let mut factor = 1u16;
        for symbol in raw.chars() {
            let encoded = match symbol.to_ascii_uppercase() {
                'B' | 'X' | '0' => 0u16,
                'Y' | '1' => 1u16,
                'G' | '2' => 2u16,
                _ => return Err("Pattern can only contain B, Y, or G.".to_string()),
            };
            value += encoded * factor;
            factor *= 3;
        }

        Ok(Self(value))
    }

    fn from_marks(marks: [u8; WORD_LEN]) -> Self {
        let mut value = 0u16;
        let mut factor = 1u16;
        for mark in marks {
            value += u16::from(mark) * factor;
            factor *= 3;
        }
        Self(value)
    }

    fn marks(self) -> [u8; WORD_LEN] {
        let mut value = self.0;
        let mut marks = [0u8; WORD_LEN];
        for mark in &mut marks {
            *mark = (value % 3) as u8;
            value /= 3;
        }
        marks
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn is_solved(self) -> bool {
        self.0 == SOLVED_PATTERN
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for mark in self.marks() {
            let symbol = match mark {
                0 => 'B',
                1 => 'Y',
                2 => 'G',
                _ => return Err(fmt::Error),
            };
            f.write_str(&symbol.to_string())?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GuessFeedback {
    pub guess: Word,
    pub pattern: Pattern,
}

#[derive(Clone, Debug)]
pub struct Suggestion {
    pub word: Word,
    pub guesses: f64,
    pub adjusted_score: f64,
    pub expected_remaining: f64,
    pub solve_next_rate: f64,
    pub is_candidate: bool,
}

pub struct Solver {
    answers: Vec<Word>,
    guesses: Vec<Word>,
    answer_set: HashSet<Word>,
    guess_set: HashSet<Word>,
}

impl Solver {
    pub fn new(answers: Vec<Word>, guesses: Vec<Word>) -> Self {
        let answer_set = answers.iter().copied().collect::<HashSet<_>>();
        let guess_set = guesses.iter().copied().collect::<HashSet<_>>();

        Self {
            answers,
            guesses,
            answer_set,
            guess_set,
        }
    }

    pub fn is_valid_guess(&self, word: Word) -> bool {
        self.guess_set.contains(&word)
    }

    pub fn is_possible_answer(&self, word: Word) -> bool {
        self.answer_set.contains(&word)
    }

    pub fn candidates(&self, history: &[GuessFeedback]) -> Vec<Word> {
        if history.is_empty() {
            return self.answers.clone();
        }

        self.answers
            .iter()
            .copied()
            .filter(|candidate| {
                history
                    .iter()
                    .all(|entry| feedback(entry.guess, *candidate) == entry.pattern)
            })
            .collect()
    }

    pub fn suggestions(
        &self,
        history: &[GuessFeedback],
        candidates: &[Word],
        hard_mode: bool,
        top_n: usize,
    ) -> Vec<Suggestion> {
        if candidates.is_empty() || top_n == 0 {
            return Vec::new();
        }

        let candidate_set = candidates.iter().copied().collect::<HashSet<_>>();
        if candidates.len() <= 2 {
            return candidates
                .iter()
                .copied()
                .take(top_n)
                .map(|word| score_guess(word, candidates, &candidate_set))
                .collect();
        }

        let guess_pool = if hard_mode {
            self.guesses
                .iter()
                .copied()
                .filter(|&guess| obeys_hard_mode(guess, history))
                .collect()
        } else {
            self.guesses.clone()
        };

        let mut scored = reduce_guess_pool(guess_pool, candidates)
            .into_iter()
            .map(|guess| score_guess(guess, candidates, &candidate_set))
            .collect::<Vec<_>>();

        scored.sort_by(|a, b| {
            a.adjusted_score
                .total_cmp(&b.adjusted_score)
                .then_with(|| a.expected_remaining.total_cmp(&b.expected_remaining))
                .then_with(|| b.is_candidate.cmp(&a.is_candidate))
                .then_with(|| a.word.cmp(&b.word))
        });
        scored.truncate(top_n);
        scored
    }
}

fn reduce_guess_pool(pool: Vec<Word>, candidates: &[Word]) -> Vec<Word> {
    if candidates.len() <= 120 {
        return pool;
    }

    let mut ranked = rough_rank_guesses(&pool, candidates);
    let cap = if candidates.len() > 1000 {
        3_000
    } else {
        6_000
    };
    ranked.truncate(cap.min(ranked.len()));
    ranked
}

pub fn feedback(guess: Word, answer: Word) -> Pattern {
    let guess_letters = guess.letters();
    let answer_letters = answer.letters();

    // 0=B, 1=Y, 2=G
    let mut marks = [0u8; WORD_LEN];
    let mut available = [0u8; 26];

    for i in 0..WORD_LEN {
        if guess_letters[i] == answer_letters[i] {
            marks[i] = 2;
        } else {
            let index = (answer_letters[i] - b'A') as usize;
            available[index] += 1;
        }
    }

    for i in 0..WORD_LEN {
        if marks[i] != 0 {
            continue;
        }

        let letter_index = (guess_letters[i] - b'A') as usize;
        if available[letter_index] > 0 {
            marks[i] = 1;
            available[letter_index] -= 1;
        }
    }

    Pattern::from_marks(marks)
}

// greens stay put, every revealed letter must be reused at least as often
pub fn obeys_hard_mode(guess: Word, history: &[GuessFeedback]) -> bool {
    let guess_letters = guess.letters();
    let mut have = [0u8; 26];
    for &letter in &guess_letters {
        have[(letter - b'A') as usize] += 1;
    }

    history.iter().all(|entry| {
        let prior_letters = entry.guess.letters();
        let marks = entry.pattern.marks();
        let mut need = [0u8; 26];

        for i in 0..WORD_LEN {
            if marks[i] == 0 {
                continue;
            }
            if marks[i] == 2 && guess_letters[i] != prior_letters[i] {
                return false;
            }
            need[(prior_letters[i] - b'A') as usize] += 1;
        }

        need.iter().zip(&have).all(|(need, have)| have >= need)
    })
}

fn score_guess(guess: Word, candidates: &[Word], candidate_set: &HashSet<Word>) -> Suggestion {
    let mut buckets = [0u16; NUM_PATTERNS];
    for &answer in candidates {
        let pattern = feedback(guess, answer).index();
        buckets[pattern] += 1;
    }

    let total_answers = candidates.len() as f64;
    let mut expected_remaining = 0.0;
    let mut singleton_buckets = 0usize;

    for &count in &buckets {
        if count == 0 {
            continue;
        }

        let count = f64::from(count);
        expected_remaining += (count * count) / total_answers;
        if count == 1.0 {
            singleton_buckets += 1;
        }
    }

    // GGGGG is known but needs no next turn
    let solved_now = usize::from(buckets[SOLVED_PATTERN as usize]);
    let known_rate = singleton_buckets as f64 / total_answers;
    let solve_next_rate = (singleton_buckets - solved_now) as f64 / total_answers;
    let adjusted_score = (1.0 - known_rate) * expected_remaining;
    let guesses = estimate_guesses(&buckets, total_answers, expected_remaining);

    Suggestion {
        word: guess,
        guesses,
        adjusted_score,
        expected_remaining,
        solve_next_rate,
        is_candidate: candidate_set.contains(&guess),
    }
}

// expected turns including this one; solved bucket ends the game here
fn estimate_guesses(buckets: &[u16; NUM_PATTERNS], total_answers: f64, expected_remaining: f64) -> f64 {
    let reduction = (total_answers / expected_remaining.max(1.0)).max(1.000_001);

    let mut turns = 0.0;
    for (index, &count) in buckets.iter().enumerate() {
        if count == 0 {
            continue;
        }

        let count = f64::from(count);
        let weight = count / total_answers;
        if index == SOLVED_PATTERN as usize {
            turns += weight;
        } else {
            turns += weight * (1.0 + remaining_turns(count, reduction));
        }
    }
    turns
}

// bounded by "guess a candidate, rest splits" and "walk the candidates one by one"
fn remaining_turns(count: f64, reduction: f64) -> f64 {
    let estimate = 1.0 + count.ln() / reduction.ln();
    estimate.clamp(2.0 - 1.0 / count, (count + 1.0) / 2.0)
}

fn rough_rank_guesses(guesses: &[Word], candidates: &[Word]) -> Vec<Word> {
    let mut total_letter_freq = [0u16; 26];
    let mut positional_freq = [[0u16; 26]; WORD_LEN];

    for &candidate in candidates {
        let letters = candidate.letters();
        let mut seen = [false; 26];

        for i in 0..WORD_LEN {
            let idx = (letters[i] - b'A') as usize;
            positional_freq[i][idx] += 1;
            if !seen[idx] {
                total_letter_freq[idx] += 1;
                seen[idx] = true;
            }
        }
    }

    let mut ranked = guesses
        .iter()
        .copied()
        .map(|guess| {
            let letters = guess.letters();
            let mut seen = [false; 26];
            let mut score = 0u32;

            for i in 0..WORD_LEN {
                let idx = (letters[i] - b'A') as usize;
                if seen[idx] {
                    continue;
                }
                seen[idx] = true;
                score += u32::from(total_letter_freq[idx]);
                score += u32::from(positional_freq[i][idx]);
            }

            (guess, score)
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.into_iter().map(|(word, _)| word).collect()
}

#[cfg(test)]
mod tests {
    use super::{GuessFeedback, Pattern, Solver, feedback};
    use crate::word::Word;

    fn w(raw: &str) -> Word {
        Word::parse_upper_ascii(raw).expect("word literal must be valid")
    }

    #[test]
    fn feedback_handles_repeated_letters() {
        let pattern = feedback(w("ALLEY"), w("APPLE"));
        assert_eq!(pattern.to_string(), "GYBYB");
    }

    #[test]
    fn feedback_matches_readme_example() {
        let pattern = feedback(w("AROSE"), w("ABACK"));
        assert_eq!(pattern.to_string(), "GBBBB");
    }

    #[test]
    fn pattern_round_trip() {
        let pattern = Pattern::parse("GYBBB").expect("valid pattern");
        assert_eq!(pattern.to_string(), "GYBBB");
    }

    #[test]
    fn filtering_keeps_only_matching_answers() {
        let answers = vec![w("ABACK"), w("ALARM"), w("BEARD")];
        let guesses = vec![w("AROSE"), w("ABACK"), w("ALARM"), w("BEARD")];
        let solver = Solver::new(answers, guesses);

        let history = vec![GuessFeedback {
            guess: w("AROSE"),
            pattern: Pattern::parse("GBBBB").expect("valid pattern"),
        }];

        let candidates = solver.candidates(&history);
        assert_eq!(candidates, vec![w("ABACK")]);
    }

    #[test]
    fn guesses_estimate_rewards_better_partitioning() {
        let answers = vec![w("ABACK"), w("ALARM"), w("BEARD"), w("CRANE"), w("SHARD")];
        let guesses = answers.clone();
        let solver = Solver::new(answers.clone(), guesses);
        let suggestions = solver.suggestions(&[], &answers, false, 5);

        let best = suggestions.first().expect("at least one suggestion");
        let worst = suggestions.last().expect("at least one suggestion");
        assert!(best.guesses <= worst.guesses + f64::EPSILON);
    }
}
