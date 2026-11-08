use aoc_harness::{aoc, aoc_generator};

use crate::intcode::{IntCode, MachineState};

// Jump over any hole in A..=C, provided D is a safe landing: !(A && B && C) && D.
const WALK_SCRIPT: &str = "OR A J\nAND B J\nAND C J\nNOT J J\nAND D J\nWALK\n";

// After landing at D, allow a step onto E or another jump to H.
const RUN_SCRIPT: &str =
    "OR A J\nAND B J\nAND C J\nNOT J J\nAND D J\nOR E T\nOR H T\nAND T J\nRUN\n";

#[aoc_generator(day21)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.trim().parse().unwrap()
}

fn run_script(program: &IntCode<i64>, script: &str) -> i64 {
    let mut droid = program.clone();
    for byte in script.bytes() {
        droid.set_input(i64::from(byte));
    }

    loop {
        match droid.run() {
            MachineState::HasOutput => {}
            MachineState::Halted => break,
            MachineState::WaitingForInput => panic!("springdroid requested more input"),
        }
    }

    let mut transcript = Vec::new();
    while let Some(value) = droid.get_output() {
        if value > i64::from(u8::MAX) {
            return value;
        }
        transcript.push(u8::try_from(value).expect("invalid springdroid output"));
    }

    panic!(
        "springdroid did not report hull damage: {}",
        String::from_utf8_lossy(&transcript)
    );
}

#[aoc(day21, part1)]
pub fn part1(program: &IntCode<i64>) -> i64 {
    run_script(program, WALK_SCRIPT)
}

#[aoc(day21, part2)]
pub fn part2(program: &IntCode<i64>) -> i64 {
    run_script(program, RUN_SCRIPT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jumps(script: &str, ground: &[bool], mode: &str) -> bool {
        assert!(script.ends_with('\n'));
        let mut lines: Vec<_> = script.lines().collect();
        assert_eq!(lines.pop(), Some(mode));
        assert!(lines.len() <= 15);

        let mut registers = [false; 26];
        registers[..ground.len()].copy_from_slice(ground);
        for line in lines {
            let fields: Vec<_> = line.split_ascii_whitespace().collect();
            let [operation, source, destination] = fields.as_slice() else {
                panic!("invalid instruction: {line}");
            };
            assert_eq!(source.len(), 1);
            assert_eq!(destination.len(), 1);
            let source = source.as_bytes()[0];
            assert!(usize::from(source - b'A') < ground.len() || matches!(source, b'T' | b'J'));
            let destination = destination.as_bytes()[0];
            assert!(matches!(destination, b'T' | b'J'));
            let source = registers[usize::from(source - b'A')];
            let destination = &mut registers[usize::from(destination - b'A')];
            *destination = match *operation {
                "NOT" => !source,
                "AND" => *destination && source,
                "OR" => *destination || source,
                _ => panic!("invalid operation: {operation}"),
            };
        }

        registers[usize::from(b'J' - b'A')]
    }

    #[test]
    fn walk_jumps_over_holes_only_with_a_safe_landing() {
        for mask in 0..16 {
            let ground: [bool; 4] = std::array::from_fn(|index| mask & (1 << index) != 0);
            assert_eq!(
                jumps(WALK_SCRIPT, &ground, "WALK"),
                ground[3] && ground[..3].contains(&false),
                "ground: {ground:?}",
            );
        }
    }

    #[test]
    fn run_requires_a_step_or_another_jump_after_landing() {
        for mask in 0..512 {
            let ground: [bool; 9] = std::array::from_fn(|index| mask & (1 << index) != 0);
            assert_eq!(
                jumps(RUN_SCRIPT, &ground, "RUN"),
                ground[3] && ground[..3].contains(&false) && (ground[4] || ground[7]),
                "ground: {ground:?}",
            );
        }
    }

    #[test]
    fn sends_ascii_including_the_final_newline_and_reads_damage() {
        let script = "NOT A J\nWALK\n";
        // Emit a prompt, then report damage only if every input byte matches.
        let mut code = vec![104, 62, 104, 10];
        for byte in script.bytes() {
            code.extend([3, 1000, 1008, 1000, i64::from(byte), 1001, 1005, 1001]);
            code.push(i64::try_from(code.len() + 2).unwrap());
            code.push(99);
        }
        code.extend([104, 12_345, 99]);
        let program = generator(
            &code
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );

        assert_eq!(run_script(&program, script), 12_345);
    }

    #[test]
    #[should_panic(expected = "springdroid did not report hull damage: Failed")]
    fn includes_ascii_diagnostics_when_the_droid_falls() {
        run_script(
            &generator("104,70,104,97,104,105,104,108,104,101,104,100,99"),
            "WALK\n",
        );
    }

    #[test]
    #[should_panic(expected = "springdroid requested more input")]
    fn rejects_incomplete_input() {
        run_script(&generator("3,100,1105,1,0"), "WALK\n");
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day21.txt");
        const ANSWERS: (i64, i64) = (19_350_938, 1_142_986_901);

        #[test]
        fn answers() {
            let program = generator(INPUT);

            assert_eq!(part1(&program), ANSWERS.0);
            assert_eq!(part2(&program), ANSWERS.1);
        }
    }
}
