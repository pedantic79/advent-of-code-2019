use aoc_runner_derive::{aoc, aoc_generator};
use num::cast::AsPrimitive;

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

fn parse_number<T>(digits: &[T], count: usize) -> usize
where
    T: Copy + AsPrimitive<usize>,
{
    digits
        .iter()
        .take(count)
        .fold(0, |acc, &digit| acc * 10 + digit.as_())
}

fn fft_step(digits: &[i32]) -> Vec<i32> {
    let n = digits.len();
    // Precompute prefix sums for O(1) range sum queries.
    let mut prefix_sum = vec![0; n + 1];
    for i in 0..n {
        prefix_sum[i + 1] = prefix_sum[i] + digits[i];
    }

    (1..=n)
        .map(|i| {
            // Accumulate the weighted sum for the i-th output digit.
            let mut sum = 0;
            let mut k = 0;
            loop {
                // Add prefix range corresponding to the +1 multiplier.
                let start_plus = (4 * k + 1) * i - 1;
                if start_plus >= n {
                    break;
                }
                let end_plus = std::cmp::min((4 * k + 2) * i - 1, n);
                sum += prefix_sum[end_plus] - prefix_sum[start_plus];

                // Subtract prefix range corresponding to the -1 multiplier.
                let start_minus = (4 * k + 3) * i - 1;
                if start_minus >= n {
                    break;
                }
                let end_minus = std::cmp::min((4 * k + 4) * i - 1, n);
                sum -= prefix_sum[end_minus] - prefix_sum[start_minus];

                // Move to the next block of the pattern.
                k += 1;
            }
            // Keep only the ones digit of the absolute value.
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

/// Maps a contiguous index `m` to a value `d` where the binomial coefficient
/// `comb(d + 99, 99) mod 2` is non-zero (i.e., odd).
///
/// ### Mathematical Reasoning:
/// By Lucas' Theorem / Kummer's Theorem, `comb(d + 99, 99)` is odd if and only if
/// there are **no carries** when adding `d` and `99` in binary.
/// This is equivalent to `d & 99 == 0`.
///
/// Since `99` in binary is `0b1100011` (with bits 0, 1, 5, and 6 set to 1),
/// any valid `d` must have 0s at bit positions 0, 1, 5, and 6.
/// The remaining bit positions of `d` can be any value:
/// - Bits 2..=4 (3 bits)
/// - Bits 7..=8 (2 bits)
/// - Bits 9 and above (infinite bits)
///
/// ### Explanation of Magic Numbers:
/// To generate all such valid `d` sequentially without gaps, we take a contiguous index `m`
/// and scatter its bits to the allowable positions in `d` (returned as `v`):
/// - `(m & 0b111) << 2`: Extracts the first 3 bits of `m` (bits 0, 1, 2) and maps them to positions 2, 3, 4 of `v`.
/// - `((m >> 3) & 0b11) << 7`: Discards the first 3 bits of `m`, extracts the next 2 bits (bits 3, 4), and maps them to positions 7, 8 of `v`.
/// - `(m >> 5) << 9`: Discards the first 5 bits of `m`, and maps the remaining bits (from position 5 onwards) to positions 9 and above of `v`.
fn map_index(m: usize) -> usize {
    let mut v = 0;
    v |= (m & 0b111) << 2;
    v |= ((m >> 3) & 0b11) << 7;
    v |= (m >> 5) << 9;
    v
}

/// Computes a single output digit at index `j` of the final suffix after 100 phases.
///
/// Uses Lucas' Theorem and the Chinese Remainder Theorem to calculate the binomial coefficient
/// multiplier `comb(d + 99, 99) mod 10` for each element in O(1) time. The calculation is split
/// into a modulo 2 term (evaluated efficiently via bitwise operations over precomputed non-zero
/// indices `valid_ds`) and a sparse modulo 5 correction term.
fn compute_digit(
    j: usize,
    offset: usize,
    suffix_len: usize,
    digits: &[i32],
    valid_ds: &[usize],
) -> u8 {
    let limit = suffix_len - 1 - j;
    let n_digits = digits.len();

    // Compute the modulo 2 contribution (which translates to 5 or 0 modulo 10).
    let mut sum_mod2 = 0;
    for &d in valid_ds {
        if d > limit {
            break;
        }
        let digit = digits[(offset + j + d) % n_digits];
        sum_mod2 ^= (digit & 1) as u8;
    }
    let term1 = sum_mod2 * 5;

    // Compute modulo 5 corrections which are sparse (only non-zero for specific offsets mod 125).
    let mut term2 = 0;
    let k_limit1 = limit / 125;
    for k in 0..=k_limit1 {
        let d = 125 * k;
        let digit = digits[(offset + j + d) % n_digits] as u32;
        term2 += 6 * digit;
    }

    if limit >= 25 {
        let k_limit2 = (limit - 25) / 125;
        for k in 0..=k_limit2 {
            let d = 25 + 125 * k;
            let digit = digits[(offset + j + d) % n_digits] as u32;
            term2 += 4 * digit;
        }
    }

    ((term1 as u32 + term2) % 10) as u8
}

#[aoc(day16, part2)]
pub fn part2(input: &Input) -> usize {
    let offset = parse_number(&input.digits, 7);
    let total_len = input.digits.len() * 10000;
    // The mathematical simplification assumes the offset is in the second half of the signal.
    assert!(
        offset >= total_len / 2,
        "Offset is not in the second half of the signal"
    );
    let suffix_len = total_len - offset;

    // Precompute indices where the binomial coefficient modulo 2 is non-zero.
    let mut valid_ds = Vec::with_capacity(suffix_len / 16 + 100);
    let mut m = 0;
    loop {
        let d = map_index(m);
        if d >= suffix_len {
            break;
        }
        valid_ds.push(d);
        m += 1;
    }

    // Compute each of the 8 output digits.
    let mut ans_digits = [0u8; 8];
    for (j, ans_j) in ans_digits.iter_mut().enumerate() {
        *ans_j = compute_digit(j, offset, suffix_len, &input.digits, &valid_ds);
    }

    parse_number(&ans_digits, 8)
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
