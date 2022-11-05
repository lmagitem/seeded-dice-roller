# Seeded Dice Roller
SeededDiceRoller is, as its name implies, a dice roller using a seed to give pseudo-random deterministic results.

In other words, it returns "random" results, which will always be the same if you use the same seed and call the same methods in the same order.

## How does it work
You generate a Dice Roller using a seed, then you can use it to make dice rolls, generate random numbers or booleans, or select a specific result in an array of possible choices using either a predefined dice roll, a gaussian distribution or a simple pick at random.

It is also possible to give weight to the various choices in order to multiply their chances to be selected.

### Seed
The seed is split into two parts, the **seed** proper and a "**step**". The **seed** represents something like the "session" of the run, while the **step** represents the name of the task currently at hand. The idea is to keep seeded generation consistent between versions of your program.

For example, if we want to generate a dungeon using the player-inputted **seed** "water temple", we might create three specific instances of **SeededDiceRoller** using "map_gen_shape", "map_gen_walls" and "map_gen_treasures" values for the **step** in order to always get the same results for those specific tasks, no matter how many other tasks you might add or remove before them in the future.

## Examples
### Dice rolls
```rust
use seeded_dice_roller::*;

#[test]
fn doc_test_dice_rolls() {
     let mut rng = SeededDiceRoller::new("seed", "step");

    assert_eq!(rng.roll(1, 6, 0), 6);
    assert_eq!(rng.roll(3, 6, -5), 1);
    assert_eq!(rng.roll(3, 6, -5), 8);
}
```


### Random picks
###### Picks a result using a predefined roll type
```rust
use seeded_dice_roller::*;

#[test]
fn doc_test_dice_rolls() {
    let mut rng = seeded_dice_roller::SeededDiceRoller::new("seed", "step");

    let possible_results = SeededDiceRoller::to_copyable_possible_results(vec![
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
    ]);
    let result = rng.get_result(&CopyableRollToProcess {
                                    possible_results: possible_results.clone(),
                                    roll_method: RollMethod::PreparedRoll(PreparedRoll {
                                        dice: 2,
                                        die_type: 6,
                                        modifier: 0
                                    }),
                                }).unwrap();

    assert_eq!(result, "g");
}
```

###### Picks a result with higher chances to get one from the middle of the array
```rust
use seeded_dice_roller::*;

#[test]
fn doc_test_dice_rolls() {
    let mut rng = seeded_dice_roller::SeededDiceRoller::new("seed", "step");

    let possible_results = SeededDiceRoller::to_copyable_possible_results(vec![
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
    ]);
    let result = rng.get_result(&CopyableRollToProcess {
                                    possible_results: possible_results.clone(),
                                    roll_method: RollMethod::GaussianRoll(5),
                                }).unwrap();

    assert_eq!(result, "e");
}
```

###### Picks a result randomly, each choice has an equal chance to be selected
```rust
use seeded_dice_roller::*;

#[test]
fn doc_test_dice_rolls() {
    let mut rng = seeded_dice_roller::SeededDiceRoller::new("seed", "step");

    let possible_results = SeededDiceRoller::to_copyable_possible_results(vec![
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"
    ]);
    let result = rng.get_result(&CopyableRollToProcess {
                                    possible_results: possible_results.clone(),
                                    roll_method: RollMethod::SimpleRoll,
                                }).unwrap();

    assert_eq!(result, "c");
}
```

###### Picks a result randomly, "a" is 5 times more likely to be selected than "b" or "c"
```rust
use seeded_dice_roller::*;

#[test]
fn doc_test_dice_rolls() {
    let mut rng = seeded_dice_roller::SeededDiceRoller::new("seed", "step");

    let weighted_set = vec![
        CopyableWeightedResult { result: "a", weight: 5 },
        CopyableWeightedResult { result: "b", weight: 1 },
        CopyableWeightedResult { result: "c", weight: 1 },
    ];
    let result = rng.get_result(&CopyableRollToProcess {
                                    possible_results: weighted_set,
                                    roll_method: RollMethod::SimpleRoll,
                                }).unwrap();

    assert_eq!(result, "c");
}
```

## Contribute
I'll be happy to receive issues asking for new features or bug fixes. Also feel free to point out where code could be improved (either in performance, readability, documentation, following best practices...) and/or make pull requests yourselves.

###### License:
Licensed under [MIT license](https://github.com/lmagitem/seeded-dice-roller/blob/master/LICENSE.md).
