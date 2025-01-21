use aoc_runner_derive::{aoc, aoc_generator};

use crate::intcode::{IntCode, MachineState};

#[aoc_generator(day5)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

#[aoc(day5, part1)]
pub fn part1(ic: &IntCode<i64>) -> i64 {
    let mut ic = ic.clone();
    ic.set_input(1);

    while ic.run() != MachineState::Halted {}
    let mut last = 0;
    while let Some(x) = ic.get_output() {
        last = x;
    }
    last
}

#[aoc(day5, part2)]
pub fn part2(ic: &IntCode<i64>) -> i64 {
    let mut ic = ic.clone();
    ic.set_input(5);

    ic.run();
    let mut last = 0;
    while let Some(x) = ic.get_output() {
        last = x;
    }
    last
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"1002,4,3,4,33";

    #[test]
    pub fn input_test() {
        println!("{:?}", generator(SAMPLE));

        // assert_eq!(generator(SAMPLE), Input());
    }

    #[test]
    pub fn part1_test() {
        assert_eq!(part1(&generator("1101,100,-1,4,0")), 0);
    }

    #[test]
    pub fn part2_test() {
        assert_eq!(
            part2(&generator(
                "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99"
            )),
            999
        );
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day5.txt");
        const ANSWERS: (i64, i64) = (8332629, 8805067);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
