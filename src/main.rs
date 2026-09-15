mod solver;
mod word;
mod word_lists;

use solver::{GuessFeedback, Pattern, Solver, feedback, obeys_hard_mode};
use std::io::{self, Write};
use word::Word;
use word_lists::load_word_lists;

struct Config {
    hard_mode: bool,
    top_n: usize,
    max_guesses: usize,
    simulate_answer: Option<Word>,
    simulate_start: Option<Word>,
}

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    let lists = load_word_lists();
    let solver = Solver::new(lists.answers, lists.guesses);

    if let Some(answer) = config.simulate_answer {
        if let Err(err) = run_simulation(&solver, answer, config.simulate_start, &config) {
            eprintln!("{err}");
            std::process::exit(1);
        }
    } else if let Err(err) = run_interactive(&solver, &config) {
        eprintln!("I/O error: {err}");
        std::process::exit(1);
    }
}

fn parse_args() -> Result<Config, String> {
    let mut hard_mode = false;
    let mut top_n = 10usize;
    let mut max_guesses = 6usize;
    let mut simulate_answer = None;
    let mut simulate_start = None;

    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--hard" => hard_mode = true,
            "--top" => {
                let Some(value) = args.next() else {
                    return Err("--top expects a number".to_string());
                };
                top_n = value
                    .parse::<usize>()
                    .map_err(|_| "--top expects a positive integer".to_string())?;
                if top_n == 0 {
                    return Err("--top must be >= 1".to_string());
                }
            }
            "--max-guesses" => {
                let Some(value) = args.next() else {
                    return Err("--max-guesses expects a number".to_string());
                };
                max_guesses = value
                    .parse::<usize>()
                    .map_err(|_| "--max-guesses expects a positive integer".to_string())?;
                if max_guesses == 0 {
                    return Err("--max-guesses must be >= 1".to_string());
                }
            }
            "--answer" => {
                let Some(value) = args.next() else {
                    return Err("--answer expects a 5-letter word".to_string());
                };
                let Some(word) = Word::parse_user_input(&value) else {
                    return Err("--answer expects a valid 5-letter word".to_string());
                };
                simulate_answer = Some(word);
            }
            "--start" => {
                let Some(value) = args.next() else {
                    return Err("--start expects a 5-letter word".to_string());
                };
                let Some(word) = Word::parse_user_input(&value) else {
                    return Err("--start expects a valid 5-letter word".to_string());
                };
                simulate_start = Some(word);
            }
            "--help" | "-h" => {
                print!("{}", help_text());
                std::process::exit(0);
            }
            other => {
                return Err(format!("Unknown argument: {other}\n\n{}", help_text()));
            }
        }
    }

    Ok(Config {
        hard_mode,
        top_n,
        max_guesses,
        simulate_answer,
        simulate_start,
    })
}

fn help_text() -> String {
    let text = r#"WordleBot Rust (terminal)

Usage:
  cargo run -- [--hard] [--top N]
  cargo run -- --answer WORD [--start WORD] [--hard] [--max-guesses N]

Flags:
  --hard            Only suggest/accept guesses that reuse revealed hints (Wordle hard mode)
  --top N           Number of suggestions to show (default: 10)
  --answer WORD     Run automatic simulation against a known answer
  --start WORD      Starting guess for simulation mode
  --max-guesses N   Guess limit in simulation mode (default: 6)
"#;
    text.to_string()
}

