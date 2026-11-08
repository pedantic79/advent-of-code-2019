use ahash::HashMap;
use aoc_harness::{aoc, aoc_generator};
use itertools::Itertools;
use num::Integer;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Position {
    y: i64,
    x: i64,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Direction {
    x: i64,
    y: i64,
}

#[aoc_generator(day10)]
fn generator(input: &str) -> Vec<Position> {
    let mut asteroids = Vec::new();

    for (y, line) in input.lines().enumerate() {
        for (x, cell) in line.bytes().enumerate() {
            if cell == b'#' {
                asteroids.push(Position {
                    y: y as i64,
                    x: x as i64,
                });
            }
        }
    }

    asteroids
}

impl Position {
    fn get_direction(&self, other: &Self) -> Direction {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let gcd = dx.gcd(&dy);

        Direction {
            x: dx / gcd,
            y: dy / gcd,
        }
    }

    fn multiple_reachable(&self, others: &[Self]) -> HashMap<Direction, Vec<Position>> {
        others
            .iter()
            .filter(|x| *x != self)
            .fold(HashMap::default(), |mut hm, a| {
                let dir = a.get_direction(self);
                hm.entry(dir).or_default().push(*a);
                hm
            })
    }

    fn distance(&self, other: &Self) -> u64 {
        let x = self.x.abs_diff(other.x);
        let y = self.y.abs_diff(other.y);

        x * x + y * y
    }
}

impl Direction {
    fn angle(&self) -> f64 {
        let radians = (self.y as f64).atan2(self.x as f64) - std::f64::consts::FRAC_PI_2;
        radians.rem_euclid(std::f64::consts::TAU)
    }
}

fn find_max_reachable(asteroids: &[Position]) -> (Position, HashMap<Direction, Vec<Position>>) {
    asteroids
        .iter()
        .map(|asteroid| (*asteroid, asteroid.multiple_reachable(asteroids)))
        .max_by_key(|pos| pos.1.len())
        .unwrap()
}

#[aoc(day10, part1)]
fn part1(asteroids: &[Position]) -> usize {
    find_max_reachable(asteroids).1.len()
}

#[aoc(day10, part2)]
fn part2(asteroids: &[Position]) -> i64 {
    let (station, best) = find_max_reachable(asteroids);

    let mut pos = best
        .into_iter()
        .sorted_by(|a, b| {
            let a = a.0.angle();
            let b = b.0.angle();

            a.total_cmp(&b)
        })
        .nth(199)
        .map(|x| x.1)
        .unwrap();

    pos.sort_by_key(|x| station.distance(x));
    pos.first().map(|p| p.x * 100 + p.y).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##";

    #[test]
    pub fn input_test() {
        println!("{:?}", generator(SAMPLE));

        // assert_eq!(generator(SAMPLE), Input());
    }

    #[test]
    pub fn part1_test() {
        assert_eq!(part1(&generator(SAMPLE)), 210);
    }

    #[test]
    pub fn part2_test() {
        assert_eq!(part2(&generator(SAMPLE)), 802);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day10.txt");
        const ANSWERS: (usize, i64) = (344, 2732);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
