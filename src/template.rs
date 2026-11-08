use aoc_harness::{aoc, aoc_generator};

#[derive(Debug, PartialEq, Eq)]
pub struct Input {}

#[aoc_generator(dayN)]
pub fn generator(input: &str) -> Input {
    unimplemented!()
}

#[aoc(dayN, part1)]
pub fn part1(input: &Input) -> usize {
    unimplemented!()
}

#[aoc(dayN, part2)]
pub fn part2(input: &Input) -> usize {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"..##.......";

    #[test]
    pub fn input_test() {
        println!("{:?}", generator(SAMPLE));

        // assert_eq!(generator(SAMPLE), Input());
    }

    #[test]
    pub fn part1_test() {
        // assert_eq!(part1(&generator(SAMPLE)), 7);
    }

    #[test]
    pub fn part2_test() {
        // assert_eq!(part2(&generator(SAMPLE)), 336);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/dayN.txt");
        const ANSWERS: (usize, usize) = (0, 0);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            // let output = generator(input);

            // assert_eq!(part1(&output), ANSWERS.0);
            // assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
