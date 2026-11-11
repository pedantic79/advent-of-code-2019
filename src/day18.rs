use std::collections::VecDeque;

use aoc_harness::{aoc, aoc_generator};
use pathfinding::prelude::{astar, dijkstra};

pub struct Vault {
    cells: Vec<u8>,
    width: usize,
    entrances: Vec<usize>,
    keys: [Option<usize>; 26],
    all_keys: u32,
}

#[aoc_generator(day18)]
pub fn generator(input: &str) -> Vault {
    let rows: Vec<_> = input.lines().map(str::as_bytes).collect();
    let width = rows.first().expect("vault must not be empty").len();
    assert!(rows.iter().all(|row| row.len() == width));

    let cells: Vec<u8> = rows.into_iter().flatten().copied().collect();
    let mut entrances = Vec::new();
    let mut keys = [None; 26];
    let mut all_keys = 0;

    for (position, &cell) in cells.iter().enumerate() {
        match cell {
            b'@' => entrances.push(position),
            b'a'..=b'z' => {
                let key = usize::from(cell - b'a');
                keys[key] = Some(position);
                all_keys |= 1 << key;
            }
            _ => {}
        }
    }

    assert!(!entrances.is_empty(), "vault must have an entrance");
    Vault {
        cells,
        width,
        entrances,
        keys,
        all_keys,
    }
}

#[aoc(day18, part1)]
pub fn part1(vault: &Vault) -> usize {
    assert_eq!(vault.entrances.len(), 1);
    solve(vault, [vault.entrances[0]])
}

#[aoc(day18, part2)]
pub fn part2(mut vault: Vault) -> usize {
    if vault.entrances.len() == 1 {
        let center = vault.entrances[0];
        let width = vault.width;
        let walls = [
            center,
            center - width,
            center + width,
            center - 1,
            center + 1,
        ];
        for position in walls {
            vault.cells[position] = b'#';
        }
        let entrances = [
            center - width - 1,
            center - width + 1,
            center + width - 1,
            center + width + 1,
        ];
        for position in entrances {
            vault.cells[position] = b'@';
        }
        vault.entrances = entrances.to_vec();
    }

    let entrances: [usize; 4] = vault
        .entrances
        .as_slice()
        .try_into()
        .expect("part 2 needs four entrances");
    solve(&vault, entrances)
}

fn solve<const ROBOTS: usize>(vault: &Vault, entrances: [usize; ROBOTS]) -> usize {
    let graph = &build_graph(vault, &entrances);
    let initial_positions: [u8; ROBOTS] = std::array::from_fn(|index| u8::try_from(index).unwrap());

    let result = if ROBOTS == 1 {
        dijkstra(
            &(initial_positions, 0_u32),
            |&(positions, keys)| {
                (0..ROBOTS).flat_map(move |robot| {
                    graph[usize::from(positions[robot])]
                        .iter()
                        .filter_map(move |edge| {
                            if edge.required & !keys != 0 {
                                return None;
                            }
                            let mut next_positions = positions;
                            next_positions[robot] = edge.target;
                            Some(((next_positions, keys | (1 << edge.key)), edge.distance))
                        })
                })
            },
            |&(_, keys)| keys == vault.all_keys,
        )
    } else {
        let relaxed = relaxed_distances(graph);
        astar(
            &(initial_positions, 0_u32),
            |&(positions, keys)| successors(graph, positions, keys),
            |&(positions, keys)| {
                remaining_distance_lower_bound(
                    &relaxed,
                    entrances.len(),
                    vault.all_keys,
                    positions,
                    keys,
                )
            },
            |&(_, keys)| keys == vault.all_keys,
        )
    };

    result.expect("not all keys are reachable").1
}

#[inline]
fn successors<const ROBOTS: usize>(
    graph: &[Vec<Edge>],
    positions: [u8; ROBOTS],
    keys: u32,
) -> impl Iterator<Item = (([u8; ROBOTS], u32), usize)> + '_ {
    (0..ROBOTS).flat_map(move |robot| {
        graph[usize::from(positions[robot])]
            .iter()
            .filter_map(move |edge| {
                if edge.required & !keys != 0 {
                    return None;
                }
                let mut next_positions = positions;
                next_positions[robot] = edge.target;
                Some(((next_positions, keys | (1 << edge.key)), edge.distance))
            })
    })
}

