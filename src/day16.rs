use aoc_harness::{aoc, aoc_generator};
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

/// Maps a dense counter `m = 0, 1, 2, ...` to the sparse set of `d` values
/// where `C(99+d, 99)` is odd, i.e. where `d & 99 == 0`.
///
/// Since `99 = 0b1100011` forces bit positions 0, 1, 5, 6 of `d` to zero,
/// the free positions are bits 2-4, 7-8, and 9+. This function scatters
/// `m`'s bits into those gaps:
///   - `(m & 0b111) << 2`:       bits 0-2 of `m` → positions 2-4 of `d`
///   - `((m >> 3) & 0b11) << 7`: bits 3-4 of `m` → positions 7-8 of `d`
///   - `(m >> 5) << 9`:          bits 5+ of `m`  → positions 9+ of `d`
///
/// See `compute_digit` for why these indices matter.
fn map_index(m: usize) -> usize {
    let mut v = 0;
    v |= (m & 0b111) << 2;
    v |= ((m >> 3) & 0b11) << 7;
    v |= (m >> 5) << 9;
    v
}

/// Computes a single output digit at index `j` of the final suffix after 100 phases,
/// replacing the naive 100-pass iterative simulation with a direct closed-form evaluation.
///
/// ### Why binomial coefficients appear
///
/// Each phase of the Part 2 FFT is a reverse prefix sum mod 10. Applying this
/// operation k times is equivalent to convolving with binomial coefficients:
///
///   after k phases: result\[i\] = Σ_d C(k-1+d, k-1) · original\[i+d\]   (mod 10)
///
/// For k=100 phases, the value at suffix position `j` is:
///
///   result\[j\] = Σ_{d=0}^{n-1-j} C(99+d, 99) · original\[j+d\]   (mod 10)
///
/// Since we only need 8 output digits, we evaluate this formula directly rather
/// than running 100 iterative passes over the entire ~522K-element suffix.
///
/// ### Exploiting sparsity via CRT
///
/// Evaluating the sum naively still requires ~522K multiply-adds per digit. The
/// key insight is that `C(99+d, 99) mod 10` is **sparse** — most terms are
/// zero — so we can skip them entirely.
///
/// ### CRT decomposition: mod 10 = mod 2 × mod 5
///
/// By the Chinese Remainder Theorem, we compute `C(99+d, 99) mod 2` and
/// `C(99+d, 99) mod 5` separately, then combine them.
///
/// While both Lucas' and Kummer's theorems describe binomial coefficients modulo a prime `p`,
/// they have different uses here:
/// - **Kummer's Theorem** states that the power of `p` dividing `C(a+b, a)` is equal to the number of
///   carries when adding `a` and `b` in base `p`. This provides a zero/nonzero test.
/// - **Lucas' Theorem** computes the exact residue `C(n, k) mod p` as a product of digit-wise binomial
///   coefficients in base `p`. This provides the actual residue value.
///
/// **Mod 2 (via Kummer's Theorem):**
/// Since any residue mod 2 is either 0 or 1, we only need to know whether `C(99+d, 99)` is odd (value 1)
/// or even (value 0). By Kummer's Theorem, `C(99+d, 99)` is odd (divisible by `2^0`) iff there are
/// **zero carries** when adding `d` and `99` in binary (base 2). This simplifies to the bitwise check
/// `d & 99 == 0`. Since 99 (`0b1100011`) has 4 bits set, only 1/16 of all `d` values have no carries and
/// contribute. These are the precomputed `valid_ds` scattered by `map_index`. The weighted digit sum mod 2,
/// multiplied by `5` (the CRT lifting coefficient: 5 ≡ 1 mod 2, 5 ≡ 0 mod 5), gives the mod-2 contribution.
///
/// **Mod 5 (via Lucas' Theorem in base 5):**
/// A residue modulo 5 can be any value in `{0, 1, 2, 3, 4}`. Kummer's Theorem only tells us if a term is
/// divisible by 5 (carrying in base 5), but it cannot determine the actual non-zero value. Thus, we must use
/// **Lucas' Theorem** to compute the exact residue.
/// `99` in base 5 is `344`. By Lucas' Theorem, `C(99+d, 99) mod 5 = 0` unless each base-5 digit of `d` can
/// be added to the corresponding digit of `99` without carrying. This constrains `d`'s base-5 digits to specific
/// values, yielding non-zero contributions only at `d ≡ 0 (mod 125)` and `d ≡ 25 (mod 125)`, with respective
/// CRT-lifted multipliers of `6` and `4`. Only ~4K terms contribute per digit from each family.
///
/// ### Performance impact
///
/// | Approach | Work per digit | Total (8 digits) |
/// |----------|---------------|------------------|
/// | Iterative (100 phases × 522K) | — | ~52M ops |
/// | Closed-form mod 2 | ~32K terms | ~260K ops |
/// | Closed-form mod 5 | ~8K terms | ~67K ops |
/// | **Total closed-form** | ~40K terms | **~330K ops** |
///
/// This is a **~160× reduction** in work compared to iterative simulation.
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

        const ANSWERS: (usize, usize) = (53296082, 43310035);

        #[test]
        pub fn test() {
            let output = generator(&aoc_harness::input!(16));

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
