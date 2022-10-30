#![warn(clippy::all, clippy::pedantic)]
use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

/// Enum used to know how to determine the result of a random pick in a list of possible results.
#[derive(Debug, Clone, Copy)]
pub enum RollMethod {
    /// Uses a prepared roll ("**dice** D **die_type** + **modifier**").
    PreparedRoll(PreparedRoll),
    /// Uses the given number of dice in order to pick a random result, with increasingly higher
    /// chances to get one from the middle of the list as the value is high.
    GaussianRoll(usize),
    /// Simply rolls against the number of possible results to get a random one.
    SimpleRoll,
}

/// Data allowing to pick a result at random in a list of possible results.
#[derive(Debug)]
pub struct RollToProcess<T> {
    /// A list of possible results that can be picked at random. Must contain less than 65535 items.
    possible_results: Vec<WeightedResult<T>>,
    /// The method with which to pick a desired result.
    roll_method: RollMethod,
}

/// Data allowing to pick a result at random in a list of possible results. The results must
/// be copyable.
#[derive(Debug, Clone)]
pub struct CopyableRollToProcess<T>
where
    T: Copy,
{
    /// A list of possible results that can be picked at random. Must contain less than 65535 items.
    possible_results: Vec<CopyableWeightedResult<T>>,
    /// The method with which to pick a desired result.
    roll_method: RollMethod,
}

/// A result able to be picked at random in a list of possible results. The **weight** is used
/// to determine the chances of this result to be picked against all other possible choices.
#[derive(Debug)]
pub struct WeightedResult<T> {
    /// The result that can be selected at random.
    result: T,
    /// The eventual weight of this result. A higher weight means that the result will be more
    /// likely to be picked in an uniform distribution.
    ///
    /// Or with an example: when using the SimpleRoll [RollMethod], an item with a weight of 5
    /// will have 5 more chances to be selected than an item with a weight of one;
    weight: usize,
}

/// A result able to be picked at random in a list of possible results. The **weight** is used
/// to determine the chances of this result to be picked against all other possible choices.
/// The result must be copyable.
#[derive(Debug, Clone, Copy)]
pub struct CopyableWeightedResult<T>
where
    T: Copy,
{
    /// The result that can be selected at random.
    result: T,
    /// The eventual weight of this result. A higher weight means that the result will be more
    /// likely to be picked in an uniform distribution.
    ///
    /// Or with an example: when using the SimpleRoll [RollMethod], an item with a weight of 5
    /// will have 5 more chances to be selected than an item with a weight of one;
    weight: usize,
}

/// Data allowing to roll **dice** times a **die_type** sided die and add an eventual **modifier**.
#[derive(Debug, Clone, Copy)]
pub struct PreparedRoll {
    /// The number of dice to roll.
    dice: usize,
    /// The type of dice to roll. It doesn't need to be a "real one". Example: 6-sided dice,
    /// 20-sided dice, 15694-sided dice...
    die_type: usize,
    /// An eventual modifier to apply to the roll's result.
    modifier: i32,
}

/// A temporary struct used for finding which result a dice roll returns.
#[derive(Debug, Clone, Copy)]
struct RangedResult {
    /// The index of the result this struct represents.
    result_index: usize,
    /// The roll must be equal or greater that **min** value. Inclusive.
    min: i64,
    /// The roll must be lower that **max** value. Non-inclusive.
    max: i64,
}

