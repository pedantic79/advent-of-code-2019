use aoc_harness::{aoc, aoc_generator};
use itertools::Itertools;

use crate::intcode::{IntCode, MachineState};

#[aoc_generator(day7)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.parse().unwrap()
}

fn run_amplifier(
    program: &IntCode<i64>,
    amplifier: &mut IntCode<i64>,
    phase_setting: i64,
    initial_input: i64,
) -> i64 {
    amplifier.clone_from(program);
    amplifier.set_input(phase_setting);
    amplifier.set_input(initial_input);
    amplifier.run();
    amplifier.get_output().unwrap()
}

fn solve_part1_with_amplifier(
    ic: &IntCode<i64>,
    amplifier: &mut IntCode<i64>,
    phases: &[i64],
) -> i64 {
    phases.iter().fold(0, |initial_setting, &phase| {
        run_amplifier(ic, amplifier, phase, initial_setting)
    })
}

#[cfg(test)]
fn solve_part1(ic: &IntCode<i64>, phases: &[i64]) -> i64 {
    solve_part1_with_amplifier(ic, &mut ic.clone(), phases)
}

#[aoc(day7, part1)]
pub fn part1(ic: &IntCode<i64>) -> i64 {
    let mut amplifier = ic.clone();
    (0..5)
        .permutations(5)
        .map(|phases| solve_part1_with_amplifier(ic, &mut amplifier, &phases))
        .max()
        .unwrap()
}

fn solve_part2_with_amplifiers(
    ic: &IntCode<i64>,
    amps: &mut [IntCode<i64>],
    phases: &[i64],
) -> i64 {
    for (amp, &phase) in amps.iter_mut().zip(phases) {
        amp.clone_from(ic);
        amp.set_input(phase);
    }

    let mut input = 0;
    for i in (0..5).cycle() {
        let (state, output) = amps[i].run_single_io(input);
        if state == MachineState::Halted {
            break;
        }
        input = output.unwrap();
    }

    input
}

#[cfg(test)]
fn solve_part2(ic: &IntCode<i64>, phases: &[i64]) -> i64 {
    solve_part2_with_amplifiers(ic, &mut vec![ic.clone(); 5], phases)
}

#[aoc(day7, part2)]
pub fn part2(ic: &IntCode<i64>) -> i64 {
    let mut amplifiers = vec![ic.clone(); 5];
    (5..10)
        .permutations(5)
        .map(|phases| solve_part2_with_amplifiers(ic, &mut amplifiers, &phases))
        .max()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0";

    #[test]
    pub fn input_test() {
        println!("{:?}", generator(SAMPLE));

        // assert_eq!(generator(SAMPLE), Input());
    }

    #[test]
    pub fn part1_test() {
        assert_eq!(
            solve_part1(
                &generator("3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0"),
                &[4, 3, 2, 1, 0]
            ),
            43210
        );

        assert_eq!(
            solve_part1(
                &generator(
                    "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0"
                ),
                &[0, 1, 2, 3, 4]
            ),
            54321
        );

        assert_eq!(
            solve_part1(
                &generator(
                    "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0"
                ),
                &[1, 0, 4, 3, 2]
            ),
            65210
        );
    }

    #[test]
    pub fn part2_test() {
        assert_eq!(
            solve_part2(
                &generator(
                    "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5"
                ),
                &[9, 8, 7, 6, 5]
            ),
            139629729
        );

        assert_eq!(
            solve_part2(
                &generator(
                    "3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10"
                ),
                &[9, 7, 8, 5, 6]
            ),
            18216
        );
    }

    mod regression {
        use super::*;

        const ANSWERS: (i64, i64) = (255590, 58285150);

        #[test]
        pub fn test() {
            let output = generator(&aoc_harness::input!(7));

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
