use nom::{
    branch::alt,
    character::complete::{
        char,
        line_ending,
    },
    combinator::{
        all_consuming,
        map,
        value,
    },
    multi::many1,
    sequence::terminated,
    Finish,
    IResult,
};
use std::{
    collections::{
        hash_map::DefaultHasher,
        HashMap,
    },
    fmt::{
        Display,
        Formatter,
    },
    hash::{
        Hash,
        Hasher,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Item {
    RoundedRock,
    CubeRock,
    Empty,
}

impl Item {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            value(Item::RoundedRock, char('O')),
            value(Item::CubeRock, char('#')),
            value(Item::Empty, char('.')),
        ))(input)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Map {
    items: Vec<Vec<Item>>,
}

impl Map {
    fn parse(input: &str) -> IResult<&str, Self> {
        all_consuming(map(
            many1(terminated(many1(Item::parse), line_ending)),
            |items| Map { items },
        ))(input)
    }

    fn tilt_north(&mut self) -> &mut Self {
        match self.items.first() {
            None => self,
            Some(first) => {
                let first_line_len = first.len();

                for x in 0..first_line_len {
                    let mut last_fixed_item_y: Option<usize> = None;
                    for y in 0..self.items.len() {
                        let item = self.items[y][x];
                        match item {
                            Item::RoundedRock => {
                                let new_y = last_fixed_item_y.map(|y| y + 1).unwrap_or_default();

                                self.items[y][x] = Item::Empty;
                                self.items[new_y][x] = Item::RoundedRock;

                                last_fixed_item_y = Some(new_y);
                            }
                            Item::CubeRock => {
                                last_fixed_item_y = Some(y);
                            }
                            Item::Empty => {}
                        }
                    }
                }

                self
            }
        }
    }

    fn tilt_west(&mut self) -> &mut Self {
        for y in 0..self.items.len() {
            let mut last_fixed_item_x: Option<usize> = None;
            for x in 0..self.items[y].len() {
                let item = self.items[y][x];
                match item {
                    Item::RoundedRock => {
                        let new_x = last_fixed_item_x.map(|x| x + 1).unwrap_or_default();

                        self.items[y][x] = Item::Empty;
                        self.items[y][new_x] = Item::RoundedRock;

                        last_fixed_item_x = Some(new_x);
                    }
                    Item::CubeRock => {
                        last_fixed_item_x = Some(x);
                    }
                    Item::Empty => {}
                }
            }
        }

        self
    }

    fn tilt_south(&mut self) -> &mut Self {
        match self.items.first() {
            None => self,
            Some(first) => {
                let first_line_len = first.len();

                for x in 0..first_line_len {
                    let mut last_fixed_item_y: Option<usize> = None;
                    for y in (0..self.items.len()).rev() {
                        let item = self.items[y][x];
                        match item {
                            Item::RoundedRock => {
                                let new_y = last_fixed_item_y
                                    .map(|y| y - 1)
                                    .unwrap_or(self.items.len() - 1);

                                self.items[y][x] = Item::Empty;
                                self.items[new_y][x] = Item::RoundedRock;

                                last_fixed_item_y = Some(new_y);
                            }
                            Item::CubeRock => {
                                last_fixed_item_y = Some(y);
                            }
                            Item::Empty => {}
                        }
                    }
                }

                self
            }
        }
    }

    fn tilt_east(&mut self) -> &mut Self {
        for y in 0..self.items.len() {
            let mut last_fixed_item_x: Option<usize> = None;
            for x in (0..self.items[y].len()).rev() {
                let item = self.items[y][x];
                match item {
                    Item::RoundedRock => {
                        let new_x = last_fixed_item_x
                            .map(|x| x - 1)
                            .unwrap_or(self.items[y].len() - 1);

                        self.items[y][x] = Item::Empty;
                        self.items[y][new_x] = Item::RoundedRock;

                        last_fixed_item_x = Some(new_x);
                    }
                    Item::CubeRock => {
                        last_fixed_item_x = Some(x);
                    }
                    Item::Empty => {}
                }
            }
        }

        self
    }

    fn load(&self) -> usize {
        let lines = self.items.len();
        self.items
            .iter()
            .enumerate()
            .flat_map(|(y, line)| {
                line.iter().filter_map(move |item| {
                    match item {
                        Item::RoundedRock => Some(lines - y),
                        Item::CubeRock => None,
                        Item::Empty => None,
                    }
                })
            })
            .sum()
    }
}

impl Display for Map {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        for line in &self.items {
            for item in line {
                match item {
                    Item::RoundedRock => f.write_str("O")?,
                    Item::CubeRock => f.write_str("#")?,
                    Item::Empty => f.write_str(".")?,
                }
            }

            f.write_str("\n")?;
        }

        Ok(())
    }
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, mut map) = Map::parse(data).finish().unwrap();
    let tilted = map.tilt_north();
    println!("[{}] Load: {}", name, tilted.load());
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, mut map) = Map::parse(data).finish().unwrap();

    let mut tilted = &mut map;
    let mut seen = HashMap::new();

    for i in 0..1000000000 {
        let mut hasher = DefaultHasher::new();
        tilted.hash(&mut hasher);
        let hash = hasher.finish();

        // Cycle detection
        if let Some(seen_at) = seen.insert(hash, i) {
            if (1000000000 - i) % (i - seen_at) == 0 {
                break;
            }
        }

        tilted = tilted.tilt_north().tilt_west().tilt_south().tilt_east();
    }

    println!("[{}] Load: {}", name, tilted.load());
}

pub fn run() {
    first("First example", include_str!("data/day14/ex1")); // 136
    first("First", include_str!("data/day14/input")); // 108 792
    second("Second example", include_str!("data/day14/ex1")); // 64
    second("Second", include_str!("data/day14/input")); // 99 118
}
