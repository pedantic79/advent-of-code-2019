use aoc_harness::{aoc, aoc_generator};
use rustc_hash::FxHashSet as HashSet;

const SIZE: usize = 5;
const TILES: usize = SIZE * SIZE;
const GRID_MASK: u32 = (1 << TILES) - 1;
const TOP_EDGE: u32 = (1 << SIZE) - 1;
const BOTTOM_EDGE: u32 = TOP_EDGE << (TILES - SIZE);
const LEFT_EDGE: u32 = 0b00001_00001_00001_00001_00001;
const RIGHT_EDGE: u32 = LEFT_EDGE << (SIZE - 1);

#[aoc_generator(day24)]
pub fn generator(input: &str) -> u32 {
    input
        .lines()
        .flat_map(str::bytes)
        .enumerate()
        .filter(|&(_, tile)| tile == b'#')
        .fold(0, |layout, (index, _)| layout | (1 << index))
}

// Each mask marks tiles with a neighbor in that direction. Mask horizontal
// shifts so the last column of one row never wraps into the next row.
const fn neighbor_directions(layout: u32) -> [u32; 4] {
    [
        layout << SIZE,
        layout >> SIZE,
        (layout << 1) & !LEFT_EDGE,
        (layout >> 1) & !RIGHT_EDGE,
    ]
}

#[inline]
fn evolve(layout: u32, neighbors: [u32; 4]) -> u32 {
    // Each mask tracks tiles with exactly zero, one, or two neighbors seen
    // so far. Update in descending order to use the previous counts.
    let mut zero = GRID_MASK;
    let mut one = 0;
    let mut two = 0;
    for occupied in neighbors {
        two = (two & !occupied) | (one & occupied);
        one = (one & !occupied) | (zero & occupied);
        zero &= !occupied;
    }
    one | (two & !layout)
}

fn step(layout: u32) -> u32 {
    let layout = layout & GRID_MASK;
    evolve(layout, neighbor_directions(layout))
}

#[aoc(day24, part1)]
pub fn part1(input: &u32) -> u32 {
    // Row-major occupancy bits are exactly the biodiversity rating.
    let mut layout = *input;
    let mut seen = HashSet::default();
    while seen.insert(layout) {
        layout = step(layout);
    }
    layout
}

const CENTER: usize = TILES / 2;
const CENTER_BIT: u32 = 1 << CENTER;

struct Portal {
    tile: u32,
    inner_edge: u32,
    same_level_neighbors: u32,
}

impl Portal {
    const fn new(index: usize, inner_edge: u32) -> Self {
        let tile = 1 << index;
        let [north, south, west, east] = neighbor_directions(tile);
        Self {
            tile,
            inner_edge,
            same_level_neighbors: (north | south | west | east) & !CENTER_BIT,
        }
    }
}

// Match neighbor_directions: north, south, west, east. Each center-adjacent
// tile touches an inner edge; inversely, that outer tile borders our edge.
const PORTALS: [Portal; 4] = [
    Portal::new(CENTER - SIZE, TOP_EDGE),
    Portal::new(CENTER + SIZE, BOTTOM_EDGE),
    Portal::new(CENTER - 1, LEFT_EDGE),
    Portal::new(CENTER + 1, RIGHT_EDGE),
];

fn recursive_step(layout: u32, outer: u32, inner: u32) -> u32 {
    let layout = layout & GRID_MASK & !CENTER_BIT;
    let mut neighbors = neighbor_directions(layout);
    for (direction, portal) in neighbors.iter_mut().zip(&PORTALS) {
        if outer & portal.tile != 0 {
            *direction |= portal.inner_edge;
        }
    }
    let mut next = evolve(layout, neighbors) & !CENTER_BIT;

    // Center-adjacent tiles also see a whole inner edge, so their counts
    // can exceed four. Handle those four tiles separately.
    for portal in &PORTALS {
        let adjacent = (layout & portal.same_level_neighbors).count_ones()
            + (inner & portal.inner_edge).count_ones();
        next &= !portal.tile;
        if adjacent == 1 || (adjacent == 2 && layout & portal.tile == 0) {
            next |= portal.tile;
        }
    }
    next
}

fn recursive_layouts(input: u32, minutes: usize) -> Vec<u32> {
    // Bugs can reach at most one new depth per minute. Keep empty sentinel
    // levels at both ends so every active level has an outer and inner grid.
    let center = minutes + 1;
    let mut levels = vec![0; minutes * 2 + 3];
    let mut next = levels.clone();
    levels[center] = input & !CENTER_BIT;
    let mut first = center;
    let mut last = center;
    for _ in 0..minutes {
        for index in first - 1..=last + 1 {
            next[index] = recursive_step(levels[index], levels[index - 1], levels[index + 1]);
        }
        // Expand only when bugs actually reach a new depth. Never shrink the
        // range, so reused buffers remain zero outside the computed region.
        if next[first - 1] != 0 {
            first -= 1;
        }
        if next[last + 1] != 0 {
            last += 1;
        }
        std::mem::swap(&mut levels, &mut next);
    }
    levels
}

