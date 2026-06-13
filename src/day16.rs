use aoc_runner_derive::{aoc, aoc_generator};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Input {
    pub digits: Vec<i32>,
}

#[aoc_generator(day16)]
pub fn generator(input: &str) -> Input {
    let digits: Vec<i32> = input
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as i32)
        .collect();
    Input { digits }
}

fn parse_number(digits: &[i32], count: usize) -> usize {
    digits
        .iter()
        .take(count)
        .fold(0, |acc, &digit| acc * 10 + digit as usize)
}

fn fft_step(digits: &[i32]) -> Vec<i32> {
    let n = digits.len();
    let base_pattern = [0, 1, 0, -1];
    (1..=n)
        .map(|i| {
            let mut sum = 0;
            for (j, &digit) in digits.iter().enumerate() {
                let pattern_val = base_pattern[((j + 1) / i) % 4];
                sum += digit * pattern_val;
            }
            sum.abs() % 10
        })
        .collect()
}

#[aoc(day16, part1)]
pub fn part1(input: &Input) -> usize {
    let mut digits = input.digits.clone();
    for _ in 0..100 {
        digits = fft_step(&digits);
    }
    parse_number(&digits, 8)
}

#[aoc(day16, part2)]
pub fn part2(input: &Input) -> usize {
    let offset = parse_number(&input.digits, 7);

    let total_len = input.digits.len() * 10000;
    assert!(
        offset >= total_len / 2,
        "Offset is not in the second half of the signal"
    );

    let mut suffix: Vec<i32> = (offset..total_len)
        .map(|i| input.digits[i % input.digits.len()])
        .collect();

    for _ in 0..100 {
        let mut sum = 0;
        for val in suffix.iter_mut().rev() {
            sum = (sum + *val) % 10;
            *val = sum;
        }
    }

    parse_number(&suffix, 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn sample_1() {
        let input = generator("12345678");
        let mut digits = input.digits.clone();
        for _ in 0..4 {
            digits = fft_step(&digits);
        }
        let result = parse_number(&digits, 8);
        assert_eq!(result, 1029498);
    }

    #[test]
    pub fn sample_2() {
        assert_eq!(
            part1(&generator("80871224585914546619083218645595")),
            24176176
        );
    }

    #[test]
    pub fn sample_3() {
        assert_eq!(
            part1(&generator("19617804207202209144916044189917")),
            73745418
        );
    }

    #[test]
    pub fn sample_4() {
        assert_eq!(
            part1(&generator("69317163492948606335995924319873")),
            52432133
        );
    }

    #[test]
    pub fn part2_sample_1() {
        assert_eq!(
            part2(&generator("03036732577212944063491565474664")),
            84462026
        );
    }

    #[test]
    pub fn part2_sample_2() {
        assert_eq!(
            part2(&generator("02935109699940807407585447034323")),
            78725270
        );
    }

    #[test]
    pub fn part2_sample_3() {
        assert_eq!(
            part2(&generator("03081770884921959731165446850517")),
            53553731
        );
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day16.txt");
        const ANSWERS: (usize, usize) = (53296082, 43310035);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
