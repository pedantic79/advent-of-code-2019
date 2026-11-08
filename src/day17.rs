use crate::intcode::{IntCode, MachineState};
use aoc_harness::{aoc, aoc_generator};

const ASCII_LINE_LIMIT: usize = 20;

#[aoc_generator(day17)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Turn {
    Left,
    Right,
}

impl Turn {
    fn as_byte(self) -> u8 {
        match self {
            Self::Left => b'L',
            Self::Right => b'R',
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Movement {
    turn: Turn,
    steps: usize,
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn turn(self, turn: Turn) -> Self {
        match (self, turn) {
            (Self::Up, Turn::Left) | (Self::Down, Turn::Right) => Self::Left,
            (Self::Up, Turn::Right) | (Self::Down, Turn::Left) => Self::Right,
            (Self::Right, Turn::Left) | (Self::Left, Turn::Right) => Self::Up,
            (Self::Right, Turn::Right) | (Self::Left, Turn::Left) => Self::Down,
        }
    }

    fn delta(self) -> (isize, isize) {
        match self {
            Self::Up => (-1, 0),
            Self::Right => (0, 1),
            Self::Down => (1, 0),
            Self::Left => (0, -1),
        }
    }
}

struct ScaffoldMap {
    cells: Vec<u8>,
    width: usize,
    height: usize,
}

impl ScaffoldMap {
    #[cfg(test)]
    fn from_ascii(output: &[u8]) -> Self {
        Self::from_bytes(output.iter().copied())
    }

    fn from_program(program: &IntCode<i64>) -> Self {
        let mut camera = program.clone();
        run_to_halt(&mut camera);
        Self::from_bytes(
            std::iter::from_fn(|| camera.get_output())
                .map(|value| u8::try_from(value).expect("camera output should be ASCII")),
        )
    }

    fn from_bytes(output: impl IntoIterator<Item = u8>) -> Self {
        let mut cells = Vec::new();
        let mut width = 0;
        let mut current_width = 0;

        for byte in output {
            if byte == b'\n' {
                if current_width == 0 {
                    continue;
                }
                if width == 0 {
                    width = current_width;
                } else {
                    assert_eq!(current_width, width, "scaffold map must be rectangular");
                }
                current_width = 0;
            } else {
                cells.push(byte);
                current_width += 1;
            }
        }

        if current_width != 0 {
            if width == 0 {
                width = current_width;
            } else {
                assert_eq!(current_width, width, "scaffold map must be rectangular");
            }
        }
        assert!(width != 0, "scaffold map must not be empty");

        let height = cells.len() / width;
        Self {
            cells,
            width,
            height,
        }
    }

    fn is_scaffold(&self, row: isize, column: isize) -> bool {
        if row < 0 || column < 0 {
            return false;
        }
        let row = usize::try_from(row).unwrap();
        let column = usize::try_from(column).unwrap();
        if row >= self.height || column >= self.width {
            return false;
        }

        is_scaffold_cell(self.cells[row * self.width + column])
    }

    fn alignment_sum(&self) -> usize {
        let mut sum = 0;
        for row in 1..self.height.saturating_sub(1) {
            for column in 1..self.width.saturating_sub(1) {
                let index = row * self.width + column;
                if [
                    index,
                    index - self.width,
                    index + self.width,
                    index - 1,
                    index + 1,
                ]
                .into_iter()
                .all(|index| is_scaffold_cell(self.cells[index]))
                {
                    sum += row * column;
                }
            }
        }
        sum
    }

    fn robot(&self) -> ((isize, isize), Direction) {
        for (index, &cell) in self.cells.iter().enumerate() {
            let direction = match cell {
                b'^' => Direction::Up,
                b'>' => Direction::Right,
                b'v' => Direction::Down,
                b'<' => Direction::Left,
                _ => continue,
            };
            return (
                (
                    isize::try_from(index / self.width).unwrap(),
                    isize::try_from(index % self.width).unwrap(),
                ),
                direction,
            );
        }

        panic!("scaffold map does not contain the vacuum robot")
    }

    fn movement_path(&self) -> Vec<Movement> {
        let (mut position, mut direction) = self.robot();
        let mut path = Vec::new();

        loop {
            let next_turn = [Turn::Left, Turn::Right].into_iter().find(|&turn| {
                let (row_delta, column_delta) = direction.turn(turn).delta();
                self.is_scaffold(position.0 + row_delta, position.1 + column_delta)
            });
            let Some(turn) = next_turn else {
                break;
            };

            direction = direction.turn(turn);
            let (row_delta, column_delta) = direction.delta();
            let mut steps = 0;
            while self.is_scaffold(position.0 + row_delta, position.1 + column_delta) {
                position.0 += row_delta;
                position.1 += column_delta;
                steps += 1;
            }
            path.push(Movement { turn, steps });
        }

        path
    }
}

fn is_scaffold_cell(cell: u8) -> bool {
    matches!(cell, b'#' | b'^' | b'v' | b'<' | b'>')
}

#[derive(Debug)]
struct MovementProgram {
    main: Vec<usize>,
    functions: [Vec<Movement>; 3],
}

fn encoded_main_len(function_count: usize) -> usize {
    function_count.saturating_mul(2).saturating_sub(1)
}

fn encoded_movement_len(movements: &[Movement]) -> usize {
    movements
        .iter()
        .map(|movement| 2 + decimal_len(movement.steps))
        .sum::<usize>()
        + movements.len().saturating_sub(1)
}

#[cfg(test)]
fn encode_movements(movements: &[Movement]) -> Vec<u8> {
    let mut result = Vec::with_capacity(encoded_movement_len(movements));
    append_movements(&mut result, movements);
    result
}

fn append_movements(result: &mut Vec<u8>, movements: &[Movement]) {
    for (index, movement) in movements.iter().enumerate() {
        if index != 0 {
            result.push(b',');
        }
        result.extend([movement.turn.as_byte(), b',']);
        append_decimal(result, movement.steps);
    }
}

fn decimal_len(value: usize) -> usize {
    usize::try_from(value.checked_ilog10().unwrap_or(0)).unwrap() + 1
}

fn append_decimal(result: &mut Vec<u8>, mut value: usize) {
    let start = result.len();
    loop {
        result.push(b'0' + u8::try_from(value % 10).unwrap());
        value /= 10;
        if value == 0 {
            break;
        }
    }
    result[start..].reverse();
}

fn compress_path(path: &[Movement]) -> Option<MovementProgram> {
    if path.is_empty() {
        return None;
    }

    let mut program = MovementProgram {
        main: Vec::with_capacity(10),
        functions: std::array::from_fn(|_| Vec::with_capacity(path.len())),
    };

    if !search_compression(path, 0, &mut program) {
        return None;
    }

    // The controller requires definitions for A, B, and C even when one is unused.
    for function in &mut program.functions {
        if function.is_empty() {
            function.push(Movement {
                turn: Turn::Left,
                steps: 1,
            });
        }
    }

    Some(program)
}

fn search_compression(path: &[Movement], cursor: usize, program: &mut MovementProgram) -> bool {
    if cursor == path.len() {
        return true;
    }
    if encoded_main_len(program.main.len() + 1) > ASCII_LINE_LIMIT {
        return false;
    }

    for function_index in 0..program.functions.len() {
        let function_len = program.functions[function_index].len();
        if function_len != 0 && path[cursor..].starts_with(&program.functions[function_index]) {
            program.main.push(function_index);
            if search_compression(path, cursor + function_len, program) {
                return true;
            }
            program.main.pop();
        }
    }

    let Some(function_index) = program.functions.iter().position(Vec::is_empty) else {
        return false;
    };

    for end in cursor + 1..=path.len() {
        let candidate = &path[cursor..end];
        if encoded_movement_len(candidate) > ASCII_LINE_LIMIT {
            break;
        }

        program.functions[function_index].extend_from_slice(candidate);
        program.main.push(function_index);
        if search_compression(path, end, program) {
            return true;
        }
        program.main.pop();
        program.functions[function_index].clear();
    }

    false
}

fn encode_program(program: &MovementProgram) -> Vec<u8> {
    let capacity = encoded_main_len(program.main.len())
        + 1
        + program
            .functions
            .iter()
            .map(|function| encoded_movement_len(function) + 1)
            .sum::<usize>()
        + 2;
    let mut input = Vec::with_capacity(capacity);

    for (index, &function) in program.main.iter().enumerate() {
        if index != 0 {
            input.push(b',');
        }
        input.push(b'A' + u8::try_from(function).unwrap());
    }
    input.push(b'\n');

    for function in &program.functions {
        append_movements(&mut input, function);
        input.push(b'\n');
    }
    input.extend_from_slice(b"n\n");

    input
}

fn run_to_halt(machine: &mut IntCode<i64>) {
    loop {
        match machine.run() {
            MachineState::HasOutput => {}
            MachineState::Halted => return,
            MachineState::WaitingForInput => {
                panic!("Intcode machine unexpectedly requested more input")
            }
        }
    }
}

fn collect_dust(program: &IntCode<i64>, movement_program: &MovementProgram) -> usize {
    let mut robot = program.clone();
    robot.set_values(&[(0, 2)]);
    for byte in encode_program(movement_program) {
        robot.set_input(i64::from(byte));
    }

    run_to_halt(&mut robot);
    let mut dust = None;
    while let Some(value) = robot.get_output() {
        if value > i64::from(u8::MAX) {
            dust = Some(value);
        }
    }

    usize::try_from(dust.expect("vacuum robot did not report collected dust")).unwrap()
}

#[aoc(day17, part1)]
pub fn part1(program: &IntCode<i64>) -> usize {
    ScaffoldMap::from_program(program).alignment_sum()
}

#[aoc(day17, part2)]
pub fn part2(program: &IntCode<i64>) -> usize {
    let map = ScaffoldMap::from_program(program);
    let path = map.movement_path();
    let movement_program = compress_path(&path).expect("movement path cannot be compressed");

    collect_dust(program, &movement_program)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALIGNMENT_SAMPLE: &str = "..#..........\n..#..........\n#######...###\n#.#...#...#.#\n#############\n..#...#...#..\n..#####...^..\n";
    const PATH_SAMPLE: &str = "#######...#####\n#.....#...#...#\n#.....#...#...#\n......#...#...#\n......#...###.#\n......#.....#.#\n^########...#.#\n......#.#...#.#\n......#########\n........#...#..\n....#########..\n....#...#......\n....#...#......\n....#...#......\n....#####......\n";

    fn camera_program(view: &str) -> IntCode<i64> {
        let mut program = Vec::new();
        for byte in view.bytes() {
            program.extend([104, i64::from(byte)]);
        }
        program.push(99);

        program
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(",")
            .parse()
            .unwrap()
    }

    #[test]
    fn sums_alignment_parameters_from_camera_output() {
        assert_eq!(part1(&camera_program(ALIGNMENT_SAMPLE)), 76);
    }

    #[test]
    fn derives_the_complete_movement_path() {
        let map = ScaffoldMap::from_ascii(PATH_SAMPLE.as_bytes());
        let path = map.movement_path();

        assert_eq!(
            encode_movements(&path),
            b"R,8,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2"
        );
    }

    #[test]
    fn encodes_multi_digit_movement_steps() {
        let movements = [
            Movement {
                turn: Turn::Left,
                steps: 1,
            },
            Movement {
                turn: Turn::Right,
                steps: 10,
            },
            Movement {
                turn: Turn::Left,
                steps: 123,
            },
        ];

        assert_eq!(encode_movements(&movements), b"L,1,R,10,L,123");
    }

    #[test]
    fn compressed_program_round_trips_within_ascii_limits() {
        let map = ScaffoldMap::from_ascii(PATH_SAMPLE.as_bytes());
        let path = map.movement_path();
        let program = compress_path(&path).unwrap();
        let expanded = program
            .main
            .iter()
            .flat_map(|&function| program.functions[function].iter().copied())
            .collect::<Vec<_>>();

        assert_eq!(expanded, path);
        assert!(encoded_main_len(program.main.len()) <= 20);
        assert!(
            program
                .functions
                .iter()
                .all(|function| encoded_movement_len(function) <= 20)
        );
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
