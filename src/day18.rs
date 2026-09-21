use std::collections::VecDeque;

use aoc_runner_derive::{aoc, aoc_generator};
use pathfinding::prelude::dijkstra;

#[derive(Clone)]
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
pub fn part2(vault: &Vault) -> usize {
    let mut vault = vault.clone();
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
        .clone()
        .try_into()
        .expect("part 2 needs four entrances");
    solve(&vault, entrances)
}

fn solve<const ROBOTS: usize>(vault: &Vault, entrances: [usize; ROBOTS]) -> usize {
    let graph = &build_graph(vault, &entrances);
    let initial_positions: [u8; ROBOTS] = std::array::from_fn(|index| u8::try_from(index).unwrap());

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
    .expect("not all keys are reachable")
    .1
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
            assert_eq!(part2(&generator(map)), expected, "map:\n{map}");
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

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day18.txt");
        const ANSWERS: (usize, usize) = (3962, 1844);

        #[test]
        fn test() {
            let vault = generator(INPUT.trim_end_matches('\n'));

            assert_eq!(part1(&vault), ANSWERS.0);
            assert_eq!(part2(&vault), ANSWERS.1);
        }
    }
}
