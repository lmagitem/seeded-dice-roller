//! # Seeded Dice Roller
//! SeededDiceRoller is, as its name implies, a dice roller using a seed to give pseudo-random deterministic results.
//!
//! In other words, it returns "random" results, which will always be the same if you use the same seed and call the same methods in the
//! same order.
//!
//! ## How does it work
//! You generate a Dice Roller using a seed, then you can use it to make dice rolls, generate random numbers or booleans, or select a
//! specific result in an array of possible choices using either a predefined dice roll, a gaussian distribution or a simple pick at random.
//!
//! It is also possible to give weight to the various choices in order to multiply their chances to be selected.
//!
//! ## Seed
//! The seed is split into two parts, the **seed** proper and a "**step**". The **seed** represents something like the "session" of the run,
//! while the **step** represents the name of the task currently at hand. The idea is to keep seeded generation consistent between versions
//! of your program.
//!
//! For example, if we want to generate a dungeon using the player-inputted **seed** "water temple", we might create three specific
//! instances of **SeededDiceRoller** using "map_gen_shape", "map_gen_walls" and "map_gen_treasures" values for the **step** in order to
//! always get the same results for those specific tasks, no matter how many other tasks you might add or remove before them in the future.
//!
//! ## Examples
//! ### Dice rolls
//! ```rust
//! # use seeded_dice_roller::*;
//! #
//! # #[test]
//! # fn doc_test_dice_rolls() {
//! 	let mut rng = SeededDiceRoller::new("seed", "step");
//!
//!     assert_eq!(rng.roll(1, 6, 0), 6);
//!     assert_eq!(rng.roll(3, 6, -5), 1);
//!     assert_eq!(rng.roll(3, 6, -5), 8);
//! # }
//! ```
//!
//! ### Random picks
//! ###### Picks a result using a predefined roll type
//! ```rust
//! # use seeded_dice_roller::*;
//! #
//! # #[test]
//! # fn doc_test_dice_rolls() {
//! 	let mut rng = seeded_dice_roller::SeededDiceRoller::new("seed", "step");
//!
//! 	let possible_results = SeededDiceRoller::to_copyable_possible_results(vec![
//! 	    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
//! 	]);
//!     let result = rng.get_result(&CopyableRollToProcess {
//!                                     possible_results: possible_results.clone(),
//!                                     roll_method: RollMethod::PreparedRoll(PreparedRoll {
//!                                         dice: 2,
//!                                         die_type: 6,
//!                                         modifier: 0
//!                                     }),
//!                                 }).unwrap();
//!
//!     assert_eq!(result, "g");
//! # }
//! ```
//!
//! ###### Picks a result with higher chances to get one from the middle of the array
//! ```rust
//! # use seeded_dice_roller::*;
//! #
//! # #[test]
//! # fn doc_test_dice_rolls() {
//! 	let mut rng = seeded_dice_roller::SeededDiceRoller::new("seed", "step");
//!
//! 	let possible_results = SeededDiceRoller::to_copyable_possible_results(vec![
//! 	    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
//! 	]);
//!     let result = rng.get_result(&CopyableRollToProcess {
//!                                     possible_results: possible_results.clone(),
//!                                     roll_method: RollMethod::GaussianRoll(5),
//!                                 }).unwrap();
//!
//!     assert_eq!(result, "e");
//! # }
//! ```
//!
//! ###### Picks a result randomly, each choice has an equal chance to be selected
//! ```rust
//! # use seeded_dice_roller::*;
//! #
//! # #[test]
//! # fn doc_test_dice_rolls() {
//! 	let mut rng = seeded_dice_roller::SeededDiceRoller::new("seed", "step");
//!
//! 	let possible_results = SeededDiceRoller::to_copyable_possible_results(vec![
//! 	    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
//! 	]);
//!     let result = rng.get_result(&CopyableRollToProcess {
//!                                     possible_results: possible_results.clone(),
//!                                     roll_method: RollMethod::SimpleRoll,
//!                                 }).unwrap();
//!
//!     assert_eq!(result, "c");
//! # }
//! ```
//!
//! ###### Picks a result randomly, "a" is 5 times more likely to be selected than "b" or "c"
//! ```rust
//! # use seeded_dice_roller::*;
//! #
//! # #[test]
//! # fn doc_test_dice_rolls() {
//! 	let mut rng = seeded_dice_roller::SeededDiceRoller::new("seed", "step");
//!
//!     let weighted_set = vec![
//!         CopyableWeightedResult { result: "a", weight: 5 },
//!         CopyableWeightedResult { result: "b", weight: 1 },
//!         CopyableWeightedResult { result: "c", weight: 1 },
//!     ];
//!     let result = rng.get_result(&CopyableRollToProcess {
//!                                     possible_results: weighted_set,
//!                                     roll_method: RollMethod::SimpleRoll,
//!                                 }).unwrap();
//!
//!     assert_eq!(result, "c");
//! # }
//! ```

