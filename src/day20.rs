use std::collections::VecDeque;

use aoc_runner_derive::{aoc, aoc_generator};
use pathfinding::prelude::dijkstra;
use rustc_hash::FxHashMap as HashMap;

#[derive(Clone, Copy)]
struct Portal {
    exit: usize,
    outer: bool,
}

pub struct Maze {
    cells: Vec<u8>,
    width: usize,
    start: usize,
    end: usize,
    portals: Vec<Option<Portal>>,
}

struct Graph {
    edges: Vec<Vec<(usize, usize)>>,
    portals: Vec<Option<Portal>>,
}

#[aoc_generator(day20)]
pub fn generator(input: &str) -> Maze {
    let rows: Vec<_> = input.lines().map(str::as_bytes).collect();
    let height = rows.len();
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0);
    assert!(width != 0 && height != 0, "maze must not be empty");

    let mut cells = vec![b' '; width * height];
    for (y, row) in rows.into_iter().enumerate() {
        cells[y * width..y * width + row.len()].copy_from_slice(row);
    }

    let mut entrances: HashMap<[u8; 2], Vec<(usize, bool)>> = HashMap::default();
    for position in 0..cells.len() {
        if cells[position] != b'.' {
            continue;
        }

        let x = position % width;
        let y = position / width;
        for (dx, dy) in [(0_isize, -1_isize), (1, 0), (0, 1), (-1, 0)] {
            let Some(near_x) = x.checked_add_signed(dx) else {
                continue;
            };
            let Some(near_y) = y.checked_add_signed(dy) else {
                continue;
            };
            let Some(far_x) = x.checked_add_signed(2 * dx) else {
                continue;
            };
            let Some(far_y) = y.checked_add_signed(2 * dy) else {
                continue;
            };
            if near_x >= width || far_x >= width || near_y >= height || far_y >= height {
                continue;
            }

            let near = cells[near_y * width + near_x];
            let far = cells[far_y * width + far_x];
            if !near.is_ascii_uppercase() || !far.is_ascii_uppercase() {
                continue;
            }

            let label = if dx < 0 || dy < 0 {
                [far, near]
            } else {
                [near, far]
            };
            let outer = x <= 2 || y <= 2 || x + 3 >= width || y + 3 >= height;
            entrances.entry(label).or_default().push((position, outer));
        }
    }

    let start = single_entrance(&mut entrances, *b"AA");
    let end = single_entrance(&mut entrances, *b"ZZ");
    let mut portals = vec![None; cells.len()];
    for (label, pair) in entrances {
        let [(first, first_outer), (second, second_outer)] = pair.as_slice() else {
            panic!(
                "portal {}{} must have two entrances",
                char::from(label[0]),
                char::from(label[1])
            );
        };
        portals[*first] = Some(Portal {
            exit: *second,
            outer: *first_outer,
        });
        portals[*second] = Some(Portal {
            exit: *first,
            outer: *second_outer,
        });
    }

    Maze {
        cells,
        width,
        start,
        end,
        portals,
    }
}

fn single_entrance(entrances: &mut HashMap<[u8; 2], Vec<(usize, bool)>>, label: [u8; 2]) -> usize {
    let positions = entrances.remove(&label).expect("portal must exist");
    assert_eq!(positions.len(), 1, "portal must have one entrance");
    positions[0].0
}

fn build_graph(maze: &Maze) -> Graph {
    let mut nodes = vec![maze.start, maze.end];
    nodes.extend(
        maze.portals
            .iter()
            .enumerate()
            .filter_map(|(position, portal)| portal.map(|_| position)),
    );

    let mut node_at = vec![None; maze.cells.len()];
    for (node, &position) in nodes.iter().enumerate() {
        node_at[position] = Some(node);
    }

    let mut edges = vec![Vec::new(); nodes.len()];
    let mut portals = vec![None; nodes.len()];
    let mut visited = vec![false; maze.cells.len()];
    let mut queue = VecDeque::new();
    // Stop each corridor at an entrance so a level-specific wall cannot be crossed inside an edge.
    for (source, &position) in nodes.iter().enumerate() {
        portals[source] = maze.portals[position].map(|portal| Portal {
            exit: node_at[portal.exit].expect("portal exit must be an entrance"),
            outer: portal.outer,
        });
        visited.fill(false);
        queue.clear();
        visited[position] = true;
        queue.push_back((position, 0));

        while let Some((position, distance)) = queue.pop_front() {
            for next in adjacent_positions(maze, position).into_iter().flatten() {
                if maze.cells[next] != b'.' || visited[next] {
                    continue;
                }
                visited[next] = true;
                if let Some(target) = node_at[next] {
                    edges[source].push((target, distance + 1));
                } else {
                    queue.push_back((next, distance + 1));
                }
            }
        }
    }

    Graph { edges, portals }
}

