use aoc_runner_derive::{aoc, aoc_generator};

use crate::common::utils::mod_inv;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Technique {
    NewStack,
    Cut(i128),
    Increment(i128),
}

impl Technique {
    fn apply(self, position: i128, size: i128) -> i128 {
        // Normalize operands first so even full-width cuts and increments fit.
        match self {
            Self::NewStack => size - 1 - position,
            Self::Cut(amount) => (position - amount.rem_euclid(size)).rem_euclid(size),
            Self::Increment(increment) => (position * increment.rem_euclid(size)).rem_euclid(size),
        }
    }
}

#[aoc_generator(day22)]
pub fn generator(input: &str) -> Vec<Technique> {
    input
        .lines()
        .map(|line| {
            if line == "deal into new stack" {
                Technique::NewStack
            } else if let Some(amount) = line.strip_prefix("cut ") {
                Technique::Cut(amount.parse().unwrap())
            } else if let Some(increment) = line.strip_prefix("deal with increment ") {
                Technique::Increment(increment.parse().unwrap())
            } else {
                panic!("unknown shuffle technique: {line}");
            }
        })
        .collect()
}

// Every technique maps a card's old position x to a*x + b (mod deck size).
// i128 also holds products of two positions in the much larger Part 2 deck.
#[derive(Clone, Copy)]
struct Shuffle {
    multiplier: i128,
    offset: i128,
}

impl Shuffle {
    const IDENTITY: Self = Self {
        multiplier: 1,
        offset: 0,
    };

    fn apply(self, position: i128, size: i128) -> i128 {
        (self.multiplier * position + self.offset).rem_euclid(size)
    }

    fn then(self, next: Self, size: i128) -> Self {
        Self {
            multiplier: (next.multiplier * self.multiplier).rem_euclid(size),
            offset: next.apply(self.offset, size),
        }
    }

    fn repeat(mut self, mut repetitions: u64, size: i128) -> Self {
        // Compose powers of the whole shuffle; this also works when multiplier == 1.
        let mut result = Self::IDENTITY;
        while repetitions != 0 {
            if repetitions & 1 != 0 {
                result = result.then(self, size);
            }
            self = self.then(self, size);
            repetitions >>= 1;
        }
        result
    }

    fn card_at(self, position: i128, size: i128) -> i128 {
        // Solve position = a*card + b. Valid shuffles make a invertible modulo size.
        ((position - self.offset) * mod_inv(self.multiplier, size)).rem_euclid(size)
    }
}

fn shuffle(techniques: &[Technique], size: i128) -> Shuffle {
    techniques
        .iter()
        .fold(Shuffle::IDENTITY, |shuffle, technique| Shuffle {
            multiplier: match technique {
                Technique::NewStack => (-shuffle.multiplier).rem_euclid(size),
                Technique::Cut(_) => shuffle.multiplier,
                Technique::Increment(_) => technique.apply(shuffle.multiplier, size),
            },
            offset: technique.apply(shuffle.offset, size),
        })
}

#[aoc(day22, part1)]
pub fn part1(techniques: &[Technique]) -> i128 {
    // Only one card's position is needed; do not build the full affine shuffle.
    techniques.iter().fold(2019, |position, technique| {
        technique.apply(position, 10_007)
    })
}

