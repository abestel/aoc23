use nom::{
    branch::alt,
    bytes::complete::{
        is_not,
        tag,
        take,
    },
    character,
    character::complete::{
        char,
        line_ending,
        space1,
    },
    combinator::{
        all_consuming,
        map,
        map_res,
        value,
    },
    multi::many1,
    sequence::{
        delimited,
        terminated,
        tuple,
    },
    Finish,
    IResult,
};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn parse_1(input: &str) -> IResult<&str, Self> {
        alt((
            value(Direction::Up, char('U')),
            value(Direction::Down, char('D')),
            value(Direction::Left, char('L')),
            value(Direction::Right, char('R')),
        ))(input)
    }

    fn parse_2(input: &str) -> IResult<&str, Self> {
        alt((
            value(Direction::Up, char('3')),
            value(Direction::Down, char('1')),
            value(Direction::Left, char('2')),
            value(Direction::Right, char('0')),
        ))(input)
    }
}

#[derive(Debug)]
struct Drill {
    direction: Direction,
    length: i64,
}

impl Drill {
    fn parse_1(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                Direction::parse_1,
                space1,
                character::complete::i64,
                is_not("\n"),
            )),
            |(direction, _, length, _)| Self { direction, length },
        )(input)
    }

    fn parse_2(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                is_not("("),
                delimited(
                    tag("(#"),
                    tuple((
                        map_res(take(5u8), |x| i64::from_str_radix(x, 16)),
                        Direction::parse_2,
                    )),
                    tag(")"),
                ),
            )),
            |(_, (length, direction))| Self { direction, length },
        )(input)
    }
}

fn parse_1(input: &str) -> IResult<&str, Vec<Drill>> {
    all_consuming(many1(terminated(Drill::parse_1, line_ending)))(input)
}

fn parse_2(input: &str) -> IResult<&str, Vec<Drill>> {
    all_consuming(many1(terminated(Drill::parse_2, line_ending)))(input)
}

fn shoelace(points: &Vec<(i64, i64)>) -> i64 {
    points
        .as_slice()
        .windows(2)
        .map(|window| {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];
            x1 * y2 - y1 * x2
        })
        .sum::<i64>()
        .abs()
        / 2
}

fn perimeter(points: &Vec<(i64, i64)>) -> i64 {
    points
        .as_slice()
        .windows(2)
        .map(|window| {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];
            (x1 - x2).abs() + (y1 - y2).abs()
        })
        .sum::<i64>()
}

fn process(drills: Vec<Drill>) -> i64 {
    let mut current = (0i64, 0i64);
    let mut points = vec![current];
    for Drill {
        direction, length, ..
    } in drills
    {
        let (x, y) = current;
        current = match direction {
            Direction::Up => (x, y - length),
            Direction::Down => (x, y + length),
            Direction::Left => (x - length, y),
            Direction::Right => (x + length, y),
        };

        points.push(current);
    }

    let shoelace_area = shoelace(&points);
    let perimeter_area = perimeter(&points);
    shoelace_area + perimeter_area / 2 + 1
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, drills) = parse_1(data).finish().unwrap();
    let area = process(drills);
    println!("[{}] Area is {:#?}", name, area);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, drills) = parse_2(data).finish().unwrap();
    let area = process(drills);
    println!("[{}] Area is {:#?}", name, area);
}

pub fn run() {
    first("First example", include_str!("data/day18/ex1")); // 62
    first("First", include_str!("data/day18/input")); // 50603
    second("Second example", include_str!("data/day18/ex1")); // 952 408 144 115
    second("Second", include_str!("data/day18/input")); // 96 556 251 590 677
}
