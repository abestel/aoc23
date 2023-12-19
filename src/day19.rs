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
        value,
    },
    multi::{
        many1,
        separated_list1,
    },
    sequence::{
        delimited,
        separated_pair,
        terminated,
        tuple,
    },
    Finish,
    IResult,
};
use std::{
    collections::HashMap,
    ops::Range,
};

#[derive(Clone, Copy, Debug)]
enum Result {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug)]
enum Action<'a> {
    Result(Result),
    MoveTo(&'a str),
}

impl<'a> Action<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(Action::Result(Result::Accepted), char('A')),
            value(Action::Result(Result::Rejected), char('R')),
            map(is_not(",}"), Action::MoveTo),
        ))(input)
    }
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    LessThan,
    MoreThan,
}

impl Operation {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            value(Operation::MoreThan, char('>')),
            value(Operation::LessThan, char('<')),
        ))(input)
    }
}

#[derive(Clone, Copy, Debug)]
enum Condition<'a> {
    All {
        action: Action<'a>,
    },
    Operation {
        field: &'a str,
        operation: Operation,
        threshold: u32,
        action: Action<'a>,
    },
}

impl<'a> Condition<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                tuple((
                    alpha1,
                    Operation::parse,
                    character::complete::u32,
                    char(':'),
                    Action::parse,
                )),
                |(field, operation, threshold, _, action)| {
                    Condition::Operation {
                        field,
                        operation,
                        threshold,
                        action,
                    }
                },
            ),
            map(Action::parse, |action| Condition::All { action }),
        ))(input)
    }

    fn process(
        &self,
        data: &Data<'a>,
    ) -> Option<Action> {
        match self {
            Condition::All { action } => Some(*action),
            Condition::Operation {
                field,
                operation,
                threshold,
                action,
            } => {
                if data.values.get(field).is_some_and(|value| {
                    match operation {
                        Operation::LessThan => value < threshold,
                        Operation::MoreThan => value > threshold,
                    }
                }) {
                    Some(*action)
                } else {
                    None
                }
            }
        }
    }

    fn process_range(
        &self,
        data: &DataRange<'a>,
    ) -> ConditionRangeResult {
        match self {
            Condition::All { action } => {
                ConditionRangeResult {
                    action: *action,
                    matched: (*data).clone(),
                    unmatched: DataRange::empty(),
                }
            }
            Condition::Operation {
                field,
                operation,
                threshold,
                action,
            } => {
                if let Some(range) = data.values.get(field) {
                    match operation {
                        Operation::LessThan => {
                            if range.start < *threshold {
                                let mut matched = (*data).clone();
                                matched.values.insert(field, range.start..*threshold);

                                let mut unmatched = (*data).clone();
                                unmatched.values.insert(field, *threshold..range.end);

                                ConditionRangeResult {
                                    action: *action,
                                    matched,
                                    unmatched,
                                }
                            } else {
                                ConditionRangeResult {
                                    action: Action::Result(Result::Rejected), // Don't care
                                    matched: DataRange::empty(),
                                    unmatched: (*data).clone(),
                                }
                            }
                        }
                        Operation::MoreThan => {
                            if range.end > *threshold {
                                let mut matched = (*data).clone();
                                matched.values.insert(field, (*threshold + 1)..range.end);

                                let mut unmatched = (*data).clone();
                                unmatched
                                    .values
                                    .insert(field, range.start..(*threshold + 1));

                                ConditionRangeResult {
                                    action: *action,
                                    matched,
                                    unmatched,
                                }
                            } else {
                                ConditionRangeResult {
                                    action: Action::Result(Result::Rejected), // Don't care
                                    matched: DataRange::empty(),
                                    unmatched: (*data).clone(),
                                }
                            }
                        }
                    }
                } else {
                    ConditionRangeResult {
                        action: Action::Result(Result::Rejected), // Don't care
                        matched: DataRange::empty(),
                        unmatched: DataRange::empty(),
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct ConditionRangeResult<'a> {
    action: Action<'a>,
    matched: DataRange<'a>,
    unmatched: DataRange<'a>,
}

#[derive(Debug)]
struct Conditions<'a> {
    conditions: HashMap<&'a str, Vec<Condition<'a>>>,
}

impl<'a> Conditions<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            many1(terminated(
                tuple((
                    alpha1,
                    delimited(
                        char('{'),
                        separated_list1(char(','), Condition::parse),
                        char('}'),
                    ),
                )),
                line_ending,
            )),
            |conditions| {
                Conditions {
                    conditions: conditions.into_iter().collect(),
                }
            },
        )(input)
    }

    fn process(
        &self,
        data: &Data<'a>,
    ) -> Result {
        let mut conditions = &self.conditions["in"];
        loop {
            match conditions
                .iter()
                .find_map(|condition| condition.process(data))
            {
                None => break,
                Some(action) => {
                    match action {
                        Action::Result(result) => return result,
                        Action::MoveTo(next_label) => {
                            conditions = &self.conditions[next_label];
                        }
                    }
                }
            }
        }

        Result::Rejected
    }

    fn process_range(
        &'a self,
        data: DataRange<'a>,
    ) -> Vec<DataRange<'a>> {
        let mut ranges = vec![("in", data)];
        let mut results = Vec::new();

        while !ranges.is_empty() {
            let mut new_ranges = Vec::new();
            for (label, data) in ranges {
                let conditions = &self.conditions[label];
                let mut data = data;
                for condition in conditions {
                    let ConditionRangeResult {
                        action,
                        matched,
                        unmatched,
                    } = condition.process_range(&data);
                    match action {
                        Action::Result(result) => {
                            match result {
                                Result::Accepted => results.push(matched),
                                Result::Rejected => {}
                            }
                        }
                        Action::MoveTo(label) => new_ranges.push((label, matched)),
                    }

                    if unmatched.is_empty() {
                        break;
                    }

                    data = unmatched;
                }
            }
            ranges = new_ranges;
        }

        results
    }
}