fn relaxed_distances(graph: &[Vec<Edge>]) -> Vec<Vec<usize>> {
    let mut distances = vec![vec![usize::MAX; graph.len()]; graph.len()];
    for (source, edges) in graph.iter().enumerate() {
        distances[source][source] = 0;
        for edge in edges {
            let target = usize::from(edge.target);
            distances[source][target] = distances[source][target].min(edge.distance);
        }
    }

    for middle in 0..graph.len() {
        for source in 0..graph.len() {
            if distances[source][middle] == usize::MAX {
                continue;
            }
            for target in 0..graph.len() {
                let through_middle =
                    distances[source][middle].saturating_add(distances[middle][target]);
                distances[source][target] = distances[source][target].min(through_middle);
            }
        }
    }
    distances
}

// Ignore doors to get an admissible lower bound. Work assigned to robots in
// disconnected regions can be added because the final distance sums their moves.
#[inline]
fn remaining_distance_lower_bound<const ROBOTS: usize>(
    relaxed: &[Vec<usize>],
    entrance_count: usize,
    all_keys: u32,
    positions: [u8; ROBOTS],
    keys: u32,
) -> usize {
    let mut unique_robot_work = [0; ROBOTS];
    let mut farthest_key = 0;

    for key in 0..26 {
        if all_keys & !keys & (1 << key) == 0 {
            continue;
        }

        let target = entrance_count + key;
        let mut nearest = usize::MAX;
        let mut only_robot = None;
        let mut reachable_robots = 0;
        for (robot, &position) in positions.iter().enumerate() {
            let distance = relaxed[usize::from(position)][target];
            if distance != usize::MAX {
                nearest = nearest.min(distance);
                only_robot = Some(robot);
                reachable_robots += 1;
            }
        }

        // An unreachable state cannot contribute a useful finite estimate.
        if nearest == usize::MAX {
            return 0;
        }
        farthest_key = farthest_key.max(nearest);
        if reachable_robots == 1 {
            let robot = only_robot.expect("a reachable key has a reachable robot");
            unique_robot_work[robot] = unique_robot_work[robot].max(nearest);
        }
    }

    unique_robot_work
        .into_iter()
        .sum::<usize>()
        .max(farthest_key)
}

#[derive(Clone, Copy)]
struct Edge {
    target: u8,
    key: u8,
    distance: usize,
    required: u32,
}

fn build_graph(vault: &Vault, entrances: &[usize]) -> Vec<Vec<Edge>> {
    let mut graph = vec![Vec::new(); entrances.len() + 26];
    for (index, &entrance) in entrances.iter().enumerate() {
        graph[index] = routes_from(vault, entrance, None, entrances.len());
    }
    for (key, position) in vault.keys.iter().enumerate() {
        if let Some(position) = position {
            graph[entrances.len() + key] =
                routes_from(vault, *position, Some(key), entrances.len());
        }
    }
    graph
}

// Keep both a short route through a door and a longer route around it.
fn add_route(routes: &mut Vec<(usize, u32)>, distance: usize, required: u32) -> bool {
    if routes.iter().any(|&(old_distance, old_required)| {
        old_distance <= distance && old_required & required == old_required
    }) {
        return false;
    }
    routes.retain(|&(old_distance, old_required)| {
        distance > old_distance || required & old_required != required
    });
    routes.push((distance, required));
    true
}

