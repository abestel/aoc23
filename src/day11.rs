use nom::{
    branch::alt,
    character::complete::{
        char,
        line_ending,
    },
    combinator::{
        all_consuming,
        map,
        opt,
        value,
    },
    multi::many1,
    sequence::terminated,
    Finish,
    IResult,
};
use std::{
    fmt::{
        Display,
        Formatter,
    },
    ops::RangeInclusive,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Item {
    Galaxy,
    Void,
}

impl Item {
    fn parse(input: &str) -> IResult<&str, Item> {
        alt((value(Item::Galaxy, char('#')), value(Item::Void, char('.'))))(input)
    }
}

#[derive(Debug)]
struct Space {
    galaxies: Vec<(usize, usize)>,
}

impl Space {
    fn new(items: Vec<Vec<Item>>) -> Self {
        Space {
            galaxies: items
                .iter()
                .enumerate()
                .flat_map(|(y, line)| {
                    line.iter().enumerate().filter_map(move |(x, item)| {
                        match item {
                            Item::Galaxy => Some((x, y)),
                            Item::Void => None,
                        }
                    })
                })
                .collect(),
        }
    }

    fn x_range(&self) -> RangeInclusive<usize> {
        0..=(self
            .galaxies
            .iter()
            .map(|(x, _)| *x)
            .max()
            .unwrap_or_default())
    }

    fn y_range(&self) -> RangeInclusive<usize> {
        0..=(self
            .galaxies
            .iter()
            .map(|(_, y)| *y)
            .max()
            .unwrap_or_default())
    }

    fn expand(
        &self,
        factor: usize,
    ) -> Self {
        let expansions = |indexes: Vec<usize>| {
            let mut expansions = Vec::new();
            let mut aggregated_expansion = 0;
            let mut previous_index = 0_usize;
            for index in indexes {
                expansions.push((previous_index..index, aggregated_expansion));
                previous_index = index;
                aggregated_expansion += factor - 1;
            }

            expansions.push((previous_index..usize::MAX, aggregated_expansion));

            expansions
        };

        let empty_cols: Vec<_> = self
            .x_range()
            .filter(|x| !self.galaxies.iter().any(|(gx, _)| gx == x))
            .collect();
        let x_expansions = expansions(empty_cols);

        let empty_rows: Vec<_> = self
            .y_range()
            .filter(|y| !self.galaxies.iter().any(|(_, gy)| gy == y))
            .collect();
        let y_expansions = expansions(empty_rows);

        let expanded = self
            .galaxies
            .iter()
            .map(|(x, y)| {
                let new_x = x_expansions
                    .iter()
                    .find_map(|(range, expansion)| {
                        if range.contains(x) {
                            Some(x + expansion)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(*x);

                let new_y = y_expansions
                    .iter()
                    .find_map(|(range, expansion)| {
                        if range.contains(y) {
                            Some(y + expansion)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(*y);

                (new_x, new_y)
            })
            .collect();

        Space { galaxies: expanded }
    }
}

impl Display for Space {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        for y in self.y_range() {
            for x in self.x_range() {
                if self.galaxies.contains(&(x, y)) {
                    f.write_str("#")?;
                } else {
                    f.write_str(".")?;
                }
            }

            f.write_str("\n")?;
        }
        Ok(())
    }
}

fn parse(input: &str) -> IResult<&str, Space> {
    all_consuming(map(
        many1(terminated(many1(Item::parse), opt(line_ending))),
        Space::new,
    ))(input)
}

fn do_stuff(
    name: &str,
    data: &str,
    factor: usize,
) {
    let (_, space) = parse(data).finish().unwrap();
    let expanded = space.expand(factor);

    let mut sum = 0_i64;
    for (i, (x1, y1)) in expanded.galaxies.iter().enumerate() {
        for (x2, y2) in &expanded.galaxies[(i + 1)..] {
            sum += (*x1 as i64 - *x2 as i64).abs() + (*y1 as i64 - *y2 as i64).abs();
        }
    }

    println!("[{}] Sum of shortest paths: {}", name, sum);
}

pub fn run() {
    do_stuff("First example", include_str!("data/day11/ex1"), 2); // 374
    do_stuff("First", include_str!("data/day11/input"), 2); // 10 173 804
    do_stuff("Second example", include_str!("data/day11/ex1"), 10); // 1030
    do_stuff("Second example 2", include_str!("data/day11/ex1"), 100); // 8410
    do_stuff("Second", include_str!("data/day11/input"), 1000000); // 634 324 905 172
}
