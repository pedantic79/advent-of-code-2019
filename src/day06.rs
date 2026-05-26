use ahash::HashMap;
use aoc_runner_derive::{aoc, aoc_generator};
use petgraph::{
    Graph,
    algo::dijkstra,
    graph::{NodeIndex, UnGraph},
};

use crate::common::{parse::parse_split_once, sstr::SStr};

#[aoc_generator(day6)]
pub fn generator(input: &str) -> (Graph<SStr<3>, ()>, Option<NodeIndex>, Option<NodeIndex>) {
    let mut nodes = HashMap::default();
    let mut graph = Graph::default();

    for (f, t) in input
        .lines()
        .map(|line| parse_split_once(line, ')').unwrap())
    {
        let f_node = *nodes.entry(f).or_insert_with(|| graph.add_node(f));
        let t_node = *nodes.entry(t).or_insert_with(|| graph.add_node(t));

        graph.add_edge(f_node, t_node, ());
    }

    (
        graph,
        nodes.get(&"YOU".parse().unwrap()).copied(),
        nodes.get(&"SAN".parse().unwrap()).copied(),
    )
}

fn count_orbits(node: NodeIndex, graph: &Graph<SStr<3>, ()>) -> usize {
    graph
        .neighbors_directed(node, petgraph::Direction::Incoming)
        .map(|parent| 1 + count_orbits(parent, graph))
        .sum()
}

#[aoc(day6, part1)]
pub fn part1(input: &(Graph<SStr<3>, ()>, Option<NodeIndex>, Option<NodeIndex>)) -> usize {
    let graph = &input.0;
    graph
        .node_indices()
        .map(|node| count_orbits(node, graph))
        .sum()
}

#[aoc(day6, part2)]
pub fn part2(
    (graph, you, san): &(Graph<SStr<3>, ()>, Option<NodeIndex>, Option<NodeIndex>),
) -> usize {
    let graph: UnGraph<SStr<3>, ()> = graph.clone().into_edge_type();
    let paths = dijkstra(&graph, you.unwrap(), *san, |_| 1);
    paths.get(&san.unwrap()).copied().unwrap() - 2
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"COM)BBB
BBB)CCC
CCC)DDD
DDD)EEE
EEE)FFF
BBB)GGG
GGG)HHH
DDD)III
EEE)JJJ
JJJ)KKK
KKK)LLL";

    const SAMPLE2: &str = r"COM)BBB
BBB)CCC
CCC)DDD
DDD)EEE
EEE)FFF
BBB)GGG
GGG)HHH
DDD)III
EEE)JJJ
JJJ)KKK
KKK)LLL
KKK)YOU
III)SAN";

    #[test]
    pub fn input_test() {
        println!("{:?}", generator(SAMPLE));

        // assert_eq!(generator(SAMPLE), Input());
    }

    #[test]
    pub fn part1_test() {
        assert_eq!(part1(&generator(SAMPLE)), 42);
    }

    #[test]
    pub fn part2_test() {
        assert_eq!(part2(&generator(SAMPLE2)), 4);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day6.txt");
        const ANSWERS: (usize, usize) = (271151, 388);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
