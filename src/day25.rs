use aoc_harness::{aoc, aoc_generator};
use rustc_hash::FxHashSet as HashSet;

use crate::intcode::{IntCode, MachineState};

// These ship items kill or trap the droid when picked up.
const HAZARDS: [&str; 5] = [
    "escape pod",
    "giant electromagnet",
    "infinite loop",
    "molten lava",
    "photons",
];

#[aoc_generator(day25)]
pub fn generator(input: &str) -> IntCode<i64> {
    input.trim().parse().unwrap()
}

fn read_output(droid: &mut IntCode<i64>) -> String {
    while droid.run() == MachineState::HasOutput {}
    let mut bytes = Vec::new();
    while let Some(value) = droid.get_output() {
        bytes.push(u8::try_from(value).expect("invalid adventure output"));
    }
    String::from_utf8(bytes).expect("adventure output must be ASCII")
}

fn command(droid: &mut IntCode<i64>, instruction: &str) -> String {
    for byte in instruction.bytes().chain(std::iter::once(b'\n')) {
        droid.set_input(i64::from(byte));
    }
    read_output(droid)
}

fn password(transcript: &str) -> Option<u64> {
    let (_, suffix) = transcript.split_once("typing ")?;
    suffix.split_ascii_whitespace().next()?.parse().ok()
}

fn reverse(direction: &str) -> &'static str {
    match direction {
        "north" => "south",
        "south" => "north",
        "east" => "west",
        "west" => "east",
        _ => panic!("invalid direction: {direction}"),
    }
}

struct Room {
    name: String,
    doors: Vec<&'static str>,
    items: Vec<String>,
}

impl Room {
    fn parse(transcript: &str) -> Self {
        // An ejection describes the floor and then the checkpoint; the last
        // room is where the droid actually ends up.
        let (_, description) = transcript.rsplit_once("== ").expect("missing room");
        let (name, rest) = description.split_once(" ==").expect("missing room name");
        let mut room = Self {
            name: name.to_owned(),
            doors: Vec::new(),
            items: Vec::new(),
        };
        let mut section = "";
        for line in rest.lines() {
            match line {
                "Doors here lead:" => section = "doors",
                "Items here:" => section = "items",
                _ => {
                    if let Some(value) = line.strip_prefix("- ") {
                        match section {
                            "doors" => room.doors.push(match value {
                                "north" => "north",
                                "south" => "south",
                                "east" => "east",
                                "west" => "west",
                                _ => panic!("invalid direction: {value}"),
                            }),
                            "items" => room.items.push(value.to_owned()),
                            _ => {}
                        }
                    } else if line.is_empty() {
                        section = "";
                    }
                }
            }
        }
        room
    }
}

struct Adventure {
    droid: IntCode<i64>,
    visited: HashSet<String>,
    items: Vec<String>,
    checkpoint: Option<(Vec<&'static str>, &'static str)>,
}

impl Adventure {
    fn explore(&mut self, room: Room, path: &mut Vec<&'static str>) -> Option<u64> {
        if !self.visited.insert(room.name.clone()) {
            return None;
        }
        for item in &room.items {
            if !HAZARDS.contains(&item.as_str()) {
                let response = command(&mut self.droid, &format!("take {item}"));
                assert!(response.contains("You take the"), "{response}");
                self.items.push(item.clone());
            }
        }
        for direction in room.doors {
            let response = command(&mut self.droid, direction);
            if let Some(code) = password(&response) {
                return Some(code);
            }
            if response.contains("Alert!") {
                assert_eq!(room.name, "Security Checkpoint");
                self.checkpoint = Some((path.clone(), direction));
                continue;
            }
            path.push(direction);
            if let Some(code) = self.explore(Room::parse(&response), path) {
                return Some(code);
            }
            path.pop();
            let response = command(&mut self.droid, reverse(direction));
            assert_eq!(Room::parse(&response).name, room.name);
        }
        None
    }

    fn weigh_inventory(&mut self, direction: &str) -> u64 {
        let combinations = 1_usize
            .checked_shl(u32::try_from(self.items.len()).unwrap())
            .expect("too many inventory items");
        let full = combinations - 1;
        let mut previous = full;
        for attempt in 0..combinations {
            // Gray code changes just one carried item between floor attempts.
            // Start with the full inventory collected during exploration.
            let carried = full ^ (attempt ^ (attempt >> 1));
            let changed = previous ^ carried;
            if changed != 0 {
                let index = usize::try_from(changed.trailing_zeros()).unwrap();
                let action = if carried & changed == 0 {
                    "drop"
                } else {
                    "take"
                };
                let response = command(&mut self.droid, &format!("{action} {}", self.items[index]));
                assert!(
                    response.contains(&format!("You {action} the")),
                    "{response}"
                );
            }
            let response = command(&mut self.droid, direction);
            if let Some(code) = password(&response) {
                return code;
            }
            assert!(response.contains("Alert!"), "{response}");
            assert_eq!(Room::parse(&response).name, "Security Checkpoint");
            previous = carried;
        }
        panic!("no inventory combination opened the airlock");
    }
}

#[aoc(day25, part1)]
pub fn part1(program: &IntCode<i64>) -> u64 {
    let mut adventure = Adventure {
        droid: program.clone(),
        visited: HashSet::default(),
        items: Vec::new(),
        checkpoint: None,
    };
    let response = read_output(&mut adventure.droid);
    if let Some(code) = password(&response) {
        return code;
    }
    if let Some(code) = adventure.explore(Room::parse(&response), &mut Vec::new()) {
        return code;
    }
    let (path, direction) = adventure.checkpoint.take().expect("checkpoint not found");
    for step in path {
        command(&mut adventure.droid, step);
    }
    adventure.weigh_inventory(direction)
}

