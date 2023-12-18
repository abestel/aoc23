use nom::{
    branch::alt,
    bytes::complete::is_not,
    character,
    character::complete::{
        alpha1,
        char,
        line_ending,
    },
    combinator::{
        all_consuming,
        map,
        opt,
    },
    multi::separated_list1,
    sequence::{
        separated_pair,
        terminated,
    },
    Finish,
    IResult,
};
use std::collections::HashMap;

fn hash(string: &str) -> u32 {
    string
        .chars()
        .fold(0, |acc, char| ((acc + char as u32) * 17) % 256)
}

#[derive(Debug)]
enum Operation<'a> {
    Assign { label: &'a str, focal_length: u32 },
    Remove { label: &'a str },
}

impl <'a> Operation<'a> {
    fn parse(input: &'a str) -> IResult<& str, Self> {
        alt((
            map(terminated(alpha1, char('-')), |label| {
                Operation::Remove { label }
            }),
            map(
                separated_pair(alpha1, char('='), character::complete::u32),
                |(label, focal_length)| {
                    Operation::Assign {
                        label,
                        focal_length,
                    }
                },
            ),
        ))(input)
    }
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, sequence) = all_consuming(terminated(
        separated_list1(char::<&str, nom::error::Error<&str>>(','), is_not(",\n")),
        opt(line_ending),
    ))(data)
    .finish()
    .unwrap();
    let tot: u32 = sequence.iter().map(|part| hash(part)).sum();
    println!("[{}] {:?}", name, tot);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, sequence) = all_consuming(terminated(
        separated_list1(char::<&str, nom::error::Error<&str>>(','), Operation::parse),
        opt(line_ending),
    ))(data)
    .finish()
    .unwrap();

    let result = sequence.iter().fold(HashMap::new(), |mut map, operation| {
        match operation {
            Operation::Assign {
                label: op_label,
                focal_length,
            } => {
                let slots = map
                    .entry(hash(op_label))
                    .or_insert_with(Vec::<(&str, u32)>::new);

                let slot = slots.iter_mut().find(|(label, _)| op_label.eq(label));
                match slot {
                    Some(entry) => {
                        *entry = (op_label, *focal_length);
                    }

                    None => slots.push((op_label, *focal_length)),
                };
            }
            Operation::Remove { label: op_label } => {
                map.entry(hash(op_label)).and_modify(|slots| {
                    let index = slots.iter().enumerate().find_map(|(index, (label, _))| {
                        if op_label.eq(label) {
                            Some(index)
                        } else {
                            None
                        }
                    });

                    if let Some(index) = index {
                        slots.remove(index);
                    }
                });
            }
        };

        map
    });

    let result: u32 = result
        .iter()
        .flat_map(|(hash, slots)| {
            slots.iter().enumerate().map(|(index, (_, focal_length))| {
                let box_coeff = *hash + 1;
                let slot_coeff = index as u32 + 1;
                box_coeff * slot_coeff * focal_length
            })
        })
        .sum();

    println!("[{}] {:?}", name, result);
}

pub fn run() {
    first("First example", include_str!("data/day15/ex1")); // 1 320
    first("First", include_str!("data/day15/input")); // 515 974
    second("Second example", include_str!("data/day15/ex1")); // 145
    second("Second", include_str!("data/day15/input")); // 265 894
}
