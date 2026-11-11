use crate::common::nom::nom_usize;
use crate::common::sstr::SStr;
use aoc_harness::{aoc, aoc_generator};
use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::complete::{alpha1, char, line_ending, space0, space1},
    multi::separated_list1,
    sequence::separated_pair,
};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

type Chemical = (SStr<4>, usize);
type RawReaction = (SStr<4>, usize, Vec<Chemical>);

fn parse_chemical(input: &str) -> IResult<&str, Chemical> {
    let (input, (qty, name)) = separated_pair(nom_usize, space1, alpha1).parse(input)?;
    Ok((input, (SStr::from(name), qty)))
}

fn parse_line(input: &str) -> IResult<&str, RawReaction> {
    let (input, inputs) =
        separated_list1((space0, char(','), space0), parse_chemical).parse(input)?;
    let (input, _) = (space0, tag("=>"), space0).parse(input)?;
    let (input, (out_name, out_qty)) = parse_chemical.parse(input)?;
    Ok((input, (out_name, out_qty, inputs)))
}

fn parse_raw_reactions(input: &str) -> Vec<RawReaction> {
    let (_, parsed) = separated_list1(line_ending, parse_line)
        .parse(input.trim())
        .unwrap();
    parsed
}

fn build_indexed_reactions(parsed_reactions: Vec<RawReaction>) -> (Vec<ReactionIndexed>, usize) {
    let mut chem_names = vec![SStr::from("FUEL"), SStr::from("ORE")];

    // Helper closure to retrieve the index of a chemical by its name.
    // If the chemical name is already registered in `chem_names`, returns its existing index.
    // Otherwise, registers the chemical by appending it to `chem_names` and returns its new index.
    let get_index = |name: &SStr<4>, chem_names: &mut Vec<SStr<4>>| -> usize {
        if let Some(pos) = chem_names.iter().position(|x| x == name) {
            pos
        } else {
            chem_names.push(*name);
            chem_names.len() - 1
        }
    };

    let mut reactions = vec![ReactionIndexed::default(); chem_names.len()];

    // Translate raw text-based reactions into an index-based graph representation
    // where each reaction is stored at the index of its output chemical.
    for (out_name, out_qty, inputs) in parsed_reactions {
        let out_idx = get_index(&out_name, &mut chem_names);

        if out_idx >= reactions.len() {
            reactions.resize(out_idx + 1, ReactionIndexed::default());
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

    // For valid inputs, every chemical (except ORE) has a producing reaction. Since ORE
    // (index 1) is allocated in the initial reactions vector, reactions.len() will already
    // equal chem_names.len() at the end of the loop, making this resize a no-op.
    // However, if the input is malformed and contains a chemical that is used as an
    // input but never produced as an output, this ensures reactions is resized to match.
    reactions.resize(chem_names.len(), ReactionIndexed::default());

    (reactions, chem_names.len())
}

/// Calculates the in-degree (dependency count) for each chemical
/// in the reaction network, starting from FUEL (index 0).
///
/// This does a breadth-first search (BFS) traversal of the reaction dependency
/// DAG to count how many reactions directly produce ingredients needed for
/// other reactions.
fn compute_in_degrees(reactions: &[ReactionIndexed], total_chems: usize) -> Vec<usize> {
    let mut in_degrees = vec![0; total_chems];
    let mut visited = vec![false; total_chems];
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
    in_degrees
}

/// Computes a topological ordering of the chemical reactions using Kahn's algorithm.
///
/// The resulting ordering guarantees that any chemical is processed before
/// its dependencies (the inputs needed to make it), starting from FUEL (index 0)
/// and ending with ORE (index 1).
fn compute_topo_order(reactions: &[ReactionIndexed], mut in_degrees: Vec<usize>) -> Vec<usize> {
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
    topo_order
}

#[aoc_generator(day14)]
pub fn generator(input: &str) -> Input {
    let parsed_reactions = parse_raw_reactions(input);
    let (reactions, total_chems) = build_indexed_reactions(parsed_reactions);
    let in_degrees = compute_in_degrees(&reactions, total_chems);
    let topo_order = compute_topo_order(&reactions, in_degrees);

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
        let mid = usize::midpoint(low, high);
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

        const ANSWERS: (usize, usize) = (319014, 4076490);

        #[test]
        pub fn test() {
            let output = generator(&aoc_harness::input!(14));

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
