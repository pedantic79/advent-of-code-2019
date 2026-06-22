use crate::intcode::{IntCode, MachineState};
use aoc_runner_derive::{aoc, aoc_generator};

#[aoc_generator(day17)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

pub fn print_grid(grid: &[Vec<u8>]) {
    for line in grid {
        let s = str::from_utf8(line).unwrap();
        println!("{s}")
    }
}

#[aoc(day17, part1)]
pub fn part1(ic: &IntCode<i64>) -> usize {
    let grid = get_map(ic);

    let mut sum = 0;
    let h = grid.len();
    let w = grid[0].len();

    for r in 1..h - 1 {
        for c in 1..w - 1 {
            if grid[r][c] == b'#'
                && grid[r - 1][c] == b'#'
                && grid[r + 1][c] == b'#'
                && grid[r][c - 1] == b'#'
                && grid[r][c + 1] == b'#'
            {
                sum += r * c;
            }
        }
    }

    sum
}

fn turn_left(dir: (i32, i32)) -> (i32, i32) {
    match dir {
        (-1, 0) => (0, -1), // Up -> Left
        (0, -1) => (1, 0),  // Left -> Down
        (1, 0) => (0, 1),   // Down -> Right
        (0, 1) => (-1, 0),  // Right -> Up
        _ => panic!("invalid dir"),
    }
}

fn turn_right(dir: (i32, i32)) -> (i32, i32) {
    match dir {
        (-1, 0) => (0, 1),  // Up -> Right
        (0, 1) => (1, 0),   // Right -> Down
        (1, 0) => (0, -1),  // Down -> Left
        (0, -1) => (-1, 0), // Left -> Up
        _ => panic!("invalid dir"),
    }
}

struct CompressedRoutines {
    main: Vec<char>,
    a: Vec<String>,
    b: Vec<String>,
    c: Vec<String>,
}

fn compress(
    path: &[String],
    a: Option<&[String]>,
    b: Option<&[String]>,
    c: Option<&[String]>,
    main: &mut Vec<char>,
    index: usize,
) -> Option<CompressedRoutines> {
    if main.len() * 2 > 21 {
        return None;
    }

    if index == path.len() {
        let final_a = a
            .map(|s| s.to_vec())
            .unwrap_or_else(|| vec!["R".to_string(), "2".to_string()]);
        let final_b = b
            .map(|s| s.to_vec())
            .unwrap_or_else(|| vec!["R".to_string(), "2".to_string()]);
        let final_c = c
            .map(|s| s.to_vec())
            .unwrap_or_else(|| vec!["R".to_string(), "2".to_string()]);
        return Some(CompressedRoutines {
            main: main.clone(),
            a: final_a,
            b: final_b,
            c: final_c,
        });
    }

    // Try matching existing definitions
    if let Some(a_seq) =
        a.filter(|seq| index + seq.len() <= path.len() && &path[index..index + seq.len()] == *seq)
    {
        main.push('A');
        if let Some(res) = compress(path, a, b, c, main, index + a_seq.len()) {
            return Some(res);
        }
        main.pop();
    }

    if let Some(b_seq) =
        b.filter(|seq| index + seq.len() <= path.len() && &path[index..index + seq.len()] == *seq)
    {
        main.push('B');
        if let Some(res) = compress(path, a, b, c, main, index + b_seq.len()) {
            return Some(res);
        }
        main.pop();
    }

    if let Some(c_seq) =
        c.filter(|seq| index + seq.len() <= path.len() && &path[index..index + seq.len()] == *seq)
    {
        main.push('C');
        if let Some(res) = compress(path, a, b, c, main, index + c_seq.len()) {
            return Some(res);
        }
        main.pop();
    }

    // Try defining a new function.
    // Movement functions must start with a turn and end with a distance, so they have an even number of tokens.
    let max_len = path.len() - index;
    if a.is_none() {
        for len in (2..=max_len).step_by(2) {
            let chunk = &path[index..index + len];
            let formatted_len: usize = chunk.iter().map(|s| s.len()).sum::<usize>() + len - 1;
            if formatted_len > 20 {
                break;
            }
            main.push('A');
            if let Some(res) = compress(path, Some(chunk), b, c, main, index + len) {
                return Some(res);
            }
            main.pop();
        }
    } else if b.is_none() {
        for len in (2..=max_len).step_by(2) {
            let chunk = &path[index..index + len];
            let formatted_len: usize = chunk.iter().map(|s| s.len()).sum::<usize>() + len - 1;
            if formatted_len > 20 {
                break;
            }
            main.push('B');
            if let Some(res) = compress(path, a, Some(chunk), c, main, index + len) {
                return Some(res);
            }
            main.pop();
        }
    } else if c.is_none() {
        for len in (2..=max_len).step_by(2) {
            let chunk = &path[index..index + len];
            let formatted_len: usize = chunk.iter().map(|s| s.len()).sum::<usize>() + len - 1;
            if formatted_len > 20 {
                break;
            }
            main.push('C');
            if let Some(res) = compress(path, a, b, Some(chunk), main, index + len) {
                return Some(res);
            }
            main.pop();
        }
    }

    None
}

fn get_map(ic: &IntCode<i64>) -> Vec<Vec<u8>> {
    let mut ic_map = ic.clone();
    loop {
        match ic_map.run() {
            MachineState::Halted => break,
            MachineState::HasOutput => {}
            MachineState::WaitingForInput => {
                panic!("Unexpectedly waiting for input when building map!");
            }
        }
    }

    let mut grid = Vec::new();
    let mut current_row = Vec::new();
    while let Some(out) = ic_map.get_output() {
        let b = out as u8;
        if b == b'\n' {
            if !current_row.is_empty() {
                grid.push(current_row);
                current_row = Vec::new();
            }
        } else {
            current_row.push(b);
        }
    }
    if !current_row.is_empty() {
        grid.push(current_row);
    }
    grid
}