#[aoc(day24, part2)]
pub fn part2(input: &u32) -> u32 {
    recursive_layouts(*input, 200)
        .iter()
        .map(|layout| layout.count_ones())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "....#\n#..#.\n#..##\n..#..\n#....";

    fn coordinate_step(levels: [u32; 3], recursive: bool) -> u32 {
        let occupied =
            |level: usize, x: i32, y: i32| u32::from(levels[level] & (1 << (y * 5 + x)) != 0);
        let mut next = 0;
        for y in 0..5 {
            for x in 0..5 {
                if recursive && (x, y) == (2, 2) {
                    continue;
                }
                let mut adjacent = 0;
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, ny) = (x + dx, y + dy);
                    adjacent += match (nx, ny) {
                        (-1, _) if recursive => occupied(1, 1, 2),
                        (5, _) if recursive => occupied(1, 3, 2),
                        (_, -1) if recursive => occupied(1, 2, 1),
                        (_, 5) if recursive => occupied(1, 2, 3),
                        (2, 2) if recursive => (0..5)
                            .map(|edge| match (dx, dy) {
                                (1, 0) => occupied(2, 0, edge),
                                (-1, 0) => occupied(2, 4, edge),
                                (0, 1) => occupied(2, edge, 0),
                                (0, -1) => occupied(2, edge, 4),
                                _ => unreachable!(),
                            })
                            .sum(),
                        (0..=4, 0..=4) => occupied(0, nx, ny),
                        _ => 0,
                    };
                }
                if adjacent == 1 || (adjacent == 2 && occupied(0, x, y) == 0) {
                    next |= 1 << (y * 5 + x);
                }
            }
        }
        next
    }

    #[test]
    fn steps_match_coordinate_model() {
        let check = |levels: [u32; 3]| {
            assert_eq!(step(levels[0]), coordinate_step(levels, false));
            assert_eq!(
                recursive_step(levels[0], levels[1], levels[2]),
                coordinate_step(levels, true)
            );
        };
        for level in 0..3 {
            for tile in 0..TILES {
                let mut levels = [0; 3];
                levels[level] = 1 << tile;
                check(levels);
            }
        }
        check([(1 << TILES) - 1; 3]);
        let mut seed = 123_456_789_u32;
        for _ in 0..1_000 {
            check(std::array::from_fn(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                seed & ((1 << TILES) - 1)
            }));
        }
    }

    #[test]
    fn recursive_depth_growth_matches_coordinate_model() {
        const MINUTES: usize = 16;
        let center = MINUTES + 1;
        for input in [0, CENTER_BIT, 1, 1 << 7, GRID_MASK, generator(SAMPLE)] {
            let mut levels = vec![0; MINUTES * 2 + 3];
            levels[center] = input & !CENTER_BIT;
            for minute in 0..=MINUTES {
                let actual = recursive_layouts(input, minute);
                assert_eq!(actual, levels[center - minute - 1..=center + minute + 1]);
                let mut next = vec![0; levels.len()];
                for index in 1..levels.len() - 1 {
                    next[index] = coordinate_step(
                        [levels[index], levels[index - 1], levels[index + 1]],
                        true,
                    );
                }
                levels = next;
            }
        }
    }

    #[test]
    fn sample_evolution() {
        let mut layout = generator(SAMPLE);
        for expected in [
            "#..#.\n####.\n###.#\n##.##\n.##..",
            "#####\n....#\n....#\n...#.\n#.###",
            "#....\n####.\n...##\n#.##.\n.##.#",
            "####.\n....#\n##..#\n.....\n##...",
        ] {
            layout = step(layout);
            assert_eq!(layout, generator(expected));
        }
    }

    #[test]
    fn first_repeated_biodiversity() {
        assert_eq!(part1(&generator(SAMPLE)), 2_129_920);
    }

    #[test]
    fn empty_layout_already_repeats() {
        assert_eq!(part1(&0), 0);
    }

    #[test]
    fn biodiversity_uses_row_major_bits() {
        assert_eq!(
            generator("#....\n.....\n.....\n.....\n....#\n"),
            1 | (1 << 24)
        );
    }

    #[test]
    fn recursive_sample_after_ten_minutes() {
        let levels = recursive_layouts(generator(SAMPLE), 10);
        let center = levels.len() / 2;
        let expected = [
            "..#..\n.#.#.\n..?.#\n.#.#.\n..#..",
            "...#.\n...##\n..?..\n...##\n...#.",
            "#.#..\n.#...\n..?..\n.#...\n#.#..",
            ".#.##\n....#\n..?.#\n...##\n.###.",
            "#..##\n...##\n..?..\n...#.\n.####",
            ".#...\n.#.##\n.#?..\n.....\n.....",
            ".##..\n#..##\n..?.#\n##.##\n#####",
            "###..\n##.#.\n#.?..\n.#.##\n#.#..",
            "..###\n.....\n#.?..\n#....\n#...#",
            ".###.\n#..#.\n#.?..\n##.#.\n.....",
            "####.\n#..#.\n#.?#.\n####.\n.....",
        ];
        for (offset, expected) in expected.into_iter().enumerate() {
            assert_eq!(levels[center - 5 + offset], generator(expected));
        }
        assert_eq!(
            levels.iter().map(|layout| layout.count_ones()).sum::<u32>(),
            99
        );
        assert!(levels[..center - 5].iter().all(|&layout| layout == 0));
        assert!(levels[center + 6..].iter().all(|&layout| layout == 0));
    }

    #[test]
    fn recursive_empty_layout_and_center_stay_empty() {
        assert_eq!(part2(&0), 0);
        assert_eq!(part2(&(1 << 12)), 0);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day24.txt");
        const ANSWERS: (u32, u32) = (12_129_040, 2109);

        #[test]
        fn answers() {
            let layout = generator(INPUT);

            assert_eq!(part1(&layout), ANSWERS.0);
            assert_eq!(part2(&layout), ANSWERS.1);
        }
    }
}
