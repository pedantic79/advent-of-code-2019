use aoc_harness::{aoc, aoc_generator};
use smallstr::SmallString;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Input {
    start: SmallString<[u8; 6]>,
    end: SmallString<[u8; 6]>,
}

fn inc(arr: &mut [u8; 6]) {
    let i = arr.iter().rposition(|x| *x != b'9').unwrap_or(0);
    let temp = arr[i] + 1;
    arr[i..].fill(temp);
}

fn check(num: &[u8]) -> bool {
    num.array_windows().any(|[a, b]| a == b) && num.array_windows().all(|[a, b]| a <= b)
}

fn check2(num: &[u8]) -> bool {
    check(num) && num.chunk_by(|a, b| a == b).any(|group| group.len() == 2)
}

#[aoc_generator(day4)]
pub fn generator(input: &str) -> Input {
    let (a, b) = input.split_once('-').unwrap();

    Input {
        start: a.into(),
        end: b.into(),
    }
}

fn solve<F: Fn(&[u8]) -> bool>(Input { start, end }: &Input, f: F) -> usize {
    let mut start = start.clone().into_inner().unwrap();
    let end = end.as_ref();

    let mut count = 0;
    while &start[..] < end {
        if f(&start) {
            count += 1;
        }

        inc(&mut start);
    }

    count
}

#[aoc(day4, part1)]
pub fn part1(input: &Input) -> usize {
    solve(input, check)
}

#[aoc(day4, part2)]
pub fn part2(input: &Input) -> usize {
    solve(input, check2)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod regression {
        use super::*;

        const ANSWERS: (usize, usize) = (945, 617);

        #[test]
        pub fn test() {
            let output = generator(&aoc_harness::input!(4));

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