fn find_robot(grid: &mut [Vec<u8>]) -> (usize, usize, (i32, i32)) {
    let mut robot_r = 0;
    let mut robot_c = 0;
    let mut robot_dir = (0, 0);
    for (r, row) in grid.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            let ch = *cell;
            if ch == b'^' || ch == b'v' || ch == b'<' || ch == b'>' {
                robot_r = r;
                robot_c = c;
                robot_dir = match ch {
                    b'^' => (-1, 0),
                    b'v' => (1, 0),
                    b'<' => (0, -1),
                    b'>' => (0, 1),
                    _ => unreachable!(),
                };
                *cell = b'#'; // Treat robot cell as scaffold
            }
        }
    }
    (robot_r, robot_c, robot_dir)
}

fn generate_path(
    grid: &[Vec<u8>],
    start_r: usize,
    start_c: usize,
    start_dir: (i32, i32),
) -> Vec<String> {
    let h = grid.len();
    let w = grid[0].len();
    let mut path = Vec::new();
    let mut r = start_r as i32;
    let mut c = start_c as i32;
    let mut dir = start_dir;

    loop {
        let mut steps = 0;
        loop {
            let next_r = r + dir.0;
            let next_c = c + dir.1;
            if next_r >= 0
                && next_r < h as i32
                && next_c >= 0
                && next_c < w as i32
                && grid[next_r as usize][next_c as usize] == b'#'
            {
                r = next_r;
                c = next_c;
                steps += 1;
                continue;
            }
            break;
        }

        if steps > 0 {
            path.push(steps.to_string());
        }

        let left_d = turn_left(dir);
        let left_r = r + left_d.0;
        let left_c = c + left_d.1;
        let can_left = left_r >= 0
            && left_r < h as i32
            && left_c >= 0
            && left_c < w as i32
            && grid[left_r as usize][left_c as usize] == b'#';

        let right_d = turn_right(dir);
        let right_r = r + right_d.0;
        let right_c = c + right_d.1;
        let can_right = right_r >= 0
            && right_r < h as i32
            && right_c >= 0
            && right_c < w as i32
            && grid[right_r as usize][right_c as usize] == b'#';

        if can_left {
            path.push("L".to_string());
            dir = left_d;
        } else if can_right {
            path.push("R".to_string());
            dir = right_d;
        } else {
            break;
        }
    }

    path
}

fn run_robot(ic: &IntCode<i64>, routines: &CompressedRoutines) -> usize {
    let main_str = routines
        .main
        .iter()
        .map(|ch| ch.to_string())
        .collect::<Vec<_>>()
        .join(",")
        + "\n";
    let a_str = routines.a.join(",") + "\n";
    let b_str = routines.b.join(",") + "\n";
    let c_str = routines.c.join(",") + "\n";
    let video_str = "n\n";

    let mut ic_run = ic.clone();
    ic_run.set_values(&[(0, 2)]);

    for ch in main_str.chars() {
        ic_run.set_input(ch as i64);
    }
    for ch in a_str.chars() {
        ic_run.set_input(ch as i64);
    }
    for ch in b_str.chars() {
        ic_run.set_input(ch as i64);
    }
    for ch in c_str.chars() {
        ic_run.set_input(ch as i64);
    }
    for ch in video_str.chars() {
        ic_run.set_input(ch as i64);
    }

    let mut last_output = 0;
    let mut debug_output = String::new();
    loop {
        match ic_run.run() {
            MachineState::Halted => {
                while let Some(out) = ic_run.get_output() {
                    if (0..128).contains(&out) {
                        debug_output.push(out as u8 as char);
                    }
                    last_output = out;
                }
                break;
            }
            MachineState::HasOutput => {
                while let Some(out) = ic_run.get_output() {
                    if (0..128).contains(&out) {
                        debug_output.push(out as u8 as char);
                    }
                    last_output = out;
                }
            }
            MachineState::WaitingForInput => {
                panic!(
                    "Waiting for input! (current debug output: {})",
                    debug_output
                );
            }
        }
    }

    last_output as usize
}

#[aoc(day17, part2)]
pub fn part2(ic: &IntCode<i64>) -> usize {
    let mut grid = get_map(ic);
    let (robot_r, robot_c, robot_dir) = find_robot(&mut grid);
    let path = generate_path(&grid, robot_r, robot_c, robot_dir);

    // 4. Compress the path into routines
    let mut main_routine = Vec::new();
    let routines =
        compress(&path, None, None, None, &mut main_routine, 0).expect("Failed to compress path!");

    run_robot(ic, &routines)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "..#..........\n..#..........\n#######...###\n#.#...#...#.#\n#############\n..#...#...#..\n..#####...^..";

    #[test]
    pub fn part1_test() {
        let grid: Vec<Vec<u8>> = SAMPLE
            .lines()
            .map(|line| line.as_bytes().to_vec())
            .filter(|row| !row.is_empty())
            .collect();
        let h = grid.len();
        let w = grid[0].len();
        let mut sum = 0;
        for r in 1..h - 1 {
            for c in 1..w - 1 {
                if grid[r][c] == b'#'
                    && grid[r - 1][c] == b'#'
                    && grid[r + 1][c] == b'#'
                    && grid[r][c - 1] == b'#'
                    && grid[r][c + 1] == b'#'
                {
                    sum += r * c;
                }
            }
        }
        assert_eq!(sum, 76);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day17.txt");
        const ANSWERS: (usize, usize) = (6212, 1016741);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