fn routes_from(
    vault: &Vault,
    start: usize,
    source_key: Option<usize>,
    entrance_count: usize,
) -> Vec<Edge> {
    let mut visited = vec![Vec::new(); vault.cells.len()];
    let mut destinations = vec![Vec::new(); 26];
    let mut queue = VecDeque::new();
    add_route(&mut visited[start], 0, 0);
    queue.push_back((start, 0, 0));

    while let Some((position, distance, required)) = queue.pop_front() {
        if !visited[position].contains(&(distance, required)) {
            continue;
        }

        let column = position % vault.width;
        let neighbors = [
            position.checked_sub(vault.width),
            (position + vault.width < vault.cells.len()).then_some(position + vault.width),
            (column > 0).then(|| position - 1),
            (column + 1 < vault.width).then_some(position + 1),
        ];

        for next in neighbors.into_iter().flatten() {
            let cell = vault.cells[next];
            if cell == b'#' {
                continue;
            }
            let next_required = if cell.is_ascii_uppercase() {
                required | (1 << (cell - b'A'))
            } else {
                required
            };
            let next_distance = distance + 1;

            if cell.is_ascii_lowercase() && Some(usize::from(cell - b'a')) != source_key {
                add_route(
                    &mut destinations[usize::from(cell - b'a')],
                    next_distance,
                    next_required,
                );
            } else if add_route(&mut visited[next], next_distance, next_required) {
                queue.push_back((next, next_distance, next_required));
            }
        }
    }

    destinations
        .into_iter()
        .enumerate()
        .flat_map(|(key, routes)| {
            routes.into_iter().map(move |(distance, required)| Edge {
                target: u8::try_from(entrance_count + key).unwrap(),
                key: u8::try_from(key).unwrap(),
                distance,
                required,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_examples() {
        let examples = [
            ("#########\n#b.A.@.a#\n#########", 8),
            (
                "########################\n#f.D.E.e.C.b.A.@.a.B.c.#\n######################.#\n#d.....................#\n########################",
                86,
            ),
            (
                "########################\n#...............b.C.D.f#\n#.######################\n#.....@.a.B.c.d.A.e.F.g#\n########################",
                132,
            ),
            (
                "#################\n#i.G..c...e..H.p#\n########.########\n#j.A..b...f..D.o#\n########@########\n#k.E..a...g..B.n#\n########.########\n#l.F..d...h..C.m#\n#################",
                136,
            ),
            (
                "########################\n#@..............ac.GI.b#\n###d#e#f################\n###A#B#C################\n###g#h#i################\n########################",
                81,
            ),
        ];

        for (map, expected) in examples {
            assert_eq!(part1(&generator(map)), expected, "map:\n{map}");
        }
    }

    #[test]
    fn part2_examples() {
        let examples = [
            (
                "#######\n#a.#Cd#\n##...##\n##.@.##\n##...##\n#cB#Ab#\n#######",
                8,
            ),
            (
                "###############\n#d.ABC.#.....a#\n######@#@######\n###############\n######@#@######\n#b.....#.....c#\n###############",
                24,
            ),
            (
                "#############\n#DcBa.#.GhKl#\n#.###@#@#I###\n#e#d#####j#k#\n###C#@#@###J#\n#fEbA.#.FgHi#\n#############",
                32,
            ),
            (
                "#############\n#g#f.D#..h#l#\n#F###e#E###.#\n#dCba@#@BcIJ#\n#############\n#nK.L@#@G...#\n#M###N#H###.#\n#o#m..#i#jk.#\n#############",
                72,
            ),
        ];

        for (map, expected) in examples {
            assert_eq!(part2(generator(map)), expected, "map:\n{map}");
        }
    }

    #[test]
    fn keeps_longer_route_around_locked_door() {
        let map = "###########\n#@..A....b#\n#.#######.#\n#.........#\n###########";
        assert_eq!(part1(&generator(map)), 12);
    }

    #[test]
    fn can_pass_through_an_already_collected_key() {
        let map = "#############\n#b.C.@.a.c..#\n#############";
        assert_eq!(part1(&generator(map)), 12);
    }

    #[test]
    fn resolves_dependencies_between_disconnected_regions() {
        let map = "#############\n#@.a#b.A@..##\n#############\n#@.c#d.C@..##\n#############";
        assert_eq!(part2(generator(map)), 10);
    }

    #[test]
    fn supports_four_robots_in_one_region() {
        let map = "###############\n#@.@.a.b.@.@..#\n###############";
        assert_eq!(part2(generator(map)), 4);
    }

    mod regression {
        use super::*;

        const ANSWERS: (usize, usize) = (3962, 1844);

        #[test]
        fn test() {
            let vault = generator(&aoc_harness::input!(18));

            assert_eq!(part1(&vault), ANSWERS.0);
            assert_eq!(part2(vault), ANSWERS.1);
        }
    }
}
