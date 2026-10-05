use aoc_runner_derive::{aoc, aoc_generator};
use rustc_hash::FxHashSet as HashSet;

const SIZE: usize = 5;
const TILES: usize = SIZE * SIZE;

#[aoc_generator(day24)]
pub fn generator(input: &str) -> u32 {
    input
        .lines()
        .flat_map(str::bytes)
        .enumerate()
        .filter(|&(_, tile)| tile == b'#')
        .fold(0, |layout, (index, _)| layout | (1 << index))
}

const fn neighbors() -> [u32; TILES] {
    let mut masks = [0; TILES];
    let mut index = 0;
    while index < TILES {
        let x = index % SIZE;
        let y = index / SIZE;
        if x > 0 {
            masks[index] |= 1 << (index - 1);
        }
        if x + 1 < SIZE {
            masks[index] |= 1 << (index + 1);
        }
        if y > 0 {
            masks[index] |= 1 << (index - SIZE);
        }
        if y + 1 < SIZE {
            masks[index] |= 1 << (index + SIZE);
        }
        index += 1;
    }
    masks
}

const NEIGHBORS: [u32; TILES] = neighbors();

fn step(layout: u32) -> u32 {
    NEIGHBORS.iter().enumerate().fold(0, |next, (index, mask)| {
        let adjacent = (layout & mask).count_ones();
        let bit = 1 << index;
        if adjacent == 1 || (adjacent == 2 && layout & bit == 0) {
            next | bit
        } else {
            next
        }
    })
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

const fn recursive_neighbors() -> [[u32; 3]; TILES] {
    let mut masks = [[0; 3]; TILES];
    let mut index = 0;
    while index < TILES {
        if index != CENTER {
            masks[index][0] = NEIGHBORS[index] & !CENTER_BIT;
            let x = index % SIZE;
            let y = index / SIZE;
            if x == 0 {
                masks[index][1] |= 1 << (CENTER - 1);
            }
            if x == SIZE - 1 {
                masks[index][1] |= 1 << (CENTER + 1);
            }
            if y == 0 {
                masks[index][1] |= 1 << (CENTER - SIZE);
            }
            if y == SIZE - 1 {
                masks[index][1] |= 1 << (CENTER + SIZE);
            }

            // Stepping into the center touches an entire edge of the inner grid.
            let mut edge = 0;
            while edge < SIZE {
                if index == CENTER - SIZE {
                    masks[index][2] |= 1 << edge;
                } else if index == CENTER + SIZE {
                    masks[index][2] |= 1 << (TILES - SIZE + edge);
                } else if index == CENTER - 1 {
                    masks[index][2] |= 1 << (edge * SIZE);
                } else if index == CENTER + 1 {
                    masks[index][2] |= 1 << (edge * SIZE + SIZE - 1);
                }
                edge += 1;
            }
        }
        index += 1;
    }
    masks
}

const RECURSIVE_NEIGHBORS: [[u32; 3]; TILES] = recursive_neighbors();

fn recursive_step(layout: u32, outer: u32, inner: u32) -> u32 {
    RECURSIVE_NEIGHBORS
        .iter()
        .enumerate()
        .fold(0, |next, (index, masks)| {
            let adjacent = (layout & masks[0]).count_ones()
                + (outer & masks[1]).count_ones()
                + (inner & masks[2]).count_ones();
            let bit = 1 << index;
            if adjacent == 1 || (adjacent == 2 && layout & bit == 0) {
                next | bit
            } else {
                next
            }
        })
}

fn recursive_layouts(input: u32, minutes: usize) -> Vec<u32> {
    // Bugs can reach at most one new depth per minute. Keep empty sentinel
    // levels at both ends so every active level has an outer and inner grid.
    let center = minutes + 1;
    let mut levels = vec![0; minutes * 2 + 3];
    let mut next = levels.clone();
    levels[center] = input & !CENTER_BIT;
    for minute in 1..=minutes {
        for index in center - minute..=center + minute {
            next[index] = recursive_step(levels[index], levels[index - 1], levels[index + 1]);
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