#![warn(clippy::all, clippy::pedantic)]
use log::*;
use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::fmt::Display;

/// Enum used to know how to determine the result of a random pick in a list of possible results.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, SmartDefault, Serialize, Deserialize,
)]
pub enum RollMethod {
    /// Uses a prepared roll ("**dice** D **die_type** + **modifier**").
    PreparedRoll(PreparedRoll),
    /// Uses the given number of dice in order to pick a random result, with increasingly higher
    /// chances to get one from the middle of the list as the value is high.
    GaussianRoll(u16),
    /// Simply rolls against the number of possible results to get a random one.
    #[default]
    SimpleRoll,
}

impl Display for RollMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RollMethod::PreparedRoll(roll) => write!(f, "PreparedRoll({})", roll),
            RollMethod::GaussianRoll(n) => write!(f, "GaussianRoll({})", n),
            RollMethod::SimpleRoll => write!(f, "SimpleRoll"),
        }
    }
}

/// Data allowing to pick a result at random in a list of possible results.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RollToProcess<T> {
    /// A list of possible results that can be picked at random.
    pub possible_results: Vec<WeightedResult<T>>,
    /// The method with which to pick a desired result.
    pub roll_method: RollMethod,
}

impl<T> Display for RollToProcess<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RollToProcess {{ {} choices, method: {} }}",
            self.possible_results.len(),
            self.roll_method
        )
    }
}

/// Data allowing to pick a result at random in a list of possible results. The results must
/// be copyable.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CopyableRollToProcess<T>
where
    T: Copy + std::fmt::Debug,
{
    /// A list of possible results that can be picked at random.
    pub possible_results: Vec<CopyableWeightedResult<T>>,
    /// The method with which to pick a desired result.
    pub roll_method: RollMethod,
}

impl<T: Copy + std::fmt::Debug> Display for CopyableRollToProcess<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CopyableRollToProcess {{ choices: {:?}, method: {} }}",
            self.possible_results,
            self.roll_method
        )
    }
}

/// A result able to be picked at random in a list of possible results. The **weight** is used
/// to determine the chances of this result to be picked against all other possible choices.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WeightedResult<T> {
    /// The result that can be selected at random.
    pub result: T,
    /// The eventual weight of this result. A higher weight means that the result will be more
    /// likely to be picked in an uniform distribution.
    ///
    /// Or with an example: when using the SimpleRoll [RollMethod], an item with a weight of 5
    /// will have 5 more chances to be selected than an item with a weight of one;
    pub weight: u32,
}

impl<T> Display for WeightedResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ result, weight: {} }}", self.weight)
    }
}

