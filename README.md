## commands

- `GUESS PATTERN` (example: `CRANE BYBBB`)
- `LIST`
- `UNDO`
- `RESET`
- `HARD ON` / `HARD OFF`
- `QUIT`

pattern symbols:
- `G` green
- `Y` yellow
- `B` black

suggestion metrics:
- `guesses`: heuristic estimate of guesses needed to finish if you choose that word now (lower is better).
- `adjusted`: weighted partition score from bucket sizes with a next-turn solve adjustment (lower is better).
- `expected`: expected number of candidates left after the guess (lower is better).
- `solve-next`: chance the puzzle is effectively solved on the next turn after this guess (higher is better).