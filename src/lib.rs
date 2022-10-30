#![warn(clippy::all, clippy::pedantic)]
use rand::prelude::*;
use rand_seeder::Seeder;
use rand_pcg::Pcg64;

pub struct SeededDiceRoller {
    rng: Pcg64
}

impl SeededDiceRoller {
    pub fn new(seed: &str, step: &str) -> Self {
        Self { rng: Seeder::from(format!("{}{}", step, seed)).make_rng() }
    }

    pub fn toss_coin(&mut self) -> bool {
        self.rng.gen::<bool>()
    }

    pub fn roll(&mut self, dice: u16, die_type: u16, modifier: i16) -> i32 {
        let mut result = 0;
        let die_type = die_type as i32;
        for _ in 0..dice {
            result += self.rng.gen::<u16>() as i32 % &die_type + 1;
        }
        result += modifier as i32;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_is_within_bounds() {
        let mut rng = SeededDiceRoller::new("seed", "test");
        for _ in 0..1000 {
            let n: i32 = rng.roll(1, 6, 0);
            assert!(n >= 1);
            assert!(n <= 6);
        }
        for _ in 0..1000 {
            let n: i32 = rng.roll(3, 6, 0);
            assert!(n >= 3);
            assert!(n <= 18);
        }
        for _ in 0..1000 {
            let n: i32 = rng.roll(1, 20, 0);
            assert!(n >= 1);
            assert!(n <= 20);
        }
    }

    #[test]
    fn generator_is_deterministic() {
        let mut rng = SeededDiceRoller::new("seed", "test");
        assert!(rng.roll(1, 6, 0) == 6);
        assert!(rng.roll(3, 6, -5) == 4);
        assert!(rng.roll(1, 6, 0) == 4);
        assert!(rng.roll(1, 20, 0) == 2);
        assert!(rng.roll(1, 6, -15) == -11);
        assert!(rng.roll(69, 6, 0) == 254);
        assert!(rng.roll(2, 123, 0) == 200);
        assert!(rng.roll(1, 6, 3343) == 3344);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == true);

        rng = SeededDiceRoller::new("other_seed", "test");
        assert!(rng.roll(1, 6, 0) == 2);
        assert!(rng.roll(3, 6, -5) == 3);
        assert!(rng.roll(1, 6, 0) == 2);
        assert!(rng.roll(1, 20, 0) == 18);
        assert!(rng.roll(1, 6, -15) == -11);
        assert!(rng.roll(69, 6, 0) == 238);
        assert!(rng.roll(2, 123, 0) == 120);
        assert!(rng.roll(1, 6, 3343) == 3345);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == false);

        rng = SeededDiceRoller::new("other_seed", "step");
        assert!(rng.roll(1, 6, 0) == 6);
        assert!(rng.roll(3, 6, -5) == 5);
        assert!(rng.roll(1, 6, 0) == 1);
        assert!(rng.roll(1, 20, 0) == 7);
        assert!(rng.roll(1, 6, -15) == -14);
        assert!(rng.roll(69, 6, 0) == 223);
        assert!(rng.roll(2, 123, 0) == 130);
        assert!(rng.roll(1, 6, 3343) == 3349);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == false);
    }
}