/// A result able to be picked at random in a list of possible results. The **weight** is used
/// to determine the chances of this result to be picked against all other possible choices.
/// The result must be copyable.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct CopyableWeightedResult<T>
where
    T: Copy + std::fmt::Debug,
{
    /// The result that can be selected at random.
    pub result: T,
    /// The eventual weight of this result. A higher weight means that the result will be more
    /// likely to be picked in an uniform distribution.
    ///
    /// Or with an example: when using the SimpleRoll [RollMethod], an item with a weight of 5
    /// will have 5 more chances to be selected than an item with a weight of one;
    pub weight: u32,
}

impl<T: Copy + std::fmt::Debug> Display for CopyableWeightedResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ result: {:?}, weight: {} }}",
            self.result, self.weight
        )
    }
}

/// Data allowing to roll **dice** times a **die_type** sided die and add an eventual **modifier**.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct PreparedRoll {
    /// The number of dice to roll.
    pub dice: u16,
    /// The type of dice to roll. It doesn't need to be a "real one". Example: 6-sided dice,
    /// 20-sided dice, 15694-sided dice...
    pub die_type: u32,
    /// An eventual modifier to apply to the roll's result.
    pub modifier: i32,
}

impl Default for PreparedRoll {
    fn default() -> Self {
        Self {
            dice: 1,
            die_type: 6,
            modifier: 0,
        }
    }
}

impl Display for PreparedRoll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}d{}+({})", self.dice, self.die_type, self.modifier)
    }
}

/// A temporary struct used for finding which result a dice roll returns.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
struct RangedResult {
    /// The index of the result this struct represents.
    pub result_index: usize,
    /// The roll must be equal or greater that **min** value. Inclusive.
    pub min: i64,
    /// The roll must be lower that **max** value. Non-inclusive.
    pub max: i64,
}

impl Display for RangedResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ result_index: {}, min: {}, max: {} }}",
            self.result_index, self.min, self.max
        )
    }
}

/// Uses a Random Number Generator fed with a **seed** to generate dice roll results, booleans
/// and numbers in a deterministic way.
///
/// It ensures that as long as you ask for the same rolls or generate the same types, you will
/// always get the same results in the same order for a given **seed** and **step**.
#[derive(Clone, Debug)]
pub struct SeededDiceRoller {
    /// The seeded random generator.
    rng: Pcg64,
}