/// Uses a Random Number Generator fed with a **seed** to generate dice roll results, booleans
/// and numbers in a deterministic way.
///
/// It ensures that as long as you ask for the same rolls or generate the same types, you will
/// always get the same results in the same order for a given **seed** and **step**.
pub struct SeededDiceRoller {
    /// The seeded random generator.
    rng: Pcg64,
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
        Self {
            rng: Seeder::from(format!("{}_{}", step, seed)).make_rng(),
        }
    }

    /// Returns **true** or **false**.
    pub fn gen_bool(&mut self) -> bool {
        self.rng.gen::<bool>()
    }

    /// Rolls **dice** times a **die_type** sided die, adds an eventual **modifier** and returns
    /// the result.
    pub fn roll(&mut self, dice: usize, die_type: usize, modifier: i32) -> i64 {
        let mut result = 0;
        let die_type = die_type as i64;
        for _ in 0..dice {
            result += (self.rng.gen::<usize>() as i64).abs() % &die_type + 1;
        }
        result += modifier as i64;
        result
    }

    /// Rolls **dice** times a **die_type** sided die, adds an eventual **modifier** and returns
    /// the result.
    pub fn roll_prepared(&mut self, to_roll: &PreparedRoll) -> i64 {
        self.roll(to_roll.dice, to_roll.die_type, to_roll.modifier)
    }

    /// Returns the result of a random selection in a **to_process** list given alongside the
    /// details of the selection method. That method can either be to follow the rules dictated
    /// in a [PreparedRoll] or by using a uniform or normal distribution.
    pub fn get_result<T: Copy>(&mut self, to_process: &CopyableRollToProcess<T>) -> Option<T> {
        let weighted_possible_results = (&to_process)
            .possible_results
            .iter()
            .map(|e| WeightedResult {
                result: e.result,
                weight: e.weight,
            })
            .collect();
        let standard_to_process = RollToProcess {
            possible_results: weighted_possible_results,
            roll_method: to_process.roll_method,
        };
        match self.get_result_index(&standard_to_process) {
            Some(index) => Some((&to_process.possible_results[index]).result),
            None => None,
        }
    }

    /// Returns the index of the result of a random selection in a **to_process** list given
    /// alongside the details of the selection method. That method can either be to follow the rules
    /// dictated in a [PreparedRoll] or by using a uniform or normal distribution.
    pub fn get_result_index<T>(&mut self, to_process: &RollToProcess<T>) -> Option<usize> {
        let length = to_process.possible_results.len();
        match length {
            0 => None,
            1 => Some(0),
            _ => match &to_process.roll_method {
                RollMethod::PreparedRoll(ref roll) => {
                    self.process_prepared_roll(&to_process, &length, &roll)
                }
                RollMethod::GaussianRoll(dice) => {
                    self.process_gaussian_roll(&to_process, &length, &dice)
                }
                RollMethod::SimpleRoll => self.process_simple_roll(&to_process, &length),
            },
        }
    }

    /// Picks a result using the [PreparedRoll] stored alongside a list **to_process**.
    fn process_prepared_roll<T>(
        &mut self,
        to_process: &RollToProcess<T>,
        length: &usize,
        roll: &PreparedRoll,
    ) -> Option<usize> {
        // Transform the array so it can be used easily
        let mut choices: Vec<RangedResult> = Vec::new();
        self.fill_choices(
            &to_process,
            &length,
            &(roll.clone().dice as i64),
            &1,
            &mut choices,
        );

        let roll = self.roll_prepared(roll);
        let result = choices.iter().find(|r| roll >= r.min && roll < r.max);

        match result {
            Some(ranged_result) => Some(ranged_result.result_index),
            None => None,
        }
    }

    /// Picks a result from a list **to_process** using multiple dice in order to get
    /// a normal distribution of the probabilities for each possible choice.
    fn process_gaussian_roll<T>(
        &mut self,
        to_process: &RollToProcess<T>,
        length: &usize,
        dice: &usize,
    ) -> Option<usize> {
        // Transform the array so it can be used easily
        let mut choices: Vec<RangedResult> = Vec::new();
        self.fill_choices(
            &to_process,
            &length,
            &(dice.clone() as i64),
            &(dice.clone() as i64),
            &mut choices,
        );

        let max = self.calculate_die_type(to_process);
        let roll = self.roll(*dice, max, 0);
        let result = choices.iter().find(|r| roll >= r.min && roll < r.max);

        match result {
            Some(ranged_result) => Some(ranged_result.result_index),
            None => None,
        }
    }

    /// Picks a result from a list **to_process** at random while respecting the required weight
    /// of each entry.
    fn process_simple_roll<T>(
        &mut self,
        to_process: &RollToProcess<T>,
        length: &usize,
    ) -> Option<usize> {
        // Transform the array so it can be used easily
        let mut choices: Vec<RangedResult> = Vec::new();
        self.fill_choices(&to_process, &length, &1, &1, &mut choices);

        let max = self.calculate_die_type(to_process);
        let roll = self.roll(1, max, 0);
        let result = choices.iter().find(|r| roll >= r.min && roll < r.max);

        match result {
            Some(ranged_result) => Some(ranged_result.result_index),
            None => None,
        }
    }

    /// Fills a **choices** vector with values allowing to elect a result from a die roll
    /// in a **to_process** list easily.
    fn fill_choices<T>(
        &self,
        to_process: &RollToProcess<T>,
        length: &usize,
        min: &i64,
        weight_multiplier: &i64,
        choices: &mut Vec<RangedResult>,
    ) {
        let mut min = min.clone();
        let mut last_end: i64 = min;
        for i in 0..*length {
            let weight: i64 = ((&to_process.possible_results[i]).weight as i64) * weight_multiplier;
            min = if i == 0 { i64::MIN } else { last_end.clone() };
            last_end += weight;
            let max: i64 = if i == length - 1 {
                i64::MAX
            } else {
                last_end.clone()
            };
            choices.push(RangedResult {
                result_index: i as usize,
                min,
                max,
            })
        }
    }

    /// Adds the weight of every entry in a list **to_process** in order to determine the type
    /// of die that must be rolled to find a desired result.
    fn calculate_die_type<T>(&self, to_process: &RollToProcess<T>) -> usize {
        let max = to_process
            .possible_results
            .iter()
            .map(|r| r.weight)
            .reduce(|a, b| a + b)
            .expect("Should be able to add the possible results' weights.");
        max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_is_within_bounds() {
        let mut rng = SeededDiceRoller::new("seed", "test");
        for _ in 0..1000 {
            let n: i64 = rng.roll(1, 6, 0);
            assert!(n >= 1);
            assert!(n <= 6);
        }
        for _ in 0..1000 {
            let n: i64 = rng.roll(3, 6, 0);
            assert!(n >= 3);
            assert!(n <= 18);
        }
        for _ in 0..1000 {
            let n: i64 = rng.roll(1, 20, 0);
            assert!(n >= 1);
            assert!(n <= 20);
        }
    }

    #[test]
    fn generator_is_deterministic() {
        let mut rng = SeededDiceRoller::new("seed", "test");
        assert!(rng.roll(1, 6, 0) == 2);
        assert!(rng.roll(3, 6, -5) == 8);
        assert!(rng.roll(1, 6, 0) == 2);
        assert!(rng.roll(1, 20, 0) == 4);
        assert!(rng.roll(1, 6, -15) == -9);
        assert!(rng.roll(69, 6, 0) == 246);
        assert!(rng.roll(2, 123, 0) == 101);
        assert!(rng.roll(1, 6, 3343) == 3346);
        assert!(rng.gen_bool() == false);
        assert!(rng.gen_bool() == true);
        assert!(rng.gen_bool() == true);
        assert!(rng.gen_bool() == false);
        assert!(rng.gen_bool() == false);

        rng = SeededDiceRoller::new("other_seed", "test");
        assert!(rng.roll(1, 6, 0) == 4);
        assert!(rng.roll(3, 6, -5) == 4);
        assert!(rng.roll(1, 6, 0) == 3);
        assert!(rng.roll(1, 20, 0) == 4);
        assert!(rng.roll(1, 6, -15) == -10);
        assert!(rng.roll(69, 6, 0) == 244);
        assert!(rng.roll(2, 123, 0) == 171);
        assert!(rng.roll(1, 6, 3343) == 3348);
        assert!(rng.gen_bool() == true);
        assert!(rng.gen_bool() == true);
        assert!(rng.gen_bool() == true);
        assert!(rng.gen_bool() == true);
        assert!(rng.gen_bool() == true);

        rng = SeededDiceRoller::new("other_seed", "step");
        assert!(rng.roll(1, 6, 0) == 5);
        assert!(rng.roll(3, 6, -5) == 4);
        assert!(rng.roll(1, 6, 0) == 6);
        assert!(rng.roll(1, 20, 0) == 19);
        assert!(rng.roll(1, 6, -15) == -10);
        assert!(rng.roll(69, 6, 0) == 209);
        assert!(rng.roll(2, 123, 0) == 106);
        assert!(rng.roll(1, 6, 3343) == 3346);
        assert!(rng.gen_bool() == false);
        assert!(rng.gen_bool() == false);
        assert!(rng.gen_bool() == true);
        assert!(rng.gen_bool() == true);
        assert!(rng.gen_bool() == false);
    }
}