#[aoc(day20, part1)]
pub fn part1(maze: &Maze) -> usize {
    let mut visited = vec![false; maze.cells.len()];
    let mut queue = VecDeque::from([(maze.start, 0)]);
    visited[maze.start] = true;

    while let Some((position, distance)) = queue.pop_front() {
        if position == maze.end {
            return distance;
        }

        for next in adjacent_positions(maze, position)
            .into_iter()
            .chain([maze.portals[position].map(|portal| portal.exit)])
            .flatten()
        {
            if maze.cells[next] == b'.' && !visited[next] {
                visited[next] = true;
                queue.push_back((next, distance + 1));
            }
        }
    }

    panic!("maze exit is unreachable")
}

#[aoc(day20, part2)]
pub fn part2(maze: &Maze) -> usize {
    let graph = build_graph(maze);
    dijkstra(
        &(0_usize, 0_usize),
        |&(node, level)| {
            let portals = &graph.portals;
            let walking = graph.edges[node]
                .iter()
                .filter(move |&&(next, _)| {
                    (level == 0 || (next != 0 && next != 1))
                        && (level != 0 || !portals[next].is_some_and(|portal| portal.outer))
                })
                .map(move |&(next, distance)| ((next, level), distance));
            let teleport = portals[node].and_then(|portal| {
                let next_level = if portal.outer {
                    level.checked_sub(1)
                } else {
                    level.checked_add(1)
                }?;
                Some(((portal.exit, next_level), 1))
            });
            walking.chain(teleport)
        },
        |&(node, level)| node == 1 && level == 0,
    )
    .expect("recursive maze exit is unreachable")
    .1
}