impl Default for SeededDiceRoller {
    fn default() -> Self {
        Self {
            rng: Seeder::from("seed".to_string()).make_rng(),
        }
    }
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
        let gen = self.rng.gen::<bool>();
        trace!("gen_bool: {}", gen);
        gen
    }

    /// Returns a random 8bit unsigned integer.
    pub fn gen_u8(&mut self) -> u8 {
        let gen = self.rng.gen::<u8>();
        trace!("gen_u8: {}", gen);
        gen
    }

    /// Returns a random 16bit unsigned integer.
    pub fn gen_u16(&mut self) -> u16 {
        let gen = self.rng.gen::<u16>();
        trace!("gen_u16: {}", gen);
        gen
    }

    /// Returns a random 32bit unsigned integer.
    pub fn gen_u32(&mut self) -> u32 {
        let gen = self.rng.gen::<u32>();
        trace!("gen_u32: {}", gen);
        gen
    }

    /// Returns a random 64bit unsigned integer.
    pub fn gen_u64(&mut self) -> u64 {
        let gen = self.rng.gen::<u64>();
        trace!("gen_u64: {}", gen);
        gen
    }

    /// Returns a random 128bit unsigned integer.
    pub fn gen_u128(&mut self) -> u128 {
        let gen = self.rng.gen::<u128>();
        trace!("gen_u128: {}", gen);
        gen
    }

    /// Returns a random pointer-sized unsigned integer.
    pub fn gen_usize(&mut self) -> usize {
        let gen = self.rng.gen::<usize>();
        trace!("gen_usize: {}", gen);
        gen
    }

    /// Returns a random 8bit signed integer.
    pub fn gen_i8(&mut self) -> i8 {
        let gen = self.rng.gen::<i8>();
        trace!("gen_i8: {}", gen);
        gen
    }

    /// Returns a random 16bit signed integer.
    pub fn gen_i16(&mut self) -> i16 {
        let gen = self.rng.gen::<i16>();
        trace!("gen_i16: {}", gen);
        gen
    }

    /// Returns a random 32bit signed integer.
    pub fn gen_i32(&mut self) -> i32 {
        let gen = self.rng.gen::<i32>();
        trace!("gen_i32: {}", gen);
        gen
    }

    /// Returns a random 64bit signed integer.
    pub fn gen_i64(&mut self) -> i64 {
        let gen = self.rng.gen::<i64>();
        trace!("gen_i64: {}", gen);
        gen
    }

    /// Returns a random 128bit signed integer.
    pub fn gen_i128(&mut self) -> i128 {
        let gen = self.rng.gen::<i128>();
        trace!("gen_i128: {}", gen);
        gen
    }

    /// Returns a random pointer-sized signed integer.
    pub fn gen_isize(&mut self) -> isize {
        let gen = self.rng.gen::<isize>();
        trace!("gen_isize: {}", gen);
        gen
    }

    /// Returns a random 32bit floating point type.
    pub fn gen_f32(&mut self) -> f32 {
        let gen = self.rng.gen::<f32>();
        trace!("gen_f32: {}", gen);
        gen
    }

    /// Returns a random 64bit floating point type.
    pub fn gen_f64(&mut self) -> f64 {
        let gen = self.rng.gen::<f64>();
        trace!("gen_f64: {}", gen);
        gen
    }

    /// Rolls **dice** times a **die_type** sided die, adds an eventual **modifier** and returns
    /// the result.
    pub fn roll(&mut self, dice: u16, die_type: u32, modifier: i32) -> i64 {
        let mut result = 0;
        let die_type = die_type as i64;
        for _ in 0..dice {
            result += (self.rng.gen::<u32>() as i64).abs() % &die_type + 1;
        }
        result += modifier as i64;

        trace!("roll: {}d{}+({}) = {}", dice, die_type, modifier, result);
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
    pub fn get_result<T: Copy + std::fmt::Debug>(
        &mut self,
        to_process: &CopyableRollToProcess<T>,
    ) -> Option<T> {
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
                    let result = self.process_prepared_roll(&to_process, &length, &roll);
                    result
                }
                RollMethod::GaussianRoll(dice) => {
                    let result = self.process_gaussian_roll(&to_process, &length, &dice);
                    result
                }
                RollMethod::SimpleRoll => {
                    let result = self.process_simple_roll(&to_process, &length);
                    result
                }
            },
        }
    }

    /// Picks a result using the [PreparedRoll] stored alongside a list **to_process**.
    fn process_prepared_roll<T>(
        &mut self,
        to_process: &RollToProcess<T>,
        length: &usize,
        prepared_roll: &PreparedRoll,
    ) -> Option<usize> {
        // Transform the array so it can be used easily
        let mut choices: Vec<RangedResult> = Vec::new();
        self.fill_choices(
            &to_process,
            &length,
            &(prepared_roll.clone().dice as i64),
            &1,
            &mut choices,
        );

        let roll = self.roll_prepared(prepared_roll);
        let result = choices.iter().find(|r| roll >= r.min && roll < r.max);
        trace!("chosen: {:?}", result);

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
        dice: &u16,
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

        let max = SeededDiceRoller::calculate_die_type(to_process);
        // Adds a modifier to avoid getting results skewed towards the beginning or the end of the set
        let modifier = (dice / 2) as i32
            + (if (dice % 2 == 0) && self.rng.gen::<bool>() {
                -1
            } else {
                0
            });
        let roll = self.roll(*dice, max, modifier);
        let result = choices.iter().find(|r| roll >= r.min && roll < r.max);
        trace!("chosen: {:?}", result);

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

        let max = SeededDiceRoller::calculate_die_type(to_process);
        let roll = self.roll(1, max, 0);
        let result = choices.iter().find(|r| roll >= r.min && roll < r.max);
        trace!("chosen: {:?}", result);

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
        trace!("choices: {:?}", choices);
    }

    /// Returns a vector of [CopyableWeightedResult] using the given **vec** of values.
    /// The result can be used in a [CopyableRollToProcess].
    pub fn to_copyable_possible_results<T: Copy + std::fmt::Debug>(
        vec: Vec<T>,
    ) -> Vec<CopyableWeightedResult<T>> {
        vec.iter()
            .map(|c| CopyableWeightedResult {
                result: c.clone(),
                weight: 1,
            })
            .collect()
    }

    /// Returns a vector of [WeightedResult] using the given **vec** of values.
    /// The result can be used in a [RollToProcess].
    pub fn to_possible_results<T>(vec: Vec<T>) -> Vec<WeightedResult<T>> {
        let mut result = Vec::new();
        for item in vec {
            result.push(WeightedResult {
                result: item,
                weight: 1,
            });
        }
        result
    }

    /// Adds the weight of every entry in a list **to_process** in order to determine the type
    /// of die that must be rolled to find a desired result.
    fn calculate_die_type<T>(to_process: &RollToProcess<T>) -> u32 {
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
    fn dice_roller_is_deterministic() {
        let mut rng = SeededDiceRoller::new("seed", "test");
        {
            assert!(rng.roll(1, 6, 0) == 2);
            assert!(rng.roll(3, 6, -5) == 2);
            assert!(rng.roll(1, 6, 0) == 2);
            assert!(rng.roll(1, 20, 0) == 6);
            assert!(rng.roll(1, 6, -15) == -13);
            assert!(rng.roll(69, 6, 0) == 242);
            assert!(rng.roll(2, 123, 0) == 81);
            assert!(rng.roll(1, 6, 3343) == 3348);
            assert!(rng.gen_bool() == false);
            assert!(rng.gen_u8() == 188);
            assert!(rng.gen_u16() == 45209);
            assert!(rng.gen_u32() == 2067204665);
            assert!(rng.gen_u64() == 11144615613207554777);
            assert!(rng.gen_u128() == 326911416680500363065339602289182768569);
            assert!(rng.gen_usize() == 8269465146262660349);
            assert!(rng.gen_i8() == 83);
            assert!(rng.gen_i16() == 3067);
            assert!(rng.gen_i32() == -1171247657);
            assert!(rng.gen_i64() == 9108059218017983344);
            assert!(rng.gen_i128() == 146530613037906103918089470235257735612);
            assert!(rng.gen_isize() == 2479790373172492566);
            assert!(rng.gen_f32() == 0.9228384);
            assert!(rng.gen_f64() == 0.8631162799734914);

            assert!("j".eq(rng
                .get_result(&CopyableRollToProcess {
                    possible_results: SeededDiceRoller::to_copyable_possible_results(vec![
                        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
                    ]),
                    roll_method: RollMethod::SimpleRoll,
                })
                .unwrap()));
        }
        rng = SeededDiceRoller::new("other_seed", "test");
        {
            assert!(rng.roll(1, 6, 0) == 6);
            assert!(rng.roll(3, 6, -5) == 10);
            assert!(rng.roll(1, 6, 0) == 3);
            assert!(rng.roll(1, 20, 0) == 10);
            assert!(rng.roll(1, 6, -15) == -12);
            assert!(rng.roll(69, 6, 0) == 240);
            assert!(rng.roll(2, 123, 0) == 138);
            assert!(rng.roll(1, 6, 3343) == 3344);
            assert!(rng.gen_bool() == true);
            assert!(rng.gen_u8() == 82);
            assert!(rng.gen_u16() == 27159);
            assert!(rng.gen_u32() == 3180098725);
            assert!(rng.gen_u64() == 11552742574431662508);
            assert!(rng.gen_u128() == 196627661076901716217737966822153854526);
            assert!(rng.gen_usize() == 1997277166238086139);
            assert!(rng.gen_i8() == 45);
            assert!(rng.gen_i16() == -22194);
            assert!(rng.gen_i32() == -1765316073);
            assert!(rng.gen_i64() == 7982030035740135755);
            assert!(rng.gen_i128() == 130008835046757806841833196450514227059);
            assert!(rng.gen_isize() == -3501112453948772746);
            assert!(rng.gen_f32() == 0.9940159);
            assert!(rng.gen_f64() == 0.45617011270821706);

            assert!("k".eq(rng
                .get_result(&CopyableRollToProcess {
                    possible_results: SeededDiceRoller::to_copyable_possible_results(vec![
                        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
                    ]),
                    roll_method: RollMethod::SimpleRoll,
                })
                .unwrap()));
        }
        rng = SeededDiceRoller::new("other_seed", "step");
        {
            assert!(rng.roll(1, 6, 0) == 1);
            assert!(rng.roll(3, 6, -5) == 4);
            assert!(rng.roll(1, 6, 0) == 6);
            assert!(rng.roll(1, 20, 0) == 15);
            assert!(rng.roll(1, 6, -15) == -12);
            assert!(rng.roll(69, 6, 0) == 239);
            assert!(rng.roll(2, 123, 0) == 91);
            assert!(rng.roll(1, 6, 3343) == 3344);
            assert!(rng.gen_bool() == false);
            assert!(rng.gen_u8() == 162);
            assert!(rng.gen_u16() == 34315);
            assert!(rng.gen_u32() == 2687893072);
            assert!(rng.gen_u64() == 10068043339616645489);
            assert!(rng.gen_u128() == 78293060686096732239048405502667573500);
            assert!(rng.gen_usize() == 15847822118157400675);
            assert!(rng.gen_i8() == -83);
            assert!(rng.gen_i16() == 683);
            assert!(rng.gen_i32() == -585801794);
            assert!(rng.gen_i64() == -1818745056280883793);
            assert!(rng.gen_i128() == 162224135727382922470578647495526568637);
            assert!(rng.gen_isize() == -6539258215208328255);
            assert!(rng.gen_f32() == 0.6179796);
            assert!(rng.gen_f64() == 0.22569667223081946);

            assert!("g".eq(rng
                .get_result(&CopyableRollToProcess {
                    possible_results: SeededDiceRoller::to_copyable_possible_results(vec![
                        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
                    ]),
                    roll_method: RollMethod::SimpleRoll,
                })
                .unwrap()));
        }
    }

    #[test]
    fn roll_is_within_bounds() {
        let mut rng = SeededDiceRoller::new("seed", "test");
        for _ in 0..1000 {
            let n: i64 = rng.roll(1, 6, 0);
            assert!(n >= 1 && n <= 6);
        }
        for _ in 0..1000 {
            let n: i64 = rng.roll(3, 6, 0);
            assert!(n >= 3 && n <= 18);
        }
        for _ in 0..1000 {
            let n: i64 = rng.roll(1, 20, 0);
            assert!(n >= 1 && n <= 20);
        }
    }

    #[test]
    fn roll_prepared_gives_same_results_as_roll() {
        let mut rng_one = SeededDiceRoller::new("seed", "test");
        let mut rng_two = SeededDiceRoller::new("seed", "test");
        let mut n_one: i64 = rng_one.roll(1, 6, 0);
        let mut n_two: i64 = rng_two.roll_prepared(&PreparedRoll {
            dice: 1,
            die_type: 6,
            modifier: 0,
        });
        assert!(n_one == n_two);
        n_one = rng_one.roll(3, 6, -4);
        n_two = rng_two.roll_prepared(&PreparedRoll {
            dice: 3,
            die_type: 6,
            modifier: -4,
        });
        assert!(n_one == n_two);
        n_one = rng_one.roll(20, 100, -444);
        n_two = rng_two.roll_prepared(&PreparedRoll {
            dice: 20,
            die_type: 100,
            modifier: -444,
        });
        assert!(n_one == n_two);
    }

    #[test]
    fn get_result_returns_a_random_result() {
        let mut rng = SeededDiceRoller::new("seed", "test");
        for _ in 0..1000 {
            assert!(vec!["a", "b", "c", "d"].contains(
                &rng.get_result(&CopyableRollToProcess {
                    possible_results: SeededDiceRoller::to_copyable_possible_results(vec![
                        "a", "b", "c", "d"
                    ]),
                    roll_method: RollMethod::PreparedRoll(PreparedRoll {
                        dice: 2,
                        die_type: 6,
                        modifier: -3
                    }),
                })
                .unwrap()
            ));
        }
        for _ in 0..1000 {
            assert!(vec!["a", "b", "c", "d"].contains(
                &rng.get_result(&CopyableRollToProcess {
                    possible_results: SeededDiceRoller::to_copyable_possible_results(vec![
                        "a", "b", "c", "d"
                    ]),
                    roll_method: RollMethod::GaussianRoll(4),
                })
                .unwrap()
            ));
        }
        for _ in 0..1000 {
            assert!(vec!["a", "b", "c", "d"].contains(
                &rng.get_result(&CopyableRollToProcess {
                    possible_results: SeededDiceRoller::to_copyable_possible_results(vec![
                        "a", "b", "c", "d"
                    ]),
                    roll_method: RollMethod::SimpleRoll,
                })
                .unwrap()
            ));
        }
    }

    #[test]
    fn get_result_doesnt_fail_out_of_bounds() {
        let mut rng = SeededDiceRoller::new("seed", "test");
        assert!("a".eq(rng
            .get_result(&CopyableRollToProcess {
                possible_results: SeededDiceRoller::to_copyable_possible_results(vec!["a", "b"]),
                roll_method: RollMethod::PreparedRoll(PreparedRoll {
                    dice: 0,
                    die_type: 0,
                    modifier: i32::MIN
                }),
            })
            .unwrap()));
        assert!("b".eq(rng
            .get_result(&CopyableRollToProcess {
                possible_results: SeededDiceRoller::to_copyable_possible_results(vec!["a", "b"]),
                roll_method: RollMethod::PreparedRoll(PreparedRoll {
                    dice: u16::MAX,
                    die_type: u32::MAX,
                    modifier: i32::MAX
                }),
            })
            .unwrap()));
    }

    #[test]
    fn get_result_index_returns_a_random_index() {
        let mut rng = SeededDiceRoller::new("seed", "test");
        for _ in 0..1000 {
            assert!(vec![0, 1, 2, 3, 4, 5, 6, 7].contains(
                &rng.get_result_index(&RollToProcess {
                    possible_results: SeededDiceRoller::to_possible_results(vec![
                        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
                    ]),
                    roll_method: RollMethod::PreparedRoll(PreparedRoll {
                        dice: 2,
                        die_type: 6,
                        modifier: -3
                    }),
                })
                .unwrap()
            ));
        }
        for _ in 0..1000 {
            assert!(vec![0, 1, 2, 3].contains(
                &rng.get_result_index(&RollToProcess {
                    possible_results: SeededDiceRoller::to_possible_results(vec![
                        "a", "b", "c", "d"
                    ]),
                    roll_method: RollMethod::GaussianRoll(3),
                })
                .unwrap()
            ));
        }
        for _ in 0..1000 {
            assert!(vec![0, 1, 2, 3].contains(
                &rng.get_result_index(&RollToProcess {
                    possible_results: SeededDiceRoller::to_possible_results(vec![
                        "a", "b", "c", "d"
                    ]),
                    roll_method: RollMethod::SimpleRoll,
                })
                .unwrap()
            ));
        }
    }
}
