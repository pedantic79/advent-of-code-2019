use aoc_harness::{aoc, aoc_generator};
use rustc_hash::FxHashMap as HashMap;

use crate::intcode::{IntCode, MachineState};

#[aoc_generator(day11)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

#[inline(always)]
fn pack(x: i32, y: i32) -> u64 {
    ((x as u64) << 32) | (y as u32 as u64)
}

#[inline(always)]
fn unpack(packed: u64) -> (i32, i32) {
    ((packed >> 32) as i32, packed as i32)
}

fn run_robot(ic: &IntCode<i64>, start_panel_color: i64) -> HashMap<u64, i64> {
    let mut ic = ic.clone();
    let mut grid = HashMap::with_capacity_and_hasher(2048, Default::default());
    let mut current_pos = (0, 0);
    // Directions: 0 = Up, 1 = Right, 2 = Down, 3 = Left
    let mut current_dir = 0;

    if start_panel_color != 0 {
        grid.insert(pack(current_pos.0, current_pos.1), start_panel_color);
    }

    loop {
        let packed_pos: u64 = pack(current_pos.0, current_pos.1);
        let current_color = *grid.get(&packed_pos).unwrap_or(&0);
        ic.set_input(current_color);

        let state1 = ic.run();
        if state1 == MachineState::Halted {
            break;
        }
        let paint_color = ic.get_output().unwrap();

        let state2 = ic.run();
        assert_eq!(state2, MachineState::HasOutput);
        let turn_dir = ic.get_output().unwrap();

        grid.insert(packed_pos, paint_color);

        if turn_dir == 0 {
            current_dir = (current_dir + 3) % 4;
        } else if turn_dir == 1 {
            current_dir = (current_dir + 1) % 4;
        } else {
            panic!("Invalid turn direction: {turn_dir}");
        }

        match current_dir {
            0 => current_pos.1 += 1, // Up
            1 => current_pos.0 += 1, // Right
            2 => current_pos.1 -= 1, // Down
            3 => current_pos.0 -= 1, // Left
            _ => unreachable!(),
        }
    }

    grid
}

#[aoc(day11, part1)]
pub fn part1(ic: &IntCode<i64>) -> usize {
    let grid = run_robot(ic, 0);
    grid.len()
}

#[aoc(day11, part2)]
pub fn part2(ic: &IntCode<i64>) -> String {
    let grid = run_robot(ic, 1);

    let white_panels: Vec<(i32, i32)> = grid
        .iter()
        .filter(|&(_, &color)| color == 1)
        .map(|(&pos, _)| unpack(pos))
        .collect();

    let min_x = white_panels.iter().map(|&(x, _)| x).min().unwrap_or(0);
    let max_x = white_panels.iter().map(|&(x, _)| x).max().unwrap_or(0);
    let min_y = white_panels.iter().map(|&(_, y)| y).min().unwrap_or(0);
    let max_y = white_panels.iter().map(|&(_, y)| y).max().unwrap_or(0);

    let mut ocr_string =
        String::with_capacity(((max_x - min_x + 1) * (max_y - min_y + 1)) as usize);
    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let packed = pack(x, y);
            let color = grid.get(&packed).copied().unwrap_or(0);
            let char_val = if color == 1 { '#' } else { '.' };
            ocr_string.push(char_val);
        }
        ocr_string.push('\n');
    }

    if ocr_string.ends_with('\n') {
        ocr_string.pop();
    }

    advent_of_code_ocr::parse_string_to_letters(&ocr_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn sample_test() {
        let program = "104,1,104,0,104,0,104,0,104,1,104,0,104,1,104,0,104,0,104,1,104,1,104,0,104,1,104,0,99";
        let ic = generator(program);
        assert_eq!(part1(&ic), 6);
    }

    mod regression {
        use super::*;

        const ANSWERS: (usize, &str) = (1894, "JKZLZJBH");

        #[test]
        pub fn test() {
            let output = generator(&aoc_harness::input!(11));

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