#[aoc(day22, part2)]
pub fn part2(techniques: &[Technique]) -> i128 {
    const SIZE: i128 = 119_315_717_514_047;
    shuffle(techniques, SIZE)
        .repeat(101_741_582_076_661, SIZE)
        .card_at(2020, SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLES: [(&str, [i128; 10]); 4] = [
        (
            "deal with increment 7\ndeal into new stack\ndeal into new stack",
            [0, 3, 6, 9, 2, 5, 8, 1, 4, 7],
        ),
        (
            "cut 6\ndeal with increment 7\ndeal into new stack",
            [3, 0, 7, 4, 1, 8, 5, 2, 9, 6],
        ),
        (
            "deal with increment 7\ndeal with increment 9\ncut -2",
            [6, 3, 0, 7, 4, 1, 8, 5, 2, 9],
        ),
        (
            "deal into new stack\ncut -2\ndeal with increment 7\ncut 8\ncut -4\n\
             deal with increment 7\ncut 3\ndeal with increment 9\n\
             deal with increment 3\ncut -1",
            [9, 2, 5, 8, 1, 4, 7, 0, 3, 6],
        ),
    ];

    fn assert_deck(input: &str, expected: [i128; 10]) {
        let shuffle = shuffle(&generator(input), 10);
        for (position, card) in expected.into_iter().enumerate() {
            assert_eq!(shuffle.apply(card, 10), i128::try_from(position).unwrap());
        }
    }

    #[test]
    fn individual_techniques() {
        assert_deck("deal into new stack", [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
        assert_deck("cut 3", [3, 4, 5, 6, 7, 8, 9, 0, 1, 2]);
        assert_deck("cut -4", [6, 7, 8, 9, 0, 1, 2, 3, 4, 5]);
        assert_deck("deal with increment 3", [0, 7, 4, 1, 8, 5, 2, 9, 6, 3]);
    }

    #[test]
    fn combined_examples() {
        for (input, expected) in EXAMPLES {
            assert_deck(input, expected);
        }
    }

    #[test]
    fn cuts_wrap_and_empty_shuffle_preserves_order() {
        assert_deck("cut 23", [3, 4, 5, 6, 7, 8, 9, 0, 1, 2]);
        assert_deck("cut -24", [6, 7, 8, 9, 0, 1, 2, 3, 4, 5]);
        assert_deck("", [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn part1_matches_literal_decks() {
        for input in EXAMPLES.iter().map(|&(input, _)| input).chain([
            "",
            "cut 0",
            "cut 10007",
            "cut -20017",
            "deal into new stack",
            "deal with increment 10008",
        ]) {
            let techniques = generator(input);
            let mut cards: Vec<_> = (0..10_007).collect();
            deal(&mut cards, &techniques);
            assert_eq!(cards[usize::try_from(part1(&techniques)).unwrap()], 2019);
        }
    }

    #[test]
    fn full_width_coefficients_match_normalized_techniques() {
        let techniques = [
            Technique::Cut(i128::MIN),
            Technique::Increment(i128::MAX),
            Technique::Cut(i128::MAX),
            Technique::Increment(i128::MIN),
        ];
        for size in [10_007, 119_315_717_514_047] {
            let normalized = techniques.map(|technique| match technique {
                Technique::Cut(amount) => Technique::Cut(amount.rem_euclid(size)),
                Technique::Increment(increment) => Technique::Increment(increment.rem_euclid(size)),
                Technique::NewStack => Technique::NewStack,
            });
            if size == 10_007 {
                assert_eq!(part1(&techniques), part1(&normalized));
            } else {
                assert_eq!(part2(&techniques), part2(&normalized));
            }
        }
    }

    fn deal(cards: &mut Vec<i128>, techniques: &[Technique]) {
        let size = i128::try_from(cards.len()).unwrap();
        for technique in techniques {
            match *technique {
                Technique::NewStack => cards.reverse(),
                Technique::Cut(amount) => {
                    cards.rotate_left(usize::try_from(amount.rem_euclid(size)).unwrap());
                }
                Technique::Increment(increment) => {
                    let increment = usize::try_from(increment).unwrap();
                    let mut dealt = vec![0; cards.len()];
                    for (position, &card) in cards.iter().enumerate() {
                        dealt[position * increment % cards.len()] = card;
                    }
                    *cards = dealt;
                }
            }
        }
    }

    #[test]
    fn repeated_shuffles_and_inverse_match_literal_decks() {
        for input in EXAMPLES.iter().map(|&(input, _)| input).chain([
            "",
            "cut 3",
            "cut -4",
            "deal into new stack",
        ]) {
            let techniques = generator(input);
            for size in [10, 11, 97] {
                let single = shuffle(&techniques, size);
                let mut cards: Vec<_> = (0..size).collect();
                for repetitions in 0..=64 {
                    let repeated = single.repeat(repetitions, size);
                    for (position, &card) in cards.iter().enumerate() {
                        let position = i128::try_from(position).unwrap();
                        assert_eq!(repeated.apply(card, size), position);
                        assert_eq!(repeated.card_at(position, size), card);
                    }
                    deal(&mut cards, &techniques);
                }
            }
        }
    }

    #[test]
    fn huge_repetition_counts_and_coefficients() {
        let size = 119_315_717_514_047;
        let repetitions = 101_741_582_076_661;
        let translation = shuffle(&generator("cut -37"), size).repeat(repetitions, size);
        assert_eq!(
            translation.card_at(2020, size),
            (2020 - 37 * i128::from(repetitions)).rem_euclid(size),
        );

        // This maps x to 1-x and is its own inverse, even with huge coefficients.
        let reflection = shuffle(
            &[Technique::Increment(size - 1), Technique::Cut(size - 1)],
            size,
        )
        .repeat(repetitions, size);
        for position in [0, 2020, size - 1] {
            assert_eq!(
                reflection.card_at(position, size),
                (1 - position).rem_euclid(size)
            );
        }
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day22.txt");
        const ANSWERS: (i128, i128) = (4703, 55_627_600_867_625);

        #[test]
        fn answers() {
            let techniques = generator(INPUT);

            assert_eq!(part1(&techniques), ANSWERS.0);
            assert_eq!(part2(&techniques), ANSWERS.1);
        }
    }
}
