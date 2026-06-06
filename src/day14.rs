use aoc_runner_derive::{aoc, aoc_generator};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionIndexed {
    pub out_qty: usize,
    pub inputs: Vec<(usize, usize)>, // (chem_idx, qty)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub reactions: Vec<ReactionIndexed>,
    pub topo_order: Vec<usize>,
    pub ore_idx: usize,
}

#[aoc_generator(day14)]
pub fn generator(input: &str) -> Input {
    let mut parsed_reactions = Vec::new();
    let mut chem_names = Vec::new();

    // Map names to indices: FUEL -> 0, ORE -> 1
    chem_names.push("FUEL".to_string());
    chem_names.push("ORE".to_string());

    let get_index = |name: &str, chem_names: &mut Vec<String>| -> usize {
        if let Some(pos) = chem_names.iter().position(|x| x == name) {
            pos
        } else {
            chem_names.push(name.to_string());
            chem_names.len() - 1
        }
    };

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split("=>").collect();
        let inputs_str = parts[0].trim();
        let output_str = parts[1].trim();

        // Parse output
        let out_parts: Vec<&str> = output_str.split_whitespace().collect();
        let out_qty = out_parts[0].parse::<usize>().unwrap();
        let out_name = out_parts[1];

        // Parse inputs
        let mut inputs = Vec::new();
        for input_item in inputs_str.split(',') {
            let in_parts: Vec<&str> = input_item.split_whitespace().collect();
            let in_qty = in_parts[0].parse::<usize>().unwrap();
            let in_name = in_parts[1];
            inputs.push((in_name.to_string(), in_qty));
        }

        parsed_reactions.push((out_name.to_string(), out_qty, inputs));
    }

    let mut reactions = vec![ReactionIndexed { out_qty: 0, inputs: Vec::new() }; chem_names.len()];

    for (out_name, out_qty, inputs) in parsed_reactions {
        let out_idx = get_index(&out_name, &mut chem_names);
        
        if out_idx >= reactions.len() {
            reactions.resize(out_idx + 1, ReactionIndexed { out_qty: 0, inputs: Vec::new() });
        }

        let mut inputs_indexed = Vec::with_capacity(inputs.len());
        for (in_name, in_qty) in inputs {
            let in_idx = get_index(&in_name, &mut chem_names);
            inputs_indexed.push((in_idx, in_qty));
        }

        reactions[out_idx] = ReactionIndexed {
            out_qty,
            inputs: inputs_indexed,
        };
    }

    reactions.resize(chem_names.len(), ReactionIndexed { out_qty: 0, inputs: Vec::new() });

    let n = chem_names.len();
    let mut in_degrees = vec![0; n];
    let mut visited = vec![false; n];
    visited[0] = true;

    let mut queue = VecDeque::new();
    queue.push_back(0);

    while let Some(current) = queue.pop_front() {
        if current == 1 {
            continue;
        }
        let reaction = &reactions[current];
        for &(in_idx, _) in &reaction.inputs {
            in_degrees[in_idx] += 1;
            if !visited[in_idx] {
                visited[in_idx] = true;
                queue.push_back(in_idx);
            }
        }
    }

    let mut topo_order = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(0);

    while let Some(current) = queue.pop_front() {
        topo_order.push(current);
        if current == 1 {
            continue;
        }
        let reaction = &reactions[current];
        for &(in_idx, _) in &reaction.inputs {
            in_degrees[in_idx] -= 1;
            if in_degrees[in_idx] == 0 {
                queue.push_back(in_idx);
            }
        }
    }

    Input {
        reactions,
        topo_order,
        ore_idx: 1,
    }
}

pub fn ore_required(input: &Input, fuel_qty: usize) -> usize {
    let mut needed = vec![0; input.reactions.len()];
    needed[0] = fuel_qty;

    for &chem_idx in &input.topo_order {
        if chem_idx == input.ore_idx {
            continue;
        }
        let req = needed[chem_idx];
        if req == 0 {
            continue;
        }
        let reaction = &input.reactions[chem_idx];
        let k = req.div_ceil(reaction.out_qty);
        for &(in_idx, in_qty) in &reaction.inputs {
            needed[in_idx] += k * in_qty;
        }
    }

    needed[input.ore_idx]
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
