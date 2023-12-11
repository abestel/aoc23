use nom::{
    character,
    character::complete::{
        line_ending,
        space1,
    },
    combinator::{
        all_consuming,
        opt,
    },
    multi::{
        many1,
        separated_list1,
    },
    sequence::terminated,
    Finish,
    IResult,
};

fn parse(input: &str) -> IResult<&str, Vec<Vec<i64>>> {
    all_consuming(many1(terminated(
        separated_list1(space1, character::complete::i64),
        opt(line_ending),
    )))(input)
}

fn compute_differences(sequence: &[i64]) -> Vec<Vec<i64>> {
    let mut differences: Vec<Vec<i64>> = Vec::new();
    differences.push(sequence.to_owned());

    loop {
        let last = differences.last().unwrap();
        if last.iter().all(|v| *v == 0) {
            break;
        }

        differences.push(
            last.windows(2)
                .map(|window| window[1] - window[0])
                .collect(),
        );
    }

    differences
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, sequences) = parse(data).finish().unwrap();

    let sum: i64 = sequences
        .iter()
        .map(|seq| compute_differences(seq.as_slice()))
        .map(|differences| {
            differences
                .iter()
                .rev()
                .fold(0_i64, |acc, differences| acc + *differences.last().unwrap())
        })
        .sum();

    println!("[{}] Sum: {}", name, sum);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, sequences) = parse(data).finish().unwrap();

    let sum: i64 = sequences
        .iter()
        .map(|seq| compute_differences(seq.as_slice()))
        .map(|differences| {
            differences.iter().rev().fold(0_i64, |acc, differences| {
                *differences.first().unwrap() - acc
            })
        })
        .sum();

    println!("[{}] Sum: {}", name, sum);
}

pub fn run() {
    first("First example", include_str!("data/day9/ex1")); // 114
    first("First", include_str!("data/day9/input")); // 1 647 269 739
    second("Second example", include_str!("data/day9/ex1")); // 2
    second("Second", include_str!("data/day9/input")); // 864
}
