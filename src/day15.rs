use aoc_runner_derive::{aoc, aoc_generator};

use crate::common::pairing::pair2d_b;
use crate::intcode::{IntCode, MachineState};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct GridMap {
    open_positions: Vec<bool>,
    min_x: i32,
    min_y: i32,
    width: usize,
    oxygen_pos: (i32, i32),
}

impl GridMap {
    #[inline]
    pub fn get_index(&self, x: i32, y: i32) -> Option<usize> {
        let x_offset = x - self.min_x;
        let y_offset = y - self.min_y;
        if x_offset >= 0 && y_offset >= 0 {
            let ux = x_offset as usize;
            let uy = y_offset as usize;
            if ux < self.width {
                let idx = uy * self.width + ux;
                if idx < self.open_positions.len() {
                    return Some(idx);
                }
            }
        }
        None
    }
}

const DIRECTIONS: [(i64, (i32, i32)); 4] = [
    (1, (0, 1)),  // North
    (2, (0, -1)), // South
    (3, (-1, 0)), // West
    (4, (1, 0)),  // East
];

fn explore(
    ic: &IntCode<i64>,
    pos: (i32, i32),
    open_positions: &mut Vec<(i32, i32)>,
    oxygen_pos: &mut (i32, i32),
    visited: &mut Vec<bool>,
) {
    for &(d, (dx, dy)) in &DIRECTIONS {
        let next_pos = (pos.0 + dx, pos.1 + dy);
        let idx = pair2d_b(next_pos.0, next_pos.1);
        if idx >= visited.len() {
            visited.resize(idx + 1, false);
        }
        if !visited[idx] {
            visited[idx] = true;
            let mut next_ic = ic.clone();
            next_ic.set_input(d);
            match next_ic.run() {
                MachineState::HasOutput => {
                    let status = next_ic.get_output().expect("Expected output");
                    match status {
                        0 => {
                            // Wall, droid did not move.
                        }
                        1 => {
                            // Moved to empty space
                            open_positions.push(next_pos);
                            explore(&next_ic, next_pos, open_positions, oxygen_pos, visited);
                        }
                        2 => {
                            // Found the oxygen system!
                            open_positions.push(next_pos);
                            *oxygen_pos = next_pos;
                            explore(&next_ic, next_pos, open_positions, oxygen_pos, visited);
                        }
                        _ => panic!("Unexpected status from repair droid: {}", status),
                    }
                }
                MachineState::Halted => {}
                state => panic!("Unexpected machine state: {:?}", state),
            }
        }
    }
}

fn build_grid_map(ic: &IntCode<i64>) -> GridMap {
    let mut open_positions = Vec::new();
    let mut oxygen_pos = (0, 0);
    let mut visited = Vec::new();

    // The starting position (0, 0) is open and visited
    open_positions.push((0, 0));
    let start_idx = pair2d_b(0, 0);
    visited.resize(start_idx + 1, false);
    visited[start_idx] = true;

    explore(
        ic,
        (0, 0),
        &mut open_positions,
        &mut oxygen_pos,
        &mut visited,
    );

    // Determine bounding box of open positions
    let min_x = open_positions.iter().map(|p| p.0).min().unwrap_or(0);
    let max_x = open_positions.iter().map(|p| p.0).max().unwrap_or(0);
    let min_y = open_positions.iter().map(|p| p.1).min().unwrap_or(0);
    let max_y = open_positions.iter().map(|p| p.1).max().unwrap_or(0);

    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;
    let mut grid = vec![false; width * height];

    for (x, y) in open_positions {
        let idx = ((y - min_y) as usize) * width + ((x - min_x) as usize);
        grid[idx] = true;
    }

    GridMap {
        open_positions: grid,
        min_x,
        min_y,
        width,
        oxygen_pos,
    }
}

#[aoc_generator(day15)]
pub fn generator(input: &str) -> GridMap {
    let ic = input.parse().unwrap();
    build_grid_map(&ic)
}

fn bfs(map: &GridMap, start: (i32, i32), target: Option<(i32, i32)>) -> usize {
    let mut queue = VecDeque::new();
    let mut visited = vec![false; map.open_positions.len()];

    if let Some(start_idx) = map.get_index(start.0, start.1) {
        visited[start_idx] = true;
    }
    queue.push_back((start, 0));

    let mut max_dist = 0;

    while let Some((pos, dist)) = queue.pop_front() {
        if Some(pos) == target {
            return dist;
        }
        max_dist = max_dist.max(dist);

        for &(_, (dx, dy)) in &DIRECTIONS {
            let next_pos = (pos.0 + dx, pos.1 + dy);
            if let Some(idx) = map
                .get_index(next_pos.0, next_pos.1)
                .filter(|&idx| map.open_positions[idx] && !visited[idx])
            {
                visited[idx] = true;
                queue.push_back((next_pos, dist + 1));
            }
        }
    }

    if target.is_some() {
        panic!("Target position not found!");
    }
    max_dist
}

#[aoc(day15, part1)]
pub fn part1(map: &GridMap) -> usize {
    bfs(map, (0, 0), Some(map.oxygen_pos))
}

#[aoc(day15, part2)]
pub fn part2(map: &GridMap) -> usize {
    bfs(map, map.oxygen_pos, None)
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
        let map = generator(MOCK_PROGRAM);
        assert_eq!(part1(&map), 2);
    }

    #[test]
    pub fn part2_mock_test() {
        let map = generator(MOCK_PROGRAM);
        assert_eq!(part2(&map), 2);
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
