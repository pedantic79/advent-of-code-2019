use aoc_runner_derive::{aoc, aoc_generator};
use std::collections::VecDeque;

use crate::intcode::{IntCode, MachineState};

#[aoc_generator(day23)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.trim().parse().unwrap()
}

const NIC_COUNT: usize = 50;
const NAT_ADDRESS: i64 = 255;

struct Nic {
    computer: IntCode<i64>,
    packets: VecDeque<(i64, i64)>,
    output: Vec<i64>,
    halted: bool,
}

enum NicEvent {
    Packet(i64, i64, i64),
    Idle,
    Halted,
}

impl Nic {
    fn new(program: &IntCode<i64>, address: usize) -> Self {
        let mut computer = program.clone();
        computer.set_input(i64::try_from(address).unwrap());
        Self {
            computer,
            packets: VecDeque::new(),
            output: Vec::with_capacity(3),
            halted: false,
        }
    }

    // Run until this NIC has consumed an empty read and reaches its next input.
    // Packets are kept as pairs here and expanded into X,Y only when requested.
    fn run_until_event(&mut self, supplied_empty_read: &mut bool, activity: &mut bool) -> NicEvent {
        if self.halted {
            return NicEvent::Halted;
        }

        loop {
            match self.computer.run() {
                MachineState::HasOutput => {
                    *activity = true;
                    self.output.push(self.computer.get_output().unwrap());
                    if self.output.len() == 3 {
                        let destination = self.output[0];
                        let x = self.output[1];
                        let y = self.output[2];
                        self.output.clear();
                        return NicEvent::Packet(destination, x, y);
                    }
                }
                MachineState::WaitingForInput => {
                    if let Some((x, y)) = self.packets.pop_front() {
                        self.computer.set_input(x);
                        self.computer.set_input(y);
                        *activity = true;
                        *supplied_empty_read = false;
                    } else if *supplied_empty_read {
                        return NicEvent::Idle;
                    } else {
                        self.computer.set_input(-1);
                        *supplied_empty_read = true;
                    }
                }
                MachineState::Halted => {
                    self.halted = true;
                    return NicEvent::Halted;
                }
            }
        }
    }
}

fn run_network(program: &IntCode<i64>, stop_on_repeated_nat_y: bool) -> i64 {
    let mut nics: Vec<_> = (0..NIC_COUNT)
        .map(|address| Nic::new(program, address))
        .collect();
    let mut nat = None;
    let mut last_nat_y = None;

    loop {
        let mut all_idle = true;

        for address in 0..nics.len() {
            let mut supplied_empty_read = false;
            let mut activity = false;

            loop {
                match nics[address].run_until_event(&mut supplied_empty_read, &mut activity) {
                    NicEvent::Packet(destination, x, y) => match destination {
                        NAT_ADDRESS => {
                            if !stop_on_repeated_nat_y {
                                return y;
                            }
                            nat = Some((x, y));
                        }
                        0..=49 => {
                            nics[usize::try_from(destination).unwrap()]
                                .packets
                                .push_back((x, y));
                        }
                        _ => panic!("invalid network destination: {destination}"),
                    },
                    NicEvent::Idle => {
                        all_idle &= !activity;
                        break;
                    }
                    NicEvent::Halted => {
                        all_idle = false;
                        break;
                    }
                }
            }
        }

        if all_idle
            && nics.iter().all(|nic| nic.packets.is_empty())
            && let Some((x, y)) = nat
        {
            nics[0].packets.push_back((x, y));
            if stop_on_repeated_nat_y && last_nat_y == Some(y) {
                return y;
            }
            last_nat_y = Some(y);
        }
    }
}

#[aoc(day23, part1)]
pub fn part1(program: &IntCode<i64>) -> i64 {
    run_network(program, false)
}

#[aoc(day23, part2)]
pub fn part2(program: &IntCode<i64>) -> i64 {
    run_network(program, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each NIC announces (address, address + 7) to the NAT, then consumes
    // packet pairs until it receives an X value of -1.
    const NAT_ANNOUNCER: &str = "3,100,104,255,4,100,1001,100,7,101,4,101,3,102,1008,102,-1,103,1005,103,12,3,104,1105,1,12";

    #[test]
    fn first_packet_to_255_returns_its_y_value() {
        let program = generator(NAT_ANNOUNCER);
        assert_eq!(part1(&program), 7);
    }

    #[test]
    fn nat_repeats_the_y_value_after_consecutive_idle_deliveries() {
        let program = generator(NAT_ANNOUNCER);
        assert_eq!(part2(&program), 56);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day23.txt");
        const ANSWERS: (i64, i64) = (20_225, 14_348);

        #[test]
        fn answers() {
            let program = generator(INPUT);

            assert_eq!(part1(&program), ANSWERS.0);
            assert_eq!(part2(&program), ANSWERS.1);
        }
    }
}
