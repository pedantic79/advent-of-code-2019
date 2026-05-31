use aoc_runner_derive::{aoc, aoc_generator};
use num::Integer;

const PAIRS: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Moon {
    pub pos: [i32; 3],
    pub vel: [i32; 3],
}

#[aoc_generator(day12)]
pub fn generator(input: &str) -> Vec<Moon> {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let clean = line.replace(['<', '>', 'x', 'y', 'z', '='], "");
            let mut nums = clean.split(',').map(|s| s.trim().parse::<i32>().unwrap());
            let x = nums.next().unwrap();
            let y = nums.next().unwrap();
            let z = nums.next().unwrap();
            Moon {
                pos: [x, y, z],
                vel: [0, 0, 0],
            }
        })
        .collect()
}

#[inline(always)]
fn update_gravity_1d(pos: &[i32; 4], vel: &mut [i32; 4]) {
    for &(i, j) in &PAIRS {
        let cmp = (pos[i] > pos[j]) as i32 - (pos[i] < pos[j]) as i32;
        vel[i] -= cmp;
        vel[j] += cmp;
    }
}

fn simulate(moons: &mut [Moon], steps: usize) {
    assert_eq!(
        moons.len(),
        4,
        "This simulator is optimized for exactly 4 moons"
    );
    for _ in 0..steps {
        // Gravity update (branchless & compile-time unrolled loop over pairs)
        for d in 0..3 {
            let pos = [
                moons[0].pos[d],
                moons[1].pos[d],
                moons[2].pos[d],
                moons[3].pos[d],
            ];
            let mut vel = [
                moons[0].vel[d],
                moons[1].vel[d],
                moons[2].vel[d],
                moons[3].vel[d],
            ];

            update_gravity_1d(&pos, &mut vel);

            for i in 0..4 {
                moons[i].vel[d] = vel[i];
            }
        }

        // Apply velocity updates
        for moon in moons.iter_mut() {
            for d in 0..3 {
                moon.pos[d] += moon.vel[d];
            }
        }
    }
}

fn calculate_total_energy(moons: &[Moon]) -> i32 {
    moons
        .iter()
        .map(|moon| {
            let pot: i32 = moon.pos.iter().map(|&x| x.abs()).sum();
            let kin: i32 = moon.vel.iter().map(|&v| v.abs()).sum();
            pot * kin
        })
        .sum()
}

#[aoc(day12, part1)]
pub fn part1(moons: &[Moon]) -> i32 {
    let mut moons = moons.to_vec();
    simulate(&mut moons, 1000);
    calculate_total_energy(&moons)
}

fn find_cycle_1d(initial_pos: [i32; 4]) -> u64 {
    let mut pos = initial_pos;
    let mut vel = [0; 4];
    let mut steps = 0u64;

    loop {
        steps += 1;
        update_gravity_1d(&pos, &mut vel);

        // Apply velocity updates
        for i in 0..4 {
            pos[i] += vel[i];
        }

        // Check if back to initial state
        if vel == [0; 4] && pos == initial_pos {
            return steps;
        }
    }
}

#[aoc(day12, part2)]
pub fn part2(moons: &[Moon]) -> u64 {
    assert_eq!(
        moons.len(),
        4,
        "This solution is optimized for exactly 4 moons"
    );

    let mut x_pos = [0; 4];
    let mut y_pos = [0; 4];
    let mut z_pos = [0; 4];

    for i in 0..4 {
        x_pos[i] = moons[i].pos[0];
        y_pos[i] = moons[i].pos[1];
        z_pos[i] = moons[i].pos[2];
    }

    let px = find_cycle_1d(x_pos);
    let py = find_cycle_1d(y_pos);
    let pz = find_cycle_1d(z_pos);

    px.lcm(&py.lcm(&pz))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE1: &str = r#"<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>"#;

    const SAMPLE2: &str = r#"<x=-8, y=-10, z=0>
<x=5, y=5, z=10>
<x=2, y=-7, z=3>
<x=9, y=-8, z=-3>"#;

    #[test]
    pub fn input_test() {
        let moons1 = generator(SAMPLE1);
        assert_eq!(moons1.len(), 4);
        assert_eq!(moons1[0].pos, [-1, 0, 2]);
        assert_eq!(moons1[0].vel, [0, 0, 0]);
    }

    #[test]
    pub fn sample1_part1_test() {
        let mut moons = generator(SAMPLE1);
        simulate(&mut moons, 10);
        assert_eq!(calculate_total_energy(&moons), 179);
    }

    #[test]
    pub fn sample2_part1_test() {
        let mut moons = generator(SAMPLE2);
        simulate(&mut moons, 100);
        assert_eq!(calculate_total_energy(&moons), 1940);
    }

    #[test]
    pub fn sample1_part2_test() {
        let moons = generator(SAMPLE1);
        assert_eq!(part2(&moons), 2772);
    }

    #[test]
    pub fn sample2_part2_test() {
        let moons = generator(SAMPLE2);
        assert_eq!(part2(&moons), 4686774924);
    }

    mod regression {
        use super::*;

        const INPUT: &str = include_str!("../input/2019/day12.txt");
        const ANSWERS: (i32, u64) = (10944, 484244804958744);

        #[test]
        pub fn test() {
            let input = INPUT.trim_end_matches('\n');
            let output = generator(input);

            assert_eq!(part1(&output), ANSWERS.0);
            assert_eq!(part2(&output), ANSWERS.1);
        }
    }
}
