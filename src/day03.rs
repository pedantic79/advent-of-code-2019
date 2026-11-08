use std::{num::ParseIntError, str::FromStr};

use aoc_harness::{aoc, aoc_generator};
use arrayvec::ArrayVec;

use crate::common::parse::parse_split;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
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

type Point = (isize, isize);

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    start: Point,
    end: Point,
    steps: usize,
}

impl Segment {
    fn horizontal(self) -> bool {
        self.start.1 == self.end.1
    }

    fn steps_to(self, point: Point) -> usize {
        self.steps + self.start.0.abs_diff(point.0) + self.start.1.abs_diff(point.1)
    }
}

fn segments(dvs: &[DirectionalVector]) -> Vec<Segment> {
    let mut position = (0, 0);
    let mut steps = 0;

    dvs.iter()
        .filter_map(|dv| {
            if dv.0 == 0 {
                return None;
            }

            let start = position;
            let distance = isize::try_from(dv.0).unwrap();
            let direction = match dv.1 {
                Direction::Up => (0, 1),
                Direction::Down => (0, -1),
                Direction::Left => (-1, 0),
                Direction::Right => (1, 0),
            };
            position.0 += direction.0 * distance;
            position.1 += direction.1 * distance;

            let segment = Segment {
                start,
                end: position,
                steps,
            };
            steps += dv.0;
            Some(segment)
        })
        .collect()
}

#[aoc_generator(day3)]
pub fn generator(input: &str) -> ArrayVec<Vec<Segment>, 2> {
    input
        .lines()
        .map(|line| segments(&parse_split(line, ',')))
        .collect()
}

fn in_range(value: isize, a: isize, b: isize) -> bool {
    (a.min(b)..=a.max(b)).contains(&value)
}

fn overlap_bounds(
    (first_start, first_end): (isize, isize),
    (second_start, second_end): (isize, isize),
) -> Option<(isize, isize)> {
    let first_lo = first_start.min(first_end);
    let first_hi = first_start.max(first_end);
    let second_lo = second_start.min(second_end);
    let second_hi = second_start.max(second_end);

    let overlap_lo = first_lo.max(second_lo);
    let overlap_hi = first_hi.min(second_hi);
    (overlap_lo <= overlap_hi).then_some((overlap_lo, overlap_hi))
}

fn visit_overlap_candidates(
    fixed: isize,
    (first_start, first_end): (isize, isize),
    (second_start, second_end): (isize, isize),
    horizontal: bool,
    visit: &mut impl FnMut(Point),
) {
    let Some((lo, hi)) = overlap_bounds((first_start, first_end), (second_start, second_end))
    else {
        return;
    };

    // Manhattan distance and combined steps are convex across a collinear overlap, so their
    // minima can only occur at these boundaries, segment starts, or nearest-to-origin points.
    let candidates = [
        lo,
        hi,
        0.clamp(lo, hi),
        first_start.clamp(lo, hi),
        second_start.clamp(lo, hi),
        -1,
        1,
    ];
    for (index, &varying) in candidates.iter().enumerate() {
        if candidates[..index].contains(&varying) {
            continue;
        }
        if (lo..=hi).contains(&varying) {
            let point = if horizontal {
                (varying, fixed)
            } else {
                (fixed, varying)
            };
            if point != (0, 0) {
                visit(point);
            }
        }
    }
}

// Inlining lets each call's constant orientation eliminate the coordinate-selection branch.
#[inline(always)]
fn visit_collinear_overlap_candidates(
    first: Segment,
    second: Segment,
    horizontal: bool,
    visit: &mut impl FnMut(Point),
) {
    let (first_fixed, first_range, second_fixed, second_range) = if horizontal {
        (
            first.start.1,
            (first.start.0, first.end.0),
            second.start.1,
            (second.start.0, second.end.0),
        )
    } else {
        (
            first.start.0,
            (first.start.1, first.end.1),
            second.start.0,
            (second.start.1, second.end.1),
        )
    };

    if first_fixed != second_fixed {
        return;
    }

    visit_overlap_candidates(first_fixed, first_range, second_range, horizontal, visit);
}

fn visit_intersection_candidates(first: Segment, second: Segment, visit: &mut impl FnMut(Point)) {
    match (first.horizontal(), second.horizontal()) {
        (true, false) => {
            let point = (second.start.0, first.start.1);
            if point != (0, 0)
                && in_range(point.0, first.start.0, first.end.0)
                && in_range(point.1, second.start.1, second.end.1)
            {
                visit(point);
            }
        }
        (false, true) => visit_intersection_candidates(second, first, visit),
        (true, true) => visit_collinear_overlap_candidates(first, second, true, visit),
        (false, false) => visit_collinear_overlap_candidates(first, second, false, visit),
    }
}

// Keep the selected metric specialized inside the hot segment-pair loop.
#[inline(always)]
fn minimum_intersection_score(
    input: &[Vec<Segment>],
    mut score: impl FnMut(Segment, Segment, Point) -> usize,
) -> Option<usize> {
    let mut minimum = None;
    for &first in &input[0] {
        for &second in &input[1] {
            visit_intersection_candidates(first, second, &mut |point| {
                let candidate = score(first, second, point);
                minimum = Some(minimum.map_or(candidate, |best: usize| best.min(candidate)));
            });
        }
    }
    minimum
}

