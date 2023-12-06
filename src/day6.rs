use nom::{
    bytes::complete::tag,
    character,
    character::complete::{
        line_ending,
        space1,
    },
    combinator::{
        all_consuming,
        map,
    },
    multi::separated_list1,
    sequence::{
        delimited,
        tuple,
    },
    Finish,
    IResult,
};

#[derive(Debug)]
struct Race {
    time: u64,
    distance: u64,
}

impl Race {
    fn records(&self) -> Vec<u64> {
        (1..self.time)
            .map(|time| (self.time - time) * time)
            .filter(|distance| *distance > self.distance)
            .collect()
    }
}

fn parse(input: &str) -> IResult<&str, Vec<Race>> {
    all_consuming(map(
        tuple((
            delimited(
                tuple((tag("Time:"), space1)),
                separated_list1(space1, character::complete::u64),
                line_ending,
            ),
            delimited(
                tuple((tag("Distance:"), space1)),
                separated_list1(space1, character::complete::u64),
                line_ending,
            ),
        )),
        |(times, distances)| {
            times
                .into_iter()
                .zip(distances)
                .map(|(time, distance)| Race { time, distance })
                .collect()
        },
    ))(input)
}

fn parse2(input: &str) -> IResult<&str, Race> {
    all_consuming(map(
        tuple((
            delimited(
                tuple((tag("Time:"), space1)),
                separated_list1(space1, character::complete::digit1),
                line_ending,
            ),
            delimited(
                tuple((tag("Distance:"), space1)),
                separated_list1(space1, character::complete::digit1),
                line_ending,
            ),
        )),
        |(times, distances)| {
            let time: u64 = times.join("").as_str().parse().unwrap();
            let distance: u64 = distances.join("").as_str().parse().unwrap();
            Race { time, distance }
        },
    ))(input)
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, races) = parse(data).finish().unwrap();

    let records: u64 = races
        .iter()
        .map(|race| race.records().len() as u64)
        .product();

    println!("[{}] {:?}", name, records);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, race) = parse2(data).finish().unwrap();
    let records = race.records().len();
    println!("[{}] {:?}", name, records);
}

pub fn run() {
    first("First example", include_str!("data/day6/ex1")); // 288
    first("First", include_str!("data/day6/input")); // 1159152
    second("Second example", include_str!("data/day6/ex1")); // 71503
    second("Second", include_str!("data/day6/input")); // 41513103
}
