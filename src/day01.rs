use std::iter::successors;

use aoc_runner_derive::{aoc, aoc_generator};

#[aoc_generator(day1)]
pub fn generator(input: &str) -> Vec<u64> {
    input.lines().map(|s| s.parse().unwrap()).collect()
}

#[aoc(day1, part1)]
pub fn part1(input: &[u64]) -> u64 {
    input.iter().map(|n| n / 3 - 2).sum()
}

#[aoc(day1, part2)]
pub fn part2(input: &[u64]) -> u64 {
    input
        .iter()
        .map(|n| {
            successors(Some(*n), |x| (x / 3).checked_sub(2))
                .skip(1)
                .sum::<u64>()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"12
14
1969
100756";

    #[test]
    pub fn input_test() {
        println!("{:?}", generator(SAMPLE));

        // assert_eq!(generator(SAMPLE), Input());
    }

    #[test]
    pub fn part1_test() {
        assert_eq!(part1(&generator(SAMPLE)), 34241);
    }

    #[test]
    pub fn part2_test() {
        assert_eq!(part2(&generator(SAMPLE)), 51316);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day1.txt");
        const ANSWERS: (u64, u64) = (3465154, 5194864);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