fn run_interactive(solver: &Solver, config: &Config) -> io::Result<()> {
    let mut history: Vec<GuessFeedback> = Vec::new();
    let mut hard_mode = config.hard_mode;

    println!("WordleBot Rust");
    println!(
        "Commands: GUESS PATTERN (e.g. CRANE BYBBB), LIST, UNDO, RESET, HARD ON, HARD OFF, QUIT"
    );
    println!("Pattern format: G = green, Y = yellow, B = black/gray");

    loop {
        let candidates = solver.candidates(&history);
        if candidates.is_empty() {
            println!("No candidates are consistent with the clues. Use UNDO or RESET.");
        } else {
            println!();
            println!(
                "Turn {} | candidates: {} | mode: {}",
                history.len() + 1,
                candidates.len(),
                if hard_mode { "hard" } else { "normal" }
            );

            if candidates.len() <= 20 {
                let list = candidates
                    .iter()
                    .map(|w| w.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("Candidates: {list}");
            }

            let suggestions = solver.suggestions(&history, &candidates, hard_mode, config.top_n);
            println!("Top suggestions:");
            for (index, suggestion) in suggestions.iter().enumerate() {
                println!(
                    "{:>2}. {}  guesses={:.3} adjusted={:.4} expected={:.3} solve-next={:>5.1}% {}",
                    index + 1,
                    suggestion.word,
                    suggestion.guesses,
                    suggestion.adjusted_score,
                    suggestion.expected_remaining,
                    suggestion.solve_next_rate * 100.0,
                    if suggestion.is_candidate {
                        "(candidate)"
                    } else {
                        ""
                    }
                );
            }
        }

        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let upper = trimmed
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_uppercase();
        match upper.as_str() {
            "QUIT" | "EXIT" => break,
            "UNDO" => {
                if history.pop().is_some() {
                    println!("Removed last clue.");
                } else {
                    println!("Nothing to undo.");
                }
                continue;
            }
            "RESET" => {
                history.clear();
                println!("History cleared.");
                continue;
            }
            "LIST" => {
                let candidates = solver.candidates(&history);
                if candidates.is_empty() {
                    println!("No candidates.");
                } else {
                    let list = candidates
                        .iter()
                        .map(|w| w.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("{list}");
                }
                continue;
            }
            "HARD ON" => {
                hard_mode = true;
                println!("Hard mode enabled.");
                continue;
            }
            "HARD OFF" => {
                hard_mode = false;
                println!("Hard mode disabled.");
                continue;
            }
            _ => {}
        }

        let mut parts = upper.split_whitespace();
        let Some(guess_raw) = parts.next() else {
            continue;
        };
        let Some(pattern_raw) = parts.next() else {
            println!("Expected input: GUESS PATTERN");
            continue;
        };
        if parts.next().is_some() {
            println!("Expected exactly two fields: GUESS PATTERN");
            continue;
        }

        let Some(guess) = Word::parse_user_input(guess_raw) else {
            println!("Invalid guess '{guess_raw}'. Use a 5-letter word.");
            continue;
        };
        if !solver.is_valid_guess(guess) {
            println!("'{guess}' is not in the guess list.");
            continue;
        }

        let pattern = match Pattern::parse(pattern_raw) {
            Ok(pattern) => pattern,
            Err(err) => {
                println!("{err}");
                continue;
            }
        };

        if hard_mode && !obeys_hard_mode(guess, &history) {
            println!("'{guess}' does not use every revealed hint (hard mode).");
            continue;
        }

        history.push(GuessFeedback { guess, pattern });
        if pattern.is_solved() {
            println!("Solved in {} guesses.", history.len());
        }
    }

    Ok(())
}

fn run_simulation(
    solver: &Solver,
    answer: Word,
    start: Option<Word>,
    config: &Config,
) -> Result<(), String> {
    if !solver.is_possible_answer(answer) {
        return Err(format!(
            "Simulation answer '{}' is not in the official answer list.",
            answer
        ));
    }

    let mut history: Vec<GuessFeedback> = Vec::new();
    let mut guess = if let Some(start) = start {
        if !solver.is_valid_guess(start) {
            return Err(format!("Starting word '{}' is not a valid guess.", start));
        }
        start
    } else {
        let candidates = solver.candidates(&history);
        let suggestions = solver.suggestions(&history, &candidates, config.hard_mode, 1);
        if let Some(first) = suggestions.first() {
            first.word
        } else {
            return Err("No suggestion could be generated for the starting turn.".to_string());
        }
    };

    println!(
        "Simulating answer {} | mode: {}",
        answer,
        if config.hard_mode { "hard" } else { "normal" }
    );

    for turn in 1..=config.max_guesses {
        let pattern = feedback(guess, answer);
        println!("{turn}. {guess} -> {pattern}");

        history.push(GuessFeedback { guess, pattern });
        if pattern.is_solved() {
            println!("Solved in {turn} guesses.");
            return Ok(());
        }

        let candidates = solver.candidates(&history);
        if candidates.is_empty() {
            println!("No candidates left after turn {turn}.");
            return Ok(());
        }

        let suggestions = solver.suggestions(&history, &candidates, config.hard_mode, config.top_n);
        let Some(next) = suggestions.first() else {
            println!("No next suggestion found after turn {turn}.");
            return Ok(());
        };
        guess = next.word;
    }

    println!(
        "Did not solve within {} guesses (limit reached).",
        config.max_guesses
    );
    Ok(())
}
