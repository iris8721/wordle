# wordle

Terminal Wordle solver/assistant in Rust. Zero dependencies, about 1k LOC including tests.

Feed it the guesses you made and the color patterns Wordle returned; it narrows the
official answer list and ranks the next best guesses by expected information gain.
It can also auto-play a game against a known answer for benchmarking.

## build / run

Needs Rust 1.85 or newer (edition 2024).

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

- `--hard` — Wordle hard mode: suggestions and typed guesses must keep every green in
  place and reuse every yellow letter. Grays are not enforced, same as the real game.
- `--top N` — number of suggestions to show (default: 10)
- `--answer WORD` — run an automatic simulation against a known answer
- `--start WORD` — starting guess for simulation mode
- `--max-guesses N` — guess limit in simulation mode (default: 6)

## suggestion metrics

- `guesses`: heuristic estimate of turns needed to finish, counting the guess itself (lower is better).
  A guess that is not a possible answer can never score below 2, since you still have to play the answer.
- `adjusted`: weighted partition score from bucket sizes, discounted by how often the answer is known after this guess (lower is better).
- `expected`: expected number of candidates left after the guess (lower is better).
- `solve-next`: chance this guess pins the answer down so it can be played on the next turn (higher is better).
  A green-everything result counts as solved now, not solve-next.

## design notes

- Feedback patterns are encoded as base-3 numbers (0–242, one per B/Y/G tile), so
  `feedback(guess, answer)` is a small integer used to bucket candidates in a
  fixed 243-entry array — no string keys, no hashing.
- Scoring partitions the remaining candidates by the pattern each guess would
  produce, then derives expected remaining count, singleton-bucket solve rate,
  and an adjusted score from that partition.
- When the candidate set is large, the guess pool is pre-ranked by letter
  frequency and capped (3k–6k words) to keep scoring interactive.
- Word lists are embedded via `include_str!`: the 2315-word NYT answer list and a
  14.8k-word allowed-guess list (larger than the original NYT guess list).

## known limitations

- The `guesses` metric sums, per feedback bucket, one turn plus a log-based
  estimate for clearing that bucket. It is not an exact minimax/optimal-tree
  computation.
- Simulation mode plays greedily by adjusted score; it does not guarantee
  optimal play.
