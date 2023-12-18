use nom::{
    branch::alt,
    character::complete::{
        char,
        line_ending,
    },
    combinator::{
        all_consuming,
        value,
    },
    multi::many1,
    sequence::terminated,
    Finish,
    IResult,
};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    iter::once,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Item {
    // -
    HorizontalSplitter,
    // |
    VerticalSplitter,
    // \
    LeftToRightMirror,
    // /
    RightToLeftMirror,
    // .
    Empty,
}

impl Item {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            value(Item::HorizontalSplitter, char('-')),
            value(Item::VerticalSplitter, char('|')),
            value(Item::LeftToRightMirror, char('\\')),
            value(Item::RightToLeftMirror, char('/')),
            value(Item::Empty, char('.')),
        ))(input)
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn next(
        &self,
        coords: (i32, i32),
    ) -> (i32, i32) {
        let (x, y) = coords;
        match self {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        }
    }
}

fn parse(input: &str) -> IResult<&str, Vec<Vec<Item>>> {
    all_consuming(many1(terminated(many1(Item::parse), line_ending)))(input)
}

fn energize(
    items: &Vec<Vec<Item>>,
    first_direction: Direction,
    first_coords: (i32, i32),
) -> HashSet<(i32, i32)> {
    fn run_loop(
        items: &Vec<Vec<Item>>,
        active: Vec<Vec<(Direction, (i32, i32))>>,
        mut done: HashSet<(Direction, (i32, i32))>,
    ) -> HashSet<(Direction, (i32, i32))> {
        if active.is_empty() {
            done
        } else {
            let mut next_active = Vec::new();
            for path in active {
                if let Some((direction, (x, y))) = path.last() {
                    let item = items[*y as usize][*x as usize];
                    let next_directions = match item {
                        Item::Empty => vec![*direction],

                        Item::VerticalSplitter => {
                            match direction {
                                Direction::Up | Direction::Down => vec![*direction],
                                Direction::Left | Direction::Right => {
                                    vec![Direction::Up, Direction::Down]
                                }
                            }
                        }

                        Item::HorizontalSplitter => {
                            match direction {
                                Direction::Left | Direction::Right => vec![*direction],
                                Direction::Up | Direction::Down => {
                                    vec![Direction::Left, Direction::Right]
                                }
                            }
                        }

                        Item::RightToLeftMirror => {
                            vec![match direction {
                                Direction::Up => Direction::Right,
                                Direction::Down => Direction::Left,
                                Direction::Left => Direction::Down,
                                Direction::Right => Direction::Up,
                            }]
                        }

                        Item::LeftToRightMirror => {
                            vec![match direction {
                                Direction::Up => Direction::Left,
                                Direction::Down => Direction::Right,
                                Direction::Left => Direction::Up,
                                Direction::Right => Direction::Down,
                            }]
                        }
                    };

                    for direction in next_directions {
                        let (x, y) = direction.next((*x, *y));
                        let already_visited = done.contains(&(direction, (x, y)));
                        let next_in_grid = 0 <= x
                            && x < items.first().map(|line| line.len()).unwrap_or_default() as i32
                            && 0 <= y
                            && y < items.len() as i32;

                        if !already_visited && next_in_grid {
                            next_active.push(
                                path.iter()
                                    .copied()
                                    .chain(once((direction, (x, y))))
                                    .collect(),
                            );
                        } else {
                            path.iter().for_each(|x| {
                                done.insert(*x);
                            });
                        }
                    }
                }
            }

            run_loop(items, next_active, done)
        }
    }

    let energized = run_loop(
        items,
        vec![vec![(first_direction, first_coords)]],
        HashSet::new(),
    );

    energized
        .iter()
        .map(|(_, coords)| coords)
        .copied()
        .collect()
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, items) = parse(data).finish().unwrap();

    let energized = energize(&items, Direction::Right, (0, 0));
    println!("[{}] Energized tiles {:?}", name, energized.len());
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, items) = parse(data).finish().unwrap();

    // First column, x=0, moving y, going right
    let max_energized = (0..(items.len() - 1))
        .map(|y| (Direction::Right, (0, y)))
        .chain(
            // Last column, x=len-1, moving y, going left
            (0..(items.len() - 1)).map(|y| (Direction::Left, (items[0].len() - 1, y))),
        )
        .chain(
            // First line, moving x, y=0, going down
            (0..(items[0].len() - 1)).map(|x| (Direction::Down, (x, 0))),
        )
        .chain(
            // Last line, moving x, y=len-1, going up
            (0..(items[0].len() - 1)).map(|x| (Direction::Up, (x, items.len() - 1))),
        )
        .collect::<Vec<_>>()
        .par_iter()
        .map(|(direction, (x, y))| energize(&items, *direction, (*x as i32, *y as i32)).len())
        .max()
        .unwrap_or_default();

    println!("[{}] Max energized tiles {:?}", name, max_energized);
}

pub fn run() {
    first("First example", include_str!("data/day16/ex1")); // 46
    first("First", include_str!("data/day16/input")); // 7472
    second("Second example", include_str!("data/day16/ex1")); // 46
    second("Second", include_str!("data/day16/input")); // 46
}
