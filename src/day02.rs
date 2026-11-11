use aoc_harness::{aoc, aoc_generator};

use crate::intcode::IntCode;

const TARGET: i64 = 19690720;

#[aoc_generator(day2)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

fn output_with_inputs(
    program: &IntCode<i64>,
    machine: &mut IntCode<i64>,
    noun: i64,
    verb: i64,
) -> i64 {
    machine.clone_from(program);
    machine.set_values(&[(1, noun), (2, verb)]);
    machine.run();
    machine.get_value(0)
}

fn find_monotonic_inputs(ic: &IntCode<i64>) -> Option<i64> {
    let mut machine = ic.clone();
    let candidates: [i64; 100] = std::array::from_fn(|index| i64::try_from(index).unwrap());
    let noun_index = candidates
        .partition_point(|&noun| output_with_inputs(ic, &mut machine, noun, 0) <= TARGET)
        .checked_sub(1)?;
    let noun = candidates[noun_index];
    let verb_index = candidates
        .binary_search_by_key(&TARGET, |&verb| {
            output_with_inputs(ic, &mut machine, noun, verb)
        })
        .ok()?;

    Some(noun * 100 + candidates[verb_index])
}

fn find_exhaustive_inputs(ic: &IntCode<i64>) -> Option<i64> {
    let mut machine = ic.clone();
    (0..100).find_map(|noun| {
        (0..100).find_map(|verb| {
            (output_with_inputs(ic, &mut machine, noun, verb) == TARGET)
                .then_some(noun * 100 + verb)
        })
    })
}

#[aoc(day2, part1)]
pub fn part1(ic: &IntCode<i64>) -> i64 {
    let mut machine = ic.clone();
    output_with_inputs(ic, &mut machine, 12, 2)
}

#[aoc(day2, part2)]
pub fn part2(ic: &IntCode<i64>) -> i64 {
    // The puzzle input is monotonic in noun and verb, so binary search is a
    // fast path. Fall back to exhaustive search for other valid programs.
    find_monotonic_inputs(ic)
        .or_else(|| find_exhaustive_inputs(ic))
        .expect("no noun and verb found")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"1,9,10,3,2,3,11,0,99,30,40,50";

    #[test]
    pub fn input_test() {
        println!("{:?}", generator(SAMPLE));

        // assert_eq!(generator(SAMPLE), Input());
    }

    #[test]
    pub fn part1_test() {
        for (program, read_addr, expected) in [
            ("1,0,0,0,99", 0, 2),
            ("2,3,0,3,99", 3, 6),
            ("2,4,4,5,99,0", 5, 9801),
            ("1,1,1,4,99,5,6,0,99", 0, 30),
        ] {
            assert_eq!(
                {
                    let mut x: IntCode<i64> = program.parse::<IntCode<i64>>().unwrap();
                    x.run();
                    x.get_value(read_addr)
                },
                expected
            );
        }
    }

    #[test]
    pub fn part2_test() {
        let mut memory = vec![0; 100];
        memory[0] = 1;
        memory[4] = 99;
        memory[5] = 10_000_000;
        memory[6] = 9_690_720;
        let program = memory
            .into_iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",");

        assert_eq!(part2(&generator(&program)), 506);
    }

    mod regression {
        use super::*;

        const ANSWERS: (i64, i64) = (10566835, 2347);

        #[test]
        pub fn test() {
            let output = generator(&aoc_harness::input!(2));

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
