use aoc_harness::{aoc, aoc_generator};
use arrayvec::ArrayVec;
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
    output: ArrayVec<i64, 3>,
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
            output: ArrayVec::new(),
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

    #[test]
    fn queued_packets_preserve_fifo_order_and_negative_x_values() {
        let program = generator("3,100,3,101,3,102,104,255,4,101,4,102,1105,1,2");
        let mut nic = Nic::new(&program, 0);
        nic.packets.extend([(-1, 7), (8, 9)]);
        let mut supplied_empty_read = false;
        let mut activity = false;

        for (expected_x, expected_y) in [(-1, 7), (8, 9)] {
            assert!(matches!(
                nic.run_until_event(&mut supplied_empty_read, &mut activity),
                NicEvent::Packet(NAT_ADDRESS, x, y) if (x, y) == (expected_x, expected_y)
            ));
        }
        assert!(nic.packets.is_empty());
        assert!(activity);
    }

    #[test]
    fn partial_packet_survives_an_empty_read_and_a_scheduler_yield() {
        let program = generator("3,100,104,255,3,101,104,12,3,102,104,34,99");
        let mut nic = Nic::new(&program, 0);
        let mut supplied_empty_read = false;
        let mut activity = false;
        assert!(matches!(
            nic.run_until_event(&mut supplied_empty_read, &mut activity),
            NicEvent::Idle
        ));
        assert!(activity);

        supplied_empty_read = false;
        activity = false;
        assert!(matches!(
            nic.run_until_event(&mut supplied_empty_read, &mut activity),
            NicEvent::Packet(NAT_ADDRESS, 12, 34)
        ));
        assert!(nic.output.is_empty());
    }

    #[test]
    fn routes_packets_to_nics_before_and_after_the_sender() {
        for (sender, recipient) in [(0, 49), (49, 0)] {
            // The sender emits one packet; the recipient forwards it to the NAT.
            // Other NICs continually read their empty queues.
            let program = generator(&format!(
                "3,100,1008,100,{sender},101,1005,101,21,1008,100,{recipient},101,1005,101,30,3,102,1105,1,16,104,{recipient},104,1234,104,5678,1105,1,16,3,102,1008,102,-1,101,1005,101,30,3,103,104,255,4,102,4,103,1105,1,30"
            ));
            assert_eq!(part1(&program), 5678);
            assert_eq!(part2(&program), 5678);
        }
    }

    #[test]
    fn nat_ignores_nonconsecutive_repeated_y_values() {
        // NIC 0 announces Y=10, then changes the NAT to 20, 10, and 30 after
        // successive deliveries. Only 30 is delivered twice consecutively.
        let program = generator(
            "3,100,1008,100,0,101,1006,101,15,104,255,104,0,104,10,3,102,1008,102,-1,101,1005,101,15,3,103,1001,104,1,104,1008,104,1,101,1005,101,54,1008,104,2,101,1005,101,63,1008,104,3,101,1005,101,72,1105,1,15,104,255,104,0,104,20,1105,1,15,104,255,104,0,104,10,1105,1,15,104,255,104,0,104,30,1105,1,15",
        );
        assert_eq!(part2(&program), 30);
    }

    mod regression {
        use super::*;

        const ANSWERS: (i64, i64) = (20_225, 14_348);

        #[test]
        fn answers() {
            let program = generator(&aoc_harness::input!(23));

            assert_eq!(part1(&program), ANSWERS.0);
            assert_eq!(part2(&program), ANSWERS.1);
        }
    }
}