#[aoc(day3, part1)]
pub fn part1(input: &[Vec<Segment>]) -> usize {
    minimum_intersection_score(input, |_, _, point| {
        point.0.unsigned_abs() + point.1.unsigned_abs()
    })
    .unwrap()
}

#[aoc(day3, part2)]
pub fn part2(input: &[Vec<Segment>]) -> usize {
    minimum_intersection_score(input, |first, second, point| {
        first.steps_to(point) + second.steps_to(point)
    })
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE1: &str = r"R75,D30,R83,U83,L12,D49,R71,U7,L72
U62,R66,U55,R34,D71,R55,D58,R83";
    const SAMPLE2: &str = r"R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51
U98,R91,D20,R16,D67,R40,U7,R15,U6,R7";
    const COLLINEAR_OVERLAP: &str = "R5\nR3";
    const ORIGIN_REVISIT: &str = "R1,L1,U2\nD1,U2";

    fn expand_wire(wire: &[Segment]) -> Vec<Point> {
        let mut points = Vec::new();
        for segment in wire {
            let direction = (
                (segment.end.0 - segment.start.0).signum(),
                (segment.end.1 - segment.start.1).signum(),
            );
            let length =
                segment.start.0.abs_diff(segment.end.0) + segment.start.1.abs_diff(segment.end.1);
            for distance in 1..=length {
                let distance = isize::try_from(distance).unwrap();
                points.push((
                    segment.start.0 + direction.0 * distance,
                    segment.start.1 + direction.1 * distance,
                ));
            }
        }
        points
    }

    fn reference_answers(input: &[Vec<Segment>]) -> (Option<usize>, Option<usize>) {
        let mut first_steps = std::collections::HashMap::new();
        for (point, steps) in expand_wire(&input[0]).into_iter().zip(1..) {
            first_steps.entry(point).or_insert(steps);
        }
        let second = expand_wire(&input[1]);

        let closest = second
            .iter()
            .filter(|&&point| point != (0, 0) && first_steps.contains_key(&point))
            .map(|point| point.0.unsigned_abs() + point.1.unsigned_abs())
            .min();
        let fewest_steps = second
            .into_iter()
            .zip(1..)
            .filter(|(point, _)| *point != (0, 0))
            .filter_map(|(point, steps)| first_steps.get(&point).map(|&first| first + steps))
            .min();

        (closest, fewest_steps)
    }

    fn short_wires() -> Vec<Vec<Segment>> {
        const DIRECTIONS: [Direction; 4] = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let mut wires = Vec::new();

        for first_direction in DIRECTIONS {
            for first_length in 1..=2 {
                for second_direction in DIRECTIONS {
                    for second_length in 1..=2 {
                        wires.push(segments(&[
                            DirectionalVector(first_length, first_direction),
                            DirectionalVector(second_length, second_direction),
                        ]));
                    }
                }
            }
        }
        wires
    }

    #[test]
    fn generator_test() {
        let input = generator("R2,U0,L1\nD1");

        assert_eq!(input[0].len(), 2);
        assert_eq!(input[0][0].start, (0, 0));
        assert_eq!(input[0][0].end, (2, 0));
        assert_eq!(input[0][0].steps, 0);
        assert_eq!(input[0][1].start, (2, 0));
        assert_eq!(input[0][1].end, (1, 0));
        assert_eq!(input[0][1].steps, 2);
    }

    #[test]
    fn overlap_bounds_handles_direction_and_inclusive_endpoints() {
        assert_eq!(overlap_bounds((1, 5), (3, 7)), Some((3, 5)));
        assert_eq!(overlap_bounds((5, 1), (7, 3)), Some((3, 5)));
        assert_eq!(overlap_bounds((1, 3), (3, 5)), Some((3, 3)));
        assert_eq!(overlap_bounds((1, 2), (3, 4)), None);
    }

    #[test]
    pub fn part1_test() {
        assert_eq!(part1(&generator(SAMPLE1)), 159);
        assert_eq!(part1(&generator(SAMPLE2)), 135);
        assert_eq!(part1(&generator(COLLINEAR_OVERLAP)), 1);
        assert_eq!(part1(&generator(ORIGIN_REVISIT)), 1);
    }

    #[test]
    pub fn part2_test() {
        assert_eq!(part2(&generator(SAMPLE1)), 610);
        assert_eq!(part2(&generator(SAMPLE2)), 410);
        assert_eq!(part2(&generator("R2,U1,L1,D1\nU1,R1,D1")), 4);
        assert_eq!(part2(&generator(COLLINEAR_OVERLAP)), 2);
        assert_eq!(part2(&generator(ORIGIN_REVISIT)), 6);
    }

    #[test]
    fn segment_intersections_match_point_reference_for_short_wires() {
        let wires = short_wires();
        for first in &wires {
            for second in &wires {
                let input = [first.clone(), second.clone()];
                let closest = minimum_intersection_score(&input, |_, _, point| {
                    point.0.unsigned_abs() + point.1.unsigned_abs()
                });
                let fewest_steps = minimum_intersection_score(&input, |first, second, point| {
                    first.steps_to(point) + second.steps_to(point)
                });
                assert_eq!((closest, fewest_steps), reference_answers(&input));
            }
        }
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
