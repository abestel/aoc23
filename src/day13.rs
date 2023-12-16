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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Item {
    Ash,
    Rock,
}

impl Item {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((value(Item::Ash, char('.')), value(Item::Rock, char('#'))))(input)
    }
}

#[derive(Debug)]
enum ReflectionAxis {
    Horizontal(usize),
    Vertical(usize),
}

#[derive(Debug)]
struct Map {
    items: Vec<Vec<Item>>,
}

impl Map {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            many1(terminated(many1(Item::parse), line_ending)),
            |items| Map { items },
        )(input)
    }

    fn find_reflection_axis(
        items: &[Vec<Item>],
        cond: fn(usize) -> bool,
    ) -> Option<usize> {
        (1..(items[0].len())).find(|index| {
            let index = *index;
            let differences: usize = items
                .iter()
                .map(|line| {
                    let size = index.min(line.len() - index);

                    let diff = &line[(index - size)..index]
                        .iter()
                        .zip((line[index..(index + size)]).iter().rev())
                        .filter(|(l, r)| **l != **r)
                        .count();

                    *diff
                })
                .sum();

            cond(differences)
        })
    }

    fn reflection(
        &self,
        cond: fn(usize) -> bool,
    ) -> Option<ReflectionAxis> {
        Self::find_reflection_axis(&self.items, cond)
            .map(ReflectionAxis::Vertical)
            .or_else(|| {
                if let Some(first) = &self.items.first() {
                    let mut transposed = (0..first.len()).map(|_| vec![]).collect::<Vec<_>>();

                    for line in &self.items {
                        for (item, transposed_row) in line.iter().zip(&mut transposed) {
                            transposed_row.push(*item);
                        }
                    }

                    Self::find_reflection_axis(&transposed, cond).map(ReflectionAxis::Horizontal)
                } else {
                    None
                }
            })
    }
}

fn parse(input: &str) -> IResult<&str, Vec<Map>> {
    all_consuming(many1(terminated(Map::parse, opt(line_ending))))(input)
}

fn find_reflections(
    name: &str,
    data: &str,
    cond: fn(usize) -> bool,
) {
    let (_, maps) = parse(data).finish().unwrap();

    let total: usize = maps
        .iter()
        .filter_map(|map| map.reflection(cond))
        .map(|reflection| {
            match reflection {
                ReflectionAxis::Horizontal(axis) => axis * 100,
                ReflectionAxis::Vertical(axis) => axis,
            }
        })
        .sum();

    println!("[{}] Total: {:?}", name, total);
}

fn first(
    name: &str,
    data: &str,
) {
    find_reflections(name, data, |differences| differences == 0);
}

fn second(
    name: &str,
    data: &str,
) {
    find_reflections(name, data, |differences| differences == 1);
}

pub fn run() {
    first("First example", include_str!("data/day13/ex1")); // 405
    first("First", include_str!("data/day13/input")); // 27 505
    second("Second example", include_str!("data/day13/ex1")); // 405
    second("Second", include_str!("data/day13/input")); // 22 906
}