fn adjacent_positions(maze: &Maze, position: usize) -> [Option<usize>; 4] {
    let column = position % maze.width;
    [
        position.checked_sub(maze.width),
        (position + maze.width < maze.cells.len()).then_some(position + maze.width),
        (column > 0).then(|| position - 1),
        (column + 1 < maze.width).then_some(position + 1),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_EXAMPLE: &str = concat!(
        "         A\n",
        "         A\n",
        "  #######.#########\n",
        "  #######.........#\n",
        "  #######.#######.#\n",
        "  #######.#######.#\n",
        "  #######.#######.#\n",
        "  #####  B    ###.#\n",
        "BC...##  C    ###.#\n",
        "  ##.##       ###.#\n",
        "  ##...DE  F  ###.#\n",
        "  #####    G  ###.#\n",
        "  #########.#####.#\n",
        "DE..#######...###.#\n",
        "  #.#########.###.#\n",
        "FG..#########.....#\n",
        "  ###########.#####\n",
        "             Z\n",
        "             Z",
    );

    const LARGE_EXAMPLE: &str = concat!(
        "                   A\n",
        "                   A\n",
        "  #################.#############\n",
        "  #.#...#...................#.#.#\n",
        "  #.#.#.###.###.###.#########.#.#\n",
        "  #.#.#.......#...#.....#.#.#...#\n",
        "  #.#########.###.#####.#.#.###.#\n",
        "  #.............#.#.....#.......#\n",
        "  ###.###########.###.#####.#.#.#\n",
        "  #.....#        A   C    #.#.#.#\n",
        "  #######        S   P    #####.#\n",
        "  #.#...#                 #......VT\n",
        "  #.#.#.#                 #.#####\n",
        "  #...#.#               YN....#.#\n",
        "  #.###.#                 #####.#\n",
        "DI....#.#                 #.....#\n",
        "  #####.#                 #.###.#\n",
        "ZZ......#               QG....#..AS\n",
        "  ###.###                 #######\n",
        "JO..#.#.#                 #.....#\n",
        "  #.#.#.#                 ###.#.#\n",
        "  #...#..DI             BU....#..LF\n",
        "  #####.#                 #.#####\n",
        "YN......#               VT..#....QG\n",
        "  #.###.#                 #.###.#\n",
        "  #.#...#                 #.....#\n",
        "  ###.###    J L     J    #.#.###\n",
        "  #.....#    O F     P    #.#...#\n",
        "  #.###.#####.#.#####.#####.###.#\n",
        "  #...#.#.#...#.....#.....#.#...#\n",
        "  #.#####.###.###.#.#.#########.#\n",
        "  #...#.#.....#...#.#.#.#.....#.#\n",
        "  #.###.#####.###.###.#.#.#######\n",
        "  #.#.........#...#.............#\n",
        "  #########.###.###.#############\n",
        "           B   J   C\n",
        "           U   P   P",
    );

    const RECURSIVE_EXAMPLE: &str = concat!(
        "             Z L X W       C\n",
        "             Z P Q B       K\n",
        "  ###########.#.#.#.#######.###############\n",
        "  #...#.......#.#.......#.#.......#.#.#...#\n",
        "  ###.#.#.#.#.#.#.#.###.#.#.#######.#.#.###\n",
        "  #.#...#.#.#...#.#.#...#...#...#.#.......#\n",
        "  #.###.#######.###.###.#.###.###.#.#######\n",
        "  #...#.......#.#...#...#.............#...#\n",
        "  #.#########.#######.#.#######.#######.###\n",
        "  #...#.#    F       R I       Z    #.#.#.#\n",
        "  #.###.#    D       E C       H    #.#.#.#\n",
        "  #.#...#                           #...#.#\n",
        "  #.###.#                           #.###.#\n",
        "  #.#....OA                       WB..#.#..ZH\n",
        "  #.###.#                           #.#.#.#\n",
        "CJ......#                           #.....#\n",
        "  #######                           #######\n",
        "  #.#....CK                         #......IC\n",
        "  #.###.#                           #.###.#\n",
        "  #.....#                           #...#.#\n",
        "  ###.###                           #.#.#.#\n",
        "XF....#.#                         RF..#.#.#\n",
        "  #####.#                           #######\n",
        "  #......CJ                       NM..#...#\n",
        "  ###.#.#                           #.###.#\n",
        "RE....#.#                           #......RF\n",
        "  ###.###        X   X       L      #.#.#.#\n",
        "  #.....#        F   Q       P      #.#.#.#\n",
        "  ###.###########.###.#######.#########.###\n",
        "  #.....#...#.....#.......#...#.....#.#...#\n",
        "  #####.#.###.#######.#######.###.###.#.#.#\n",
        "  #.......#.......#.#.#.#.#...#...#...#.#.#\n",
        "  #####.###.#####.#.#.#.#.###.###.#.###.###\n",
        "  #.......#.....#.#...#...............#...#\n",
        "  #############.#.#.###.###################\n",
        "               A O F   N\n",
        "               A A D   M",
    );

    #[test]
    fn part1_examples() {
        assert_eq!(part1(&generator(SMALL_EXAMPLE)), 23);
        assert_eq!(part1(&generator(LARGE_EXAMPLE)), 58);
    }

    #[test]
    fn part2_examples() {
        assert_eq!(part2(&generator(SMALL_EXAMPLE)), 26);
        assert_eq!(part2(&generator(RECURSIVE_EXAMPLE)), 396);
    }

    #[test]
    fn corridor_edges_stop_at_entrances() {
        let maze = generator(SMALL_EXAMPLE);
        let graph = build_graph(&maze);
        let mut distances: Vec<_> = graph.edges[0]
            .iter()
            .map(|&(_, distance)| distance)
            .collect();
        distances.sort_unstable();
        assert_eq!(distances, [4, 26, 30]);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day20.txt");
        const ANSWERS: (usize, usize) = (590, 7180);

        #[test]
        fn answers() {
            let maze = generator(INPUT.trim_end_matches('\n'));

            assert_eq!(part1(&maze), ANSWERS.0);
            assert_eq!(part2(&maze), ANSWERS.1);
        }
    }
}
