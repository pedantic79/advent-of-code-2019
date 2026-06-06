use aoc_runner_derive::{aoc, aoc_generator};
use rustc_hash::FxHashMap as HashMap;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChemicalQty {
    pub name: String,
    pub qty: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reaction {
    pub inputs: Vec<ChemicalQty>,
    pub output: ChemicalQty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub reactions: Vec<Reaction>,
}

fn parse_chemical_qty(s: &str) -> ChemicalQty {
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();
    let qty = parts[0].parse::<usize>().unwrap();
    let name = parts[1].to_string();
    ChemicalQty { name, qty }
}

fn parse_reaction(line: &str) -> Reaction {
    let parts: Vec<&str> = line.split("=>").collect();
    let inputs_str = parts[0].trim();
    let output_str = parts[1].trim();

    let inputs = inputs_str.split(',').map(parse_chemical_qty).collect();
    let output = parse_chemical_qty(output_str);

    Reaction { inputs, output }
}

#[aoc_generator(day14)]
pub fn generator(input: &str) -> Input {
    let reactions = input
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(parse_reaction)
        .collect();
    Input { reactions }
}

pub fn ore_required(input: &Input, fuel_qty: usize) -> usize {
    let mut reactions_by_output = HashMap::default();
    for r in &input.reactions {
        reactions_by_output.insert(r.output.name.clone(), r.clone());
    }

    let mut in_degrees = HashMap::default();
    in_degrees.insert("FUEL".to_string(), 0);

    let mut queue = VecDeque::new();
    queue.push_back("FUEL".to_string());

    let mut visited = std::collections::HashSet::new();
    visited.insert("FUEL".to_string());

    while let Some(current) = queue.pop_front() {
        if current == "ORE" {
            continue;
        }
        if let Some(reaction) = reactions_by_output.get(&current) {
            for input_qty in &reaction.inputs {
                *in_degrees.entry(input_qty.name.clone()).or_insert(0) += 1;
                if visited.insert(input_qty.name.clone()) {
                    queue.push_back(input_qty.name.clone());
                }
            }
        }
    }

    let mut needed = HashMap::default();
    needed.insert("FUEL".to_string(), fuel_qty);

    let mut queue = VecDeque::new();
    queue.push_back("FUEL".to_string());

    while let Some(current) = queue.pop_front() {
        if current == "ORE" {
            continue;
        }

        let req = *needed.get(&current).unwrap_or(&0);
        if req == 0 {
            if let Some(reaction) = reactions_by_output.get(&current) {
                for input_qty in &reaction.inputs {
                    let deg = in_degrees.get_mut(&input_qty.name).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(input_qty.name.clone());
                    }
                }
            }
            continue;
        }

        let reaction = reactions_by_output
            .get(&current)
            .expect("No reaction found for chemical");
        let out_qty = reaction.output.qty;
        let k = req.div_ceil(out_qty);

        for input_qty in &reaction.inputs {
            *needed.entry(input_qty.name.clone()).or_insert(0) += k * input_qty.qty;
            let deg = in_degrees.get_mut(&input_qty.name).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push_back(input_qty.name.clone());
            }
        }
    }

    *needed.get("ORE").unwrap_or(&0)
}

#[aoc(day14, part1)]
pub fn part1(input: &Input) -> usize {
    ore_required(input, 1)
}

#[aoc(day14, part2)]
pub fn part2(input: &Input) -> usize {
    let mut low = 1;
    let mut high = 1_000_000_000_000;
    let mut ans = 0;

    while low <= high {
        let mid = low + (high - low) / 2;
        let ore = ore_required(input, mid);
        if ore <= 1_000_000_000_000 {
            ans = mid;
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    ans
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE1: &str = r"
10 ORE => 10 A
1 ORE => 1 B
7 A, 1 B => 1 C
7 A, 1 C => 1 D
7 A, 1 D => 1 E
7 A, 1 E => 1 FUEL
";

    const SAMPLE2: &str = r"
9 ORE => 2 A
8 ORE => 3 B
7 ORE => 5 C
3 A, 4 B => 1 AB
5 B, 7 C => 1 BC
4 C, 1 A => 1 CA
2 AB, 3 BC, 4 CA => 1 FUEL
";

    const SAMPLE3: &str = r"
157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT
";

    #[test]
    pub fn part1_sample1_test() {
        assert_eq!(part1(&generator(SAMPLE1)), 31);
    }

    #[test]
    pub fn part1_sample2_test() {
        assert_eq!(part1(&generator(SAMPLE2)), 165);
    }

    #[test]
    pub fn part1_sample3_test() {
        assert_eq!(part1(&generator(SAMPLE3)), 13312);
    }

    #[test]
    pub fn part2_sample3_test() {
        assert_eq!(part2(&generator(SAMPLE3)), 82892753);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day14.txt");
        const ANSWERS: (usize, usize) = (319014, 4076490);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
