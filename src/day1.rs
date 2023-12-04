use nom::{
    branch::alt,
    bytes::complete::tag,
    character,
    character::complete::{
        char,
        line_ending,
    },
    combinator::{
        all_consuming,
        map,
        opt,
        peek,
        value,
    },
    multi::many1,
    sequence::{
        terminated,
        tuple,
    },
    Finish,
    IResult,
};

#[derive(Clone, Debug)]
enum Value {
    Char(char),
    Number(u8),
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl Value {
    fn number(&self) -> Option<u8> {
        match self {
            Value::Number(num) => Some(*num),
            _ => None,
        }
    }

    fn number_2(&self) -> Option<u8> {
        match self {
            Value::Number(num) => Some(*num),
            Value::One => Some(1),
            Value::Two => Some(2),
            Value::Three => Some(3),
            Value::Four => Some(4),
            Value::Five => Some(5),
            Value::Six => Some(6),
            Value::Seven => Some(7),
            Value::Eight => Some(8),
            Value::Nine => Some(9),
            Value::Char(_) => None,
        }
    }
}

fn parse(input: &str) -> IResult<&str, Vec<Vec<Value>>> {
    all_consuming(many1(terminated(
        many1(alt((
            // Match "stringified" numbers without consuming more than the first character
            // This allows to read 'twone' as [Two, One] instead of [Two, Char(n), Char(e)]
            value(Value::One, tuple((char('o'), peek(tag("ne"))))),
            value(Value::Two, tuple((char('t'), peek(tag("wo"))))),
            value(Value::Three, tuple((char('t'), peek(tag("hree"))))),
            value(Value::Four, tuple((char('f'), peek(tag("our"))))),
            value(Value::Five, tuple((char('f'), peek(tag("ive"))))),
            value(Value::Six, tuple((char('s'), peek(tag("ix"))))),
            value(Value::Seven, tuple((char('s'), peek(tag("even"))))),
            value(Value::Eight, tuple((char('e'), peek(tag("ight"))))),
            value(Value::Nine, tuple((char('n'), peek(tag("ine"))))),
            // Read a number
            map(character::complete::satisfy(|c| c.is_numeric()), |c| {
                Value::Number(c as u8 - b'0')
            }),
            // Read any other character
            map(
                character::complete::satisfy(|c| c.is_alphabetic()),
                Value::Char,
            ),
        ))),
        opt(line_ending),
    )))(input)
}

fn parse_and_sum(
    name: &str,
    data: &str,
    extract_number: fn(&Value) -> Option<u8>,
) {
    // Parse the input date
    let (_, result) = parse(data).finish().unwrap();

    let sum: u64 = result
        .iter()
        .map(|line| {
            // Extract the numbers from the line
            let numbers: Vec<_> = line.iter().filter_map(extract_number).collect();

            // Get the first and last number of the line
            let first = numbers.first().map(|d| *d as u64).unwrap_or(0);
            let last = numbers.last().map(|d| *d as u64).unwrap_or(0);

            // Return the number [first number][last number]
            first * 10 + last
        })
        // Sum all numbers
        .sum();

    println!("[{}] Sum is '{}'", name, sum)
}

fn first(
    name: &str,
    data: &str,
) {
    // Do not care about stringified numbers
    parse_and_sum(name, data, Value::number)
}

fn second(
    name: &str,
    data: &str,
) {
    // Handle stringified numbers
    parse_and_sum(name, data, Value::number_2)
}

pub fn run() {
    first("First example", include_str!("data/day1/ex1")); // 142
    first("First", include_str!("data/day1/input")); // 54573
    second("Second example", include_str!("data/day1/ex2")); // 302
    second("Second", include_str!("data/day1/input")); // 54591
}
