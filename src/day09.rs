use aoc_harness::{aoc, aoc_generator};

use crate::intcode::IntCode;

#[aoc_generator(day9)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

#[aoc(day9, part1)]
pub fn part1(ic: &IntCode<i64>) -> i64 {
    let mut ic = ic.clone();
    ic.run_single_io(1).1.unwrap()
}

#[aoc(day9, part2)]
pub fn part2(ic: &IntCode<i64>) -> i64 {
    let mut ic = ic.clone();
    ic.run_single_io(2).1.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn part1_test() {
        assert_eq!(
            part1(&generator("1102,34915192,34915192,7,4,7,99,0")),
            1219070632396864
        );

        assert_eq!(
            part1(&generator("104,1125899906842624,99")),
            1125899906842624
        );
    }

    #[test]
    pub fn part2_test() {
        // assert_eq!(part2(&generator(SAMPLE)), 336);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day9.txt");
        const ANSWERS: (i64, i64) = (2941952859, 66113);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
