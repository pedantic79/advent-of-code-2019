use aoc_harness::{aoc, aoc_generator};

use crate::intcode::{IntCode, MachineState};

#[aoc_generator(day13)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

#[aoc(day13, part1)]
pub fn part1(ic: &IntCode<i64>) -> usize {
    let mut ic = ic.clone();

    loop {
        match ic.run() {
            MachineState::Halted => break,
            MachineState::HasOutput => {}
            MachineState::WaitingForInput => {
                panic!("Unexpectedly waiting for input in Part 1!");
            }
        }
    }

    let mut count = 0;
    while ic.get_output().is_some() {
        let _y = ic.get_output().expect("missing y coordinate");
        let tile_id = ic.get_output().expect("missing tile id");
        if tile_id == 2 {
            count += 1;
        }
    }

    count
}

#[aoc(day13, part2)]
pub fn part2(ic: &IntCode<i64>) -> i64 {
    let mut ic = ic.clone();
    ic.set_values(&[(0, 2)]);

    let mut ball_x = 0;
    let mut paddle_x = 0;
    let mut score = 0;

    let process_outputs =
        |ic: &mut IntCode<i64>, score: &mut i64, paddle_x: &mut i64, ball_x: &mut i64| {
            while let Some(x) = ic.get_output() {
                let y = ic.get_output().expect("missing y coordinate");
                let val = ic.get_output().expect("missing tile id/score");
                if x == -1 && y == 0 {
                    *score = val;
                } else {
                    match val {
                        3 => *paddle_x = x,
                        4 => *ball_x = x,
                        _ => {}
                    }
                }
            }
        };

    loop {
        match ic.run() {
            MachineState::Halted => break,
            MachineState::HasOutput => {}
            MachineState::WaitingForInput => {
                process_outputs(&mut ic, &mut score, &mut paddle_x, &mut ball_x);
                let joystick = (ball_x - paddle_x).signum();
                ic.set_input(joystick);
            }
        }
    }

    process_outputs(&mut ic, &mut score, &mut paddle_x, &mut ball_x);
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn part1_test() {
        // Construct a program that outputs:
        // x=1, y=2, tile_id=3 (paddle)
        // x=6, y=5, tile_id=4 (ball)
        // x=3, y=4, tile_id=2 (block)
        // x=4, y=4, tile_id=2 (block)
        // then halts (99).
        // Instructions: 104, val
        let program = "104,1,104,2,104,3,104,6,104,5,104,4,104,3,104,4,104,2,104,4,104,4,104,2,99";
        let ic = generator(program);
        assert_eq!(part1(&ic), 2);
    }

    #[test]
    pub fn part2_test() {
        // Construct a program that starts with a dummy opcode 2 instruction at address 0
        // (to handle set_values(&[(0, 2)]) safely), then outputs:
        // x=-1, y=0, score=12345 (segment display update)
        // then halts (99).
        // Instructions: 104, val
        let program = "2,4,4,3,104,-1,104,0,104,12345,99";
        let ic = generator(program);
        assert_eq!(part2(&ic), 12345);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day13.txt");
        const ANSWERS: (usize, i64) = (273, 13140);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
