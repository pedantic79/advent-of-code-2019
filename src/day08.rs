use std::str::from_utf8_unchecked;

use aoc_runner_derive::aoc;
use itertools::Itertools;

fn solve_part1<const H: usize, const W: usize>(input: &str) -> usize {
    let target = input
        .as_bytes()
        .chunks(H * W)
        .sorted_by_key(|x| bytecount::count(x, b'0'))
        .next()
        .unwrap();

    bytecount::count(target, b'1') * bytecount::count(target, b'2')
}

#[aoc(day8, part1)]
pub fn part1(input: &str) -> usize {
    solve_part1::<6, 25>(input)
}

fn solve_part2<const H: usize, const W: usize>(input: &str) -> String {
    fn merge(old_layer: &mut [u8], new_layer: &[u8]) {
        for (old, &new) in old_layer.iter_mut().zip(new_layer.iter()) {
            if new != b'2' {
                *old = new
            }
        }
    }

    let mut input = input.to_string();
    // SAFETY: AOC is all ascii
    let input_bytes = unsafe { input.as_bytes_mut() };
    #[allow(clippy::manual_inspect)]
    let merged = input_bytes
        .chunks_mut(H * W)
        .rev()
        .reduce(|acc, layer| {
            merge(acc, layer);
            acc
        })
        .map(|m| {
            // Maps the 0 to . and 1 to # for ocr
            for c in m.iter_mut() {
                *c = if *c == b'0' { b'.' } else { b'#' };
            }

            m
        })
        .unwrap();

    // convert the merged buffer into a newline seperated string for ocr
    // SAFETY: AOC is all ascii
    merged
        .chunks(W)
        .map(|chunk| unsafe { from_utf8_unchecked(chunk) })
        .join("\n")
}

#[aoc(day8, part2)]
pub fn part2(input: &str) -> String {
    advent_of_code_ocr::parse_string_to_letters(&solve_part2::<6, 25>(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn part1_test() {
        assert_eq!(solve_part1::<2, 3>("123456789012"), 1);
    }

    #[test]
    pub fn part2_test() {
        assert_eq!(solve_part2::<2, 2>("0222112222120000"), ".#\n#.");
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day8.txt");
        const ANSWERS: (usize, &str) = (2480, "ZYBLH");

        #[test]
        pub fn test() {
            let output = INPUT.trim_end_matches('\n');

            assert_eq!(part1(output), ANSWERS.0);
            assert_eq!(part2(output), ANSWERS.1);
        }
    }
}