#[derive(Debug)]
struct Data<'a> {
    values: HashMap<&'a str, u32>,
}

impl<'a> Data<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            delimited(
                char('{'),
                separated_list1(
                    char(','),
                    separated_pair(alpha1, char('='), character::complete::u32),
                ),
                char('}'),
            ),
            |data| {
                Data {
                    values: data.into_iter().collect(),
                }
            },
        )(input)
    }
}

#[derive(Clone, Debug)]
struct DataRange<'a> {
    values: HashMap<&'a str, Range<u32>>,
}

impl<'a> DataRange<'a> {
    fn empty() -> Self {
        DataRange {
            values: HashMap::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.values.is_empty() || self.values.values().any(|range| range.is_empty())
    }
}

fn parse(input: &str) -> IResult<&str, (Conditions, Vec<Data>)> {
    all_consuming(separated_pair(
        Conditions::parse,
        line_ending,
        many1(terminated(Data::parse, line_ending)),
    ))(input)
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, (conditions, data)) = parse(data).finish().unwrap();

    let sum: u32 = data
        .iter()
        .filter(|data| {
            match conditions.process(data) {
                Result::Accepted => true,
                Result::Rejected => false,
            }
        })
        .map(|data| data.values.values().sum::<u32>())
        .sum();

    println!("[{}] Sum of accepted parts {}", name, sum);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, (conditions, _)) = parse(data).finish().unwrap();

    let mut values = HashMap::new();
    values.insert("x", 1..4001);
    values.insert("m", 1..4001);
    values.insert("a", 1..4001);
    values.insert("s", 1..4001);

    let result_ranges = conditions.process_range(DataRange { values });
    let combinations: u64 = result_ranges
        .iter()
        .map(|r| {
            r.values
                .values()
                .map(|range| range.len() as u64)
                .product::<u64>()
        })
        .sum();

    println!("[{}] Total combinations working: {}", name, combinations);
}

pub fn run() {
    first("First example", include_str!("data/day19/ex1")); // 19 114
    first("First", include_str!("data/day19/input")); // 323 625
    second("Second example", include_str!("data/day19/ex1")); // 167 409 079 868 000
    second("Second", include_str!("data/day19/input")); // 127 447 746 739 409
}
