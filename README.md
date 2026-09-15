# wordle

Terminal Wordle solver/assistant in Rust. Zero dependencies, ~840 LOC.

Feed it the guesses you made and the color patterns Wordle returned; it narrows the
official answer list and ranks the next best guesses by expected information gain.
It can also auto-play a game against a known answer for benchmarking.

## build / run

```sh
cargo run --release
```

Interactive mode commands:

- `GUESS PATTERN` (example: `CRANE BYBBB`)
- `LIST`
- `UNDO`
- `RESET`
- `HARD ON` / `HARD OFF`
- `QUIT`

Pattern symbols:

- `G` green
- `Y` yellow
- `B` black

## flags

```
cargo run -- [--hard] [--top N]
cargo run -- --answer WORD [--start WORD] [--hard] [--max-guesses N]
```

- `--hard` — restrict suggestions to remaining valid answers (Wordle hard mode)
- `--top N` — number of suggestions to show (default: 10)
- `--answer WORD` — run an automatic simulation against a known answer
- `--start WORD` — starting guess for simulation mode
- `--max-guesses N` — guess limit in simulation mode (default: 6)

## suggestion metrics

- `guesses`: heuristic estimate of guesses needed to finish if you choose that word now (lower is better).
- `adjusted`: weighted partition score from bucket sizes with a next-turn solve adjustment (lower is better).
- `expected`: expected number of candidates left after the guess (lower is better).
- `solve-next`: chance the puzzle is effectively solved on the next turn after this guess (higher is better).

## design notes

- Feedback patterns are encoded as base-3 numbers (0–242, one per B/Y/G tile), so
  `feedback(guess, answer)` is a small integer used to bucket candidates in a
  fixed 243-entry array — no string keys, no hashing.
- Scoring partitions the remaining candidates by the pattern each guess would
  produce, then derives expected remaining count, singleton-bucket solve rate,
  and an adjusted score from that partition.
- When the candidate set is large, the guess pool is pre-ranked by letter
  frequency and capped (3k–6k words) to keep scoring interactive.
- Word lists are the official NYT answer/guess lists, embedded via `include_str!`.

## known limitations

- The `guesses` metric is a heuristic (log-based estimate), not an exact
  minimax/optimal-tree computation.
- Simulation mode plays greedily by adjusted score; it does not guarantee
  optimal play.
