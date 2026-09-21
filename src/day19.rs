use aoc_runner_derive::{aoc, aoc_generator};

use crate::intcode::{IntCode, MachineState};

const SCAN_SIZE: usize = 50;
const SHIP_SIZE: usize = 100;

#[aoc_generator(day19)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

fn is_affected(program: &IntCode<i64>, x: usize, y: usize) -> bool {
    let mut drone = program.clone();
    drone.set_input(i64::try_from(x).unwrap());
    drone.set_input(i64::try_from(y).unwrap());

    assert_eq!(drone.run(), MachineState::HasOutput);
    match drone.get_output().unwrap() {
        0 => false,
        1 => true,
        output => panic!("invalid drone output: {output}"),
    }
}

fn count_affected(size: usize, mut probe: impl FnMut(usize, usize) -> bool) -> usize {
    (0..size)
        .flat_map(|y| (0..size).map(move |x| (x, y)))
        .filter(|&(x, y)| probe(x, y))
        .count()
}

#[aoc(day19, part1)]
pub fn part1(program: &IntCode<i64>) -> usize {
    count_affected(SCAN_SIZE, |x, y| is_affected(program, x, y))
}

fn find_square(size: usize, mut probe: impl FnMut(usize, usize) -> bool) -> (usize, usize) {
    assert!(size != 0, "square size must be positive");

    let mut x = 0;
    for bottom_y in size - 1.. {
        while !probe(x, bottom_y) {
            x += 1;
        }

        let top_y = bottom_y + 1 - size;
        if probe(x + size - 1, top_y) {
            return (x, top_y);
        }
    }

    unreachable!("coordinate range exhausted")
}

#[aoc(day19, part2)]
pub fn part2(program: &IntCode<i64>) -> usize {
    let (x, y) = find_square(SHIP_SIZE, |x, y| is_affected(program, x, y));
    x * 10_000 + y
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: [&[u8]; 10] = [
        b"#.........",
        b".#........",
        b"..##......",
        b"...###....",
        b"....###...",
        b".....####.",
        b"......####",
        b"......####",
        b".......###",
        b"........##",
    ];

    const LARGE_SAMPLE: [&[u8]; 35] = [
        b"#.......................................",
        b".#......................................",
        b"..##....................................",
        b"...###..................................",
        b"....###.................................",
        b".....####...............................",
        b"......#####.............................",
        b"......######............................",
        b".......#######..........................",
        b"........########........................",
        b".........#########......................",
        b"..........#########.....................",
        b"...........##########...................",
        b"...........############.................",
        b"............############................",
        b".............#############..............",
        b"..............##############............",
        b"...............###############..........",
        b"................###############.........",
        b"................#################.......",
        b".................########OOOOOOOOOO.....",
        b"..................#######OOOOOOOOOO#....",
        b"...................######OOOOOOOOOO###..",
        b"....................#####OOOOOOOOOO#####",
        b".....................####OOOOOOOOOO#####",
        b".....................####OOOOOOOOOO#####",
        b"......................###OOOOOOOOOO#####",
        b".......................##OOOOOOOOOO#####",
        b"........................#OOOOOOOOOO#####",
        b".........................OOOOOOOOOO#####",
        b"..........................##############",
        b"..........................##############",
        b"...........................#############",
        b"............................############",
        b".............................###########",
    ];

    #[test]
    fn sample_has_27_affected_points() {
        assert_eq!(count_affected(10, |x, y| SAMPLE[y][x] == b'#'), 27);
    }

    #[test]
    fn part1_queries_x_then_y() {
        let program = generator("3,12,3,13,7,12,13,14,4,14,99,0,0,0,0");
        assert_eq!(part1(&program), 1_225);
    }

    #[test]
    fn sample_square_starts_at_25_20() {
        assert_eq!(
            find_square(10, |x, y| LARGE_SAMPLE
                .get(y)
                .and_then(|row| row.get(x))
                .is_some_and(|&cell| matches!(cell, b'#' | b'O'))),
            (25, 20)
        );
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day19.txt");
        const ANSWERS: (usize, usize) = (118, 18_651_593);

        #[test]
        fn answers() {
            let program = generator(INPUT.trim_end_matches('\n'));

            assert_eq!(part1(&program), ANSWERS.0);
            assert_eq!(part2(&program), ANSWERS.1);
        }
    }
}
