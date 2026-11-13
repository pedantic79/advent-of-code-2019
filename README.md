# Advent of Code 2019 - Rust

Requires Rust 1.94 or newer. Run `cargo test --locked --all-features` to test the
solutions and shared utilities.

Install the runner with `cargo install aoc-harness-cli --version 0.6.0 --locked`.
Run these commands from this directory.

```sh
cargo aoc-harness run --all-days
cargo aoc-harness run --day 1 --part 2
cargo aoc-harness run --all-days --all-solutions --format json
cargo aoc-harness bench --day 1 --family generator,solver --smoke
cargo aoc-harness bench --all-days --family full-run --sample-size 10
cargo aoc-harness bench --day 1 --save-baseline before
cargo aoc-harness bench --day 1 --baseline before
cargo aoc-harness profile --day 1 --part 2
```

Without a day selection, the harness runs the latest registered day.
`--all-solutions` checks that variants of each part agree. A benchmark smoke
run executes each selected workload once. To compare solver timings, save a
baseline before an edit and compare against it afterward. Use `--family full-run`
to time all selected days together or `--family end-to-end` to time parsing and
solving for each solution separately.

Inputs are read from `input/2019/dayN.txt`. Use `--input PATH` with a single day
to override the input. To download missing inputs, configure the session token
in `~/.adventofcode.session` and run `cargo aoc-harness input --all`. The CLI
reads the event year from `aoc_lib!` in `src/lib.rs`.

Regression tests use `aoc_harness::input!(day)` to read the local input at test
time. Like the runner, it accepts zero-padded filenames as a fallback and strips
trailing newlines. Run `just watch-test 08` to rerun a day's library tests on
file changes, or pass a test-name filter such as `just watch-test regression`.

`just add-day N` uses the harness to generate a zero-padded source file such as
`src/day02.rs` from [the package template](templates/day.rs), makes the module
public, and adds its README links.
Omit `N` to add the day after the latest source file. To preview the underlying
harness changes, run `cargo aoc-harness generate --day N --dry-run`. All 25
days in this workspace are already registered.

- [Day 1: The Tyranny of the Rocket Equation](https://adventofcode.com/2019/day/1)
  - [solution](src/day01.rs)
- [Day 2: 1202 Program Alarm](https://adventofcode.com/2019/day/2)
  - [solution](src/day02.rs)
- [Day 3: Crossed Wires](https://adventofcode.com/2019/day/3)
  - [solution](src/day03.rs)
- [Day 4: Secure Container](https://adventofcode.com/2019/day/4)
  - [solution](src/day04.rs)
- [Day 5: Sunny with a Chance of Asteroids](https://adventofcode.com/2019/day/5)
  - [solution](src/day05.rs)
- [Day 6: Universal Orbit Map](https://adventofcode.com/2019/day/6)
  - [solution](src/day06.rs)
- [Day 7: Amplification Circuit](https://adventofcode.com/2019/day/7)
  - [solution](src/day07.rs)
- [Day 8: Space Image Format](https://adventofcode.com/2019/day/8)
  - [solution](src/day08.rs)
- [Day 9: Sensor Boost](https://adventofcode.com/2019/day/9)
  - [solution](src/day09.rs)
- [Day 10: Monitoring Station](https://adventofcode.com/2019/day/10)
  - [solution](src/day10.rs)
- [Day 11: Space Painting](https://adventofcode.com/2019/day/11)
  - [solution](src/day11.rs)
- [Day 12: The N-Body Problem](https://adventofcode.com/2019/day/12)
  - [solution](src/day12.rs)
- [Day 13: Care Package](https://adventofcode.com/2019/day/13)
  - [solution](src/day13.rs)
- [Day 14: Space Stoichiometry](https://adventofcode.com/2019/day/14)
  - [solution](src/day14.rs)
- [Day 15: Oxygen System](https://adventofcode.com/2019/day/15)
  - [solution](src/day15.rs)
- [Day 16: Flawed Frequency Transmission](https://adventofcode.com/2019/day/16)
  - [solution](src/day16.rs)
- [Day 17: Set and Forget](https://adventofcode.com/2019/day/17)
  - [solution](src/day17.rs)
- [Day 18: Many-Worlds Interpretation](https://adventofcode.com/2019/day/18)
  - [solution](src/day18.rs)
- [Day 19: Tractor Beam](https://adventofcode.com/2019/day/19)
  - [solution](src/day19.rs)
- [Day 20: Donut Maze](https://adventofcode.com/2019/day/20)
  - [solution](src/day20.rs)
- [Day 21: Springdroid Adventure](https://adventofcode.com/2019/day/21)
  - [solution](src/day21.rs)
- [Day 22: Slam Shuffle](https://adventofcode.com/2019/day/22)
  - [solution](src/day22.rs)
- [Day 23: Category Six](https://adventofcode.com/2019/day/23)
  - [solution](src/day23.rs)
- [Day 24: Planet of Discord](https://adventofcode.com/2019/day/24)
  - [solution](src/day24.rs)
- [Day 25: Cryostasis](https://adventofcode.com/2019/day/25)
  - [solution](src/day25.rs)
<!-- Insert before -->

See:

- [Advent of Code 2019](https://adventofcode.com/2019/)
- [aoc-harness](https://github.com/pedantic79/aoc-harness)
