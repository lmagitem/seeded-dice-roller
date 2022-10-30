#![warn(clippy::all, clippy::pedantic)]
use rand::prelude::*;
use rand_seeder::Seeder;
use rand_pcg::Pcg64;

/// An object that uses a Random Number Generator fed with a seed to generate dice roll results
/// in a deterministic way.
pub struct SeededDiceRoller {
    rng: Pcg64
}

impl SeededDiceRoller {
    /// Returns a generator initialized with the given seed and step.
    ///
    /// # Why two arguments
    /// The **seed** represents something like the "session" of the run, while the **step**
    /// represents the name of the task currently at hand.
    ///
    /// For example, if we want to generate a dungeon using the player-inputted **seed**
    /// "water temple", we might create three specific instances of **SeededDiceRoller** using
    /// "map_gen_shape", "map_gen_walls" and "map_gen_treasures" values for the **step** in order
    /// to always get the same results for that specific task, no matter how many other tasks
    /// you might add or remove before them in the future.
    ///
    /// It helps keeping seeded generation consistent between versions of your program.
    pub fn new(seed: &str, step: &str) -> Self {
        Self { rng: Seeder::from(format!("{}_{}", step, seed)).make_rng() }
    }

    /// Returns **true** or **false**.
    pub fn toss_coin(&mut self) -> bool {
        self.rng.gen::<bool>()
    }

    /// Rolls **dice** times a **die_type** sided die, adds an eventual **modifier** and returns
    /// the result.
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
        assert!(rng.roll(1, 6, 0) == 2);
        assert!(rng.roll(1, 20, 0) == 6);
        assert!(rng.roll(1, 6, -15) == -9);
        assert!(rng.roll(69, 6, 0) == 260);
        assert!(rng.roll(2, 123, 0) == 50);
        assert!(rng.roll(1, 6, 3343) == 3344);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == false);

        rng = SeededDiceRoller::new("other_seed", "test");
        assert!(rng.roll(1, 6, 0) == 2);
        assert!(rng.roll(3, 6, -5) == 2);
        assert!(rng.roll(1, 6, 0) == 5);
        assert!(rng.roll(1, 20, 0) == 18);
        assert!(rng.roll(1, 6, -15) == -12);
        assert!(rng.roll(69, 6, 0) == 234);
        assert!(rng.roll(2, 123, 0) == 57);
        assert!(rng.roll(1, 6, 3343) == 3344);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == true);

        rng = SeededDiceRoller::new("other_seed", "step");
        assert!(rng.roll(1, 6, 0) == 1);
        assert!(rng.roll(3, 6, -5) == 2);
        assert!(rng.roll(1, 6, 0) == 4);
        assert!(rng.roll(1, 20, 0) == 19);
        assert!(rng.roll(1, 6, -15) == -12);
        assert!(rng.roll(69, 6, 0) == 269);
        assert!(rng.roll(2, 123, 0) == 131);
        assert!(rng.roll(1, 6, 3343) == 3348);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == false);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == true);
        assert!(rng.toss_coin() == false);
    }
}
