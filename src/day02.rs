use aoc_runner_derive::{aoc, aoc_generator};

use crate::intcode::IntCode;

#[aoc_generator(day2)]
pub fn generator(input: &str) -> IntCode<usize> {
    input.parse().unwrap()
}

#[aoc(day2, part1)]
pub fn part1(ic: &IntCode<usize>) -> usize {
    let mut ic = ic.clone();
    ic.set_values(&[(1, 12), (2, 2)]);
    ic.run();

    ic.get_value(0)
}

#[aoc(day2, part2)]
pub fn part2(ic: &IntCode<usize>) -> usize {
    const TARGET: usize = 19690720;

    let Some(noun) = (0..100).rev().find(|noun| {
        let mut ic = ic.clone();
        ic.set_values(&[(1, *noun)]);
        ic.run();
        ic.get_value(0) <= TARGET
    }) else {
        panic!("no noun found")
    };

    let Some(verb) = (0..100).rev().find(|verb| {
        let mut ic = ic.clone();
        ic.set_values(&[(1, noun), (2, *verb)]);
        ic.run();
        ic.get_value(0) == TARGET
    }) else {
        panic!("no verb found")
    };

    noun * 100 + verb
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
                    let mut x: IntCode<usize> = program.parse::<IntCode<usize>>().unwrap();
                    x.run();
                    x.get_value(read_addr)
                },
                expected
            );
        }
    }

    #[test]
    pub fn part2_test() {
        // assert_eq!(part2(&generator(SAMPLE)), 336);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day2.txt");
        const ANSWERS: (usize, usize) = (10566835, 2347);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
