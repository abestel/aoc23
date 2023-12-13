use nom::{
    branch::alt,
    character,
    character::complete::{
        char,
        line_ending,
        space1,
    },
    combinator::{
        all_consuming,
        map,
        opt,
        value,
    },
    multi::{
        many1,
        separated_list1,
    },
    sequence::{
        separated_pair,
        terminated,
    },
    Finish,
    IResult,
};
use std::{
    collections::HashMap,
    iter::once,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum SpringState {
    Damaged,
    Operational,
    Unknown,
}

impl SpringState {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            value(Self::Damaged, char('#')),
            value(Self::Operational, char('.')),
            value(Self::Unknown, char('?')),
        ))(input)
    }
}

#[derive(Debug)]
struct Springs {
    states: Vec<SpringState>,
    damaged_groups: Vec<u16>,
}

impl Springs {
    fn parse(input: &str) -> IResult<&str, Vec<Self>> {
        all_consuming(many1(terminated(
            map(
                separated_pair(
                    many1(SpringState::parse),
                    space1,
                    separated_list1(char(','), character::complete::u16),
                ),
                |(states, damaged_groups)| {
                    Springs {
                        states,
                        damaged_groups,
                    }
                },
            ),
            opt(line_ending),
        )))(input)
    }

    fn find_arrangements(&self) -> usize {
        fn run_loop(
            states: &[SpringState],
            damaged_groups: &[u16],
            cache: &mut HashMap<(Vec<SpringState>, Vec<u16>), usize>,
        ) -> usize {
            let cache_key = (states.to_vec(), damaged_groups.to_vec());

            // If the cache already has the value pre-computed, just return it
            if let Some(count) = cache.get(&cache_key) {
                *count
            } else {
                // Otherwise check the input variables
                let result = match states.first() {
                    // If we still have springs to consider...
                    Some(state) => {
                        match state {
                            SpringState::Operational => {
                                run_loop(&states[1..], damaged_groups, cache)
                            }

                            SpringState::Unknown => {
                                // if the spring is unknown, it can either be operational...
                                run_loop(&states[1..], damaged_groups, cache) +
                                    // ... or damaged, in which case we just recurse swapping the first value by a damaged spring
                                    run_loop(&[&[SpringState::Damaged], &states[1..]].concat(), damaged_groups, cache)
                            }

                            SpringState::Damaged => {
                                match damaged_groups.first() {
                                    None => {
                                        // No more damaged springs, no solution
                                        0
                                    }

                                    Some(first_group_size) => {
                                        let first_group_size = *first_group_size as usize;

                                        if
                                        // Not enough springs left to fill the damaged group, no solution
                                        states.len() < first_group_size ||
                                            // There's at least one operational spring in the next 'first_group_size' springs, so the group is not possible
                                            states[..first_group_size]
                                                .iter()
                                                .any(|state| *state == SpringState::Operational) ||
                                            // The spring after the group size is damaged, which would created a group that is too big, so this is not possible
                                            states
                                                .get(first_group_size)
                                                .is_some_and(|state| *state == SpringState::Damaged)
                                        {
                                            0
                                        } else if states.len() == first_group_size {
                                            // If there's only one group left and the remaining states are all damaged or unknown, then we have a solution
                                            if damaged_groups.len() == 1 {
                                                1
                                            } else {
                                                0
                                            }
                                        } else {
                                            run_loop(
                                                &states[(first_group_size + 1)..],
                                                &damaged_groups[1..],
                                                cache,
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // ... else, if there's no more spring...
                    None => {
                        if damaged_groups.is_empty() {
                            // ... and no more groups, then we have a solution...
                            1
                        } else {
                            // ... otherwise this is not a solution since the arrangement does not have enough damaged springs
                            0
                        }
                    }
                };

                cache.insert(cache_key, result);

                result
            }
        }

        let mut cache = HashMap::new();
        run_loop(
            self.states.as_slice(),
            self.damaged_groups.as_slice(),
            &mut cache,
        )
    }
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, springs) = Springs::parse(data).finish().unwrap();

    let total: usize = springs
        .iter()
        .map(|springs| springs.find_arrangements())
        .sum();
    println!("[{}] Possible arrangements: {:#?}", name, total);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, springs) = Springs::parse(data).finish().unwrap();

    let total: usize = springs
        .into_iter()
        .map(
            |Springs {
                 states,
                 damaged_groups,
             }| {
                Springs {
                    states: states
                        .iter()
                        .chain(once(&SpringState::Unknown))
                        .cycle()
                        .take((states.len() + 1) * 5 - 1)
                        .cloned()
                        .collect(),
                    damaged_groups: damaged_groups
                        .iter()
                        .cycle()
                        .take(damaged_groups.len() * 5)
                        .copied()
                        .collect(),
                }
            },
        )
        .map(|springs| springs.find_arrangements())
        .sum();

    println!("[{}] Possible arrangements: {:#?}", name, total);
}

pub fn run() {
    first("First example", include_str!("data/day12/ex1")); // 21
    first("First", include_str!("data/day12/input")); // 7407
    second("Second example", include_str!("data/day12/ex1")); // 525 152
    second("Second", include_str!("data/day12/input")); // 30 568 243 604 962
}
