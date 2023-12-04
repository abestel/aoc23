use nom::{
    branch::alt,
    character,
    character::complete::{
        line_ending,
        satisfy,
    },
    combinator::{
        all_consuming,
        consumed,
        map,
        opt,
    },
    multi::{
        many1,
        many1_count,
    },
    sequence::terminated,
    Finish,
    IResult,
};
use std::collections::{
    HashMap,
    HashSet,
};

#[derive(Debug)]
enum Value {
    Number(u32),
    Symbol(char),
    Dots(usize),
}

impl Value {
    fn parse(input: &str) -> IResult<&str, Value> {
        alt((
            map(many1_count(character::complete::char('.')), Value::Dots),
            map(character::complete::u32, Value::Number),
            map(satisfy(|c| c != '\n' && c != '\r'), Value::Symbol),
        ))(input)
    }
}

#[derive(Debug)]
struct Cell {
    x: i64,
    y: i64,
    size: usize,
    value: Value,
}

impl Cell {
    fn adjacent(&self) -> Vec<(i64, i64)> {
        ((self.x - 1)..=(self.x + (self.size as i64)))
            .flat_map(|x| ((self.y - 1)..=(self.y + 1)).map(move |y| (x, y)))
            .collect()
    }
}

fn parse(input: &str) -> IResult<&str, Vec<Cell>> {
    all_consuming(map(
        many1(terminated(
            map(many1(consumed(Value::parse)), |parsed| parsed),
            opt(line_ending),
        )),
        |raw_cells| {
            raw_cells
                .into_iter()
                .enumerate()
                .flat_map(|(line_index, raw_cells)| {
                    let mut cells = Vec::new();
                    let mut x: i64 = 0;
                    let y = line_index as i64;

                    for (input, value) in raw_cells {
                        let size = input.len();
                        cells.push(Cell { x, y, size, value });
                        x += size as i64;
                    }

                    cells
                })
                .collect()
        },
    ))(input)
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, cells) = parse(data).finish().unwrap();
    let symbols = cells
        .iter()
        .filter_map(|cell| {
            match cell.value {
                Value::Symbol(_) => Some((cell.x, cell.y)),
                _ => None,
            }
        })
        .collect::<HashSet<_>>();

    let sum: u64 = cells
        .iter()
        .filter_map(|cell| {
            match cell.value {
                Value::Number(num) => {
                    let has_adjacent_symbol = cell.adjacent().iter().any(|c| symbols.contains(c));

                    if has_adjacent_symbol {
                        Some(num as u64)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .sum();

    println!("[{}] Sum of part numbers '{}'", name, sum)
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, cells) = parse(data).finish().unwrap();
    let mut gears = cells
        .iter()
        .filter_map(|cell| {
            match cell.value {
                Value::Symbol(symbol) => {
                    if symbol == '*' {
                        Some(((cell.x, cell.y), Vec::new()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect::<HashMap<_, _>>();

    cells.iter().for_each(|cell| {
        if let Value::Number(number) = cell.value {
            cell.adjacent().into_iter().for_each(|c| {
                if let Some(gear) = gears.get_mut(&c) {
                    gear.push(number);
                }
            })
        }
    });

    let sum: u64 = gears
        .values()
        .filter_map(|numbers| {
            if numbers.len() == 2 {
                Some(numbers.iter().map(|x| *x as u64).product::<u64>())
            } else {
                None
            }
        })
        .sum();

    println!("[{}] Sum of part numbers '{}'", name, sum)
}

pub fn run() {
    first("First Example", include_str!("data/day3/ex1")); // 4361
    first("First", include_str!("data/day3/input")); // 4361
    second("Second Example", include_str!("data/day3/ex1")); // 467835
    second("Second", include_str!("data/day3/input")); // 67779080
}
