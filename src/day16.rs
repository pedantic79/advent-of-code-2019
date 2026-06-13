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

#[aoc(day16, part1)]
pub fn part1(input: &Input) -> usize {
    let mut digits = input.digits.clone();
    let n = digits.len();
    let base_pattern = [0, 1, 0, -1];

    for _ in 0..100 {
        let mut next_digits = Vec::with_capacity(n);
        for i in 1..=n {
            let mut sum = 0;
            for (j, &digit) in digits.iter().enumerate() {
                let pattern_val = base_pattern[((j + 1) / i) % 4];
                sum += digit * pattern_val;
            }
            next_digits.push(sum.abs() % 10);
        }
        digits = next_digits;
    }

    let mut result = 0;
    for &digit in digits.iter().take(8) {
        result = result * 10 + digit as usize;
    }
    result
}

#[aoc(day16, part2)]
pub fn part2(input: &Input) -> usize {
    let mut offset = 0;
    for &digit in input.digits.iter().take(7) {
        offset = offset * 10 + digit as usize;
    }

    let total_len = input.digits.len() * 10000;
    assert!(
        offset >= total_len / 2,
        "Offset is not in the second half of the signal"
    );

    let mut suffix = Vec::with_capacity(total_len - offset);
    for i in offset..total_len {
        suffix.push(input.digits[i % input.digits.len()]);
    }

    for _ in 0..100 {
        let mut sum = 0;
        for val in suffix.iter_mut().rev() {
            sum = (sum + *val) % 10;
            *val = sum;
        }
    }

    let mut result = 0;
    for &digit in suffix.iter().take(8) {
        result = result * 10 + digit as usize;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn sample_1() {
        let input = generator("12345678");
        let mut digits = input.digits.clone();
        let n = digits.len();
        let base_pattern = [0, 1, 0, -1];
        for _ in 0..4 {
            let mut next_digits = Vec::with_capacity(n);
            for i in 1..=n {
                let mut sum = 0;
                for (j, &digit) in digits.iter().enumerate() {
                    let pattern_val = base_pattern[((j + 1) / i) % 4];
                    sum += digit * pattern_val;
                }
                next_digits.push(sum.abs() % 10);
            }
            digits = next_digits;
        }
        let mut result = 0;
        for &digit in digits.iter().take(8) {
            result = result * 10 + digit as usize;
        }
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
