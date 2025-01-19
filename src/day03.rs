use std::{num::ParseIntError, str::FromStr};

use ahash::{HashMap, HashSet};
use aoc_runner_derive::{aoc, aoc_generator};
use arrayvec::ArrayVec;
use pathfinding::matrix::directions::{E, N, S, W};

use crate::common::parse::parse_split;

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct DirectionalVector(usize, Direction);

impl FromStr for DirectionalVector {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Direction::*;
        let (dir, num) = s.split_at(1);
        let num = num.parse()?;

        Ok(DirectionalVector(
            num,
            match dir {
                "U" => Up,
                "D" => Down,
                "L" => Left,
                "R" => Right,
                letter => panic!("unknown direction: {letter}"),
            },
        ))
    }
}

#[aoc_generator(day3)]
pub fn generator(input: &str) -> ArrayVec<Vec<DirectionalVector>, 2> {
    input.lines().map(|i| parse_split(i, ',')).collect()
}

fn wire(dvs: &[DirectionalVector]) -> impl Iterator<Item = (isize, isize)> + '_ {
    let mut pos = (0, 0);
    let mut positions = vec![];

    for dv in dvs {
        let dir = match dv.1 {
            Direction::Up => N,
            Direction::Down => S,
            Direction::Left => W,
            Direction::Right => E,
        };

        for _ in 0..dv.0 {
            pos.0 += dir.0;
            pos.1 += dir.1;
            positions.push(pos);
        }
    }

    positions.into_iter()
}

#[aoc(day3, part1)]
pub fn part1(input: &[Vec<DirectionalVector>]) -> usize {
    let visited: HashSet<_> = wire(&input[0]).collect();

    wire(&input[1])
        .filter(|pos| visited.contains(pos))
        .map(|pos| (pos.0.abs() + pos.1.abs()) as usize)
        .min()
        .unwrap()
}

#[aoc(day3, part2)]
pub fn part2(input: &[Vec<DirectionalVector>]) -> usize {
    let visited: HashMap<_, _> = wire(&input[0]).zip(1..).collect();

    wire(&input[1])
        .zip(1..)
        .filter(|(pos, _)| visited.contains_key(pos))
        .map(|(pos, dist)| dist + visited.get(&pos).unwrap())
        .min()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE1: &str = r"R75,D30,R83,U83,L12,D49,R71,U7,L72
U62,R66,U55,R34,D71,R55,D58,R83";
    const SAMPLE2: &str = r"R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51
U98,R91,D20,R16,D67,R40,U7,R15,U6,R7";

    #[test]
    pub fn input_test() {
        println!("{:?}", generator(SAMPLE1));

        // assert_eq!(generator(SAMPLE), Input());
    }

    #[test]
    pub fn part1_test() {
        assert_eq!(part1(&generator(SAMPLE1)), 159);
        assert_eq!(part1(&generator(SAMPLE2)), 135);
    }

    #[test]
    pub fn part2_test() {
        assert_eq!(part2(&generator(SAMPLE1)), 610);
        assert_eq!(part2(&generator(SAMPLE2)), 410);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day3.txt");
        const ANSWERS: (usize, usize) = (1674, 14012);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