// The final star is awarded on the website once the other 49 are collected;
// there is no additional computation on the Intcode program.
#[aoc(day25, part2)]
pub fn part2(_program: &IntCode<i64>) -> &'static str {
    "Merry Christmas!"
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOM: &str = "\n== Hull Breach ==\nYou got in through a hole.\n\nDoors here lead:\n- north\n- east\n\nItems here:\n- festive hat\n\nCommand?\n";

    #[test]
    fn parses_room_exits_and_items() {
        let room = Room::parse(ROOM);
        assert_eq!(room.name, "Hull Breach");
        assert_eq!(room.doors, ["north", "east"]);
        assert_eq!(room.items, ["festive hat"]);
    }

    #[test]
    fn parses_the_room_after_a_checkpoint_rejection() {
        let transcript = format!("== Pressure-Sensitive Floor ==\nAlert! You are ejected.\n{ROOM}");
        assert_eq!(Room::parse(&transcript).name, "Hull Breach");
    }

    #[test]
    fn extracts_only_the_airlock_password() {
        assert_eq!(password(ROOM), None);
        assert_eq!(password("Droid 123 is too heavy."), None);
        assert_eq!(
            password("You should be able to get in by typing 123456 on the keypad."),
            Some(123456)
        );
    }

    fn ascii_program(output: &str, ending: &[i64]) -> IntCode<i64> {
        let mut code = Vec::new();
        for byte in output.bytes() {
            code.extend([104, i64::from(byte)]);
        }
        code.extend_from_slice(ending);
        generator(
            &code
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )
    }

    #[test]
    fn collects_complete_ascii_output_until_input_or_halt() {
        let mut waiting = ascii_program(ROOM, &[3, 1000, 99]);
        assert_eq!(read_output(&mut waiting), ROOM);
        assert_eq!(waiting.run(), MachineState::WaitingForInput);
        let mut halted = ascii_program("typing 42 on the keypad", &[99]);
        assert_eq!(read_output(&mut halted), "typing 42 on the keypad");
    }

    #[test]
    fn sends_commands_with_a_newline() {
        // Echo each byte, ending at the command's newline.
        let mut droid = generator("3,100,4,100,1008,100,10,101,1006,101,0,99");
        assert_eq!(command(&mut droid, "north"), "north\n");
    }

    #[test]
    fn accepts_success_without_a_room_or_another_prompt() {
        let program = ascii_program(
            "You should be able to get in by typing 123456 on the keypad.",
            &[99],
        );
        assert_eq!(part1(&program), 123456);
    }

    #[test]
    fn part_two_celebrates_the_completed_calendar() {
        assert_eq!(part2(&generator("99")), "Merry Christmas!");
    }

    #[test]
    fn explores_cycles_skips_hazards_and_tries_inventory_subsets() {
        let start = "== Hull Breach ==\n\nDoors here lead:\n- north\n- east\n\nItems here:\n- photons\n\nCommand?\n";
        let checkpoint =
            "== Security Checkpoint ==\n\nDoors here lead:\n- south\n- west\n\nCommand?\n";
        let storage = "== Storage ==\n\nDoors here lead:\n- west\n\nItems here:\n- coin\n- hat\n- infinite loop\n\nCommand?\n";
        let rejection =
            format!("== Pressure-Sensitive Floor ==\nAlert! You are ejected.\n{checkpoint}");
        let exchanges = [
            ("north", checkpoint),
            ("south", start),
            ("north", checkpoint),
            ("west", rejection.as_str()),
            ("south", start),
            ("east", storage),
            ("take coin", "You take the coin.\nCommand?\n"),
            ("take hat", "You take the hat.\nCommand?\n"),
            ("west", start),
            ("east", storage),
            ("west", start),
            ("north", checkpoint),
            ("west", rejection.as_str()),
            ("drop coin", "You drop the coin.\nCommand?\n"),
            ("west", rejection.as_str()),
            ("drop hat", "You drop the hat.\nCommand?\n"),
            ("west", rejection.as_str()),
            ("take coin", "You take the coin.\nCommand?\n"),
            (
                "west",
                "You should be able to get in by typing 42 on the keypad.",
            ),
        ];
        let mut code = Vec::new();
        for byte in start.bytes() {
            code.extend([104, i64::from(byte)]);
        }
        for (instruction, response) in exchanges {
            for byte in instruction.bytes().chain(std::iter::once(b'\n')) {
                // Halt immediately if the solver issues an unexpected command.
                code.extend([
                    3,
                    100_000,
                    1008,
                    100_000,
                    i64::from(byte),
                    100_001,
                    1005,
                    100_001,
                ]);
                code.push(i64::try_from(code.len() + 2).unwrap());
                code.push(99);
            }
            for byte in response.bytes() {
                code.extend([104, i64::from(byte)]);
            }
        }
        code.push(99);
        let program = generator(
            &code
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
        assert_eq!(part1(&program), 42);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day25.txt");
        const ANSWERS: (u64, &str) = (2_622_472, "Merry Christmas!");

        #[test]
        fn answers() {
            let program = generator(INPUT);
            assert_eq!(part1(&program), ANSWERS.0);
            assert_eq!(part2(&program), ANSWERS.1);
        }
    }
}
