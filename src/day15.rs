use aoc_runner_derive::{aoc, aoc_generator};

use crate::intcode::{IntCode, MachineState};
use rustc_hash::FxHashSet as HashSet;
use std::collections::VecDeque;

struct GridMap {
    open_positions: HashSet<(i32, i32)>,
    oxygen_pos: (i32, i32),
}

fn build_grid_map(ic: &IntCode<i64>) -> GridMap {
    let mut open_positions = HashSet::default();
    let mut oxygen_pos = (0, 0);
    let mut visited = HashSet::default();
    let mut queue = VecDeque::new();

    // The starting position (0, 0) is open and visited
    open_positions.insert((0, 0));
    visited.insert((0, 0));
    queue.push_back((ic.clone(), (0, 0)));

    while let Some((ic, pos)) = queue.pop_front() {
        let directions = [
            (1, (pos.0, pos.1 + 1)), // North
            (2, (pos.0, pos.1 - 1)), // South
            (3, (pos.0 - 1, pos.1)), // West
            (4, (pos.0 + 1, pos.1)), // East
        ];

        let num_unvisited = directions
            .iter()
            .filter(|(_, next_pos)| !visited.contains(next_pos))
            .count();

        let mut ic_opt = Some(ic);
        let mut unvisited_seen = 0;

        for (d, next_pos) in directions {
            if visited.contains(&next_pos) {
                continue;
            }
            unvisited_seen += 1;

            // Optimization: If this is the last unvisited direction to explore from the
            // current cell, we can consume (take) ownership of the IntCode computer state
            // rather than cloning it, avoiding unnecessary clones/allocations.
            let mut next_ic = if unvisited_seen == num_unvisited {
                ic_opt.take().unwrap()
            } else {
                ic_opt.as_ref().unwrap().clone()
            };
            next_ic.set_input(d);

            match next_ic.run() {
                MachineState::HasOutput => {
                    let status = next_ic.get_output().expect("Expected output");
                    match status {
                        0 => {
                            // Hit a wall, position didn't change
                            visited.insert(next_pos);
                        }
                        1 => {
                            // Moved one step to empty space
                            visited.insert(next_pos);
                            open_positions.insert(next_pos);
                            queue.push_back((next_ic, next_pos));
                        }
                        2 => {
                            // Found the oxygen system!
                            visited.insert(next_pos);
                            open_positions.insert(next_pos);
                            oxygen_pos = next_pos;
                            queue.push_back((next_ic, next_pos));
                        }
                        _ => panic!("Unexpected status from repair droid: {}", status),
                    }
                }
                MachineState::Halted => {
                    // Treat halts (e.g. in mock tests) as hitting a wall/boundary
                    visited.insert(next_pos);
                }
                state => panic!("Unexpected machine state: {:?}", state),
            }
        }
    }

    GridMap {
        open_positions,
        oxygen_pos,
    }
}

#[aoc_generator(day15)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

#[aoc(day15, part1)]
pub fn part1(ic: &IntCode<i64>) -> usize {
    let map = build_grid_map(ic);

    // BFS to find the shortest path from (0, 0) to oxygen_pos
    let mut queue = VecDeque::new();
    let mut visited = HashSet::default();

    visited.insert((0, 0));
    queue.push_back(((0, 0), 0));

    while let Some((pos, dist)) = queue.pop_front() {
        if pos == map.oxygen_pos {
            return dist;
        }

        for next_pos in [
            (pos.0, pos.1 + 1),
            (pos.0, pos.1 - 1),
            (pos.0 - 1, pos.1),
            (pos.0 + 1, pos.1),
        ] {
            if map.open_positions.contains(&next_pos) && visited.insert(next_pos) {
                queue.push_back((next_pos, dist + 1));
            }
        }
    }

    panic!("Oxygen system not found!");
}

#[aoc(day15, part2)]
pub fn part2(ic: &IntCode<i64>) -> usize {
    let map = build_grid_map(ic);

    // BFS to find the maximum time to fill the open positions with oxygen
    let mut queue = VecDeque::new();
    let mut filled = HashSet::default();

    filled.insert(map.oxygen_pos);
    queue.push_back((map.oxygen_pos, 0));

    let mut max_time = 0;

    while let Some((pos, time)) = queue.pop_front() {
        max_time = max_time.max(time);

        for next_pos in [
            (pos.0, pos.1 + 1),
            (pos.0, pos.1 - 1),
            (pos.0 - 1, pos.1),
            (pos.0 + 1, pos.1),
        ] {
            if map.open_positions.contains(&next_pos) && filled.insert(next_pos) {
                queue.push_back((next_pos, time + 1));
            }
        }
    }

    max_time
}

#[cfg(test)]
mod tests {
    use super::*;

    // A mock Intcode program that expects 4 (East) then 2 (South) to reach the target.
    // 0: input -> 33
    // 2: equals(33, 4 [immediate], 35)
    // 6: jump-if-true(35, 12 [immediate])
    // 9: output 0, halt
    // 12: output 1
    // 14: input -> 33
    // 16: equals(33, 2 [immediate], 35)
    // 20: jump-if-true(35, 26 [immediate])
    // 23: output 0, halt
    // 26: output 2, halt
    const MOCK_PROGRAM: &str = "3,33,1008,33,4,35,1005,35,12,104,0,99,104,1,3,33,1008,33,2,35,1005,35,26,104,0,99,104,2,99,0,0,0,0,0,0,0";

    #[test]
    pub fn part1_mock_test() {
        let ic = generator(MOCK_PROGRAM);
        assert_eq!(part1(&ic), 2);
    }

    #[test]
    pub fn part2_mock_test() {
        let ic = generator(MOCK_PROGRAM);
        assert_eq!(part2(&ic), 2);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day15.txt");
        const ANSWERS: (usize, usize) = (258, 372);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
