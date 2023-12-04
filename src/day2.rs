use nom::{
    branch::alt,
    bytes::complete::tag,
    character,
    character::complete::{
        line_ending,
        space0,
    },
    combinator::{
        all_consuming,
        map,
        opt,
        value,
    },
    multi::{
        many1,
        separated_list1,
    },
    sequence::{
        terminated,
        tuple,
    },
    Finish,
    IResult,
};

#[derive(Debug)]
struct Game {
    id: u16,
    sets: Vec<Dices>,
}

impl Game {
    fn parse(input: &str) -> IResult<&str, Game> {
        terminated(
            map(
                tuple((
                    tag("Game"),
                    space0,
                    character::complete::u16,
                    space0,
                    tag(":"),
                    separated_list1(tag(";"), Dices::parse),
                )),
                |(_, _, id, _, _, sets)| Game { id, sets },
            ),
            opt(line_ending),
        )(input)
    }
}

#[derive(Debug, Default)]
struct Dices {
    blue: u16,
    green: u16,
    red: u16,
}

impl Dices {
    fn power(&self) -> u64 {
        (self.blue as u64) * (self.green as u64) * (self.red as u64)
    }

    fn merge(
        &self,
        other: &Self,
    ) -> Self {
        Dices {
            blue: self.blue + other.blue,
            green: self.green + other.green,
            red: self.red + other.red,
        }
    }

    fn max(
        &self,
        other: &Self,
    ) -> Self {
        Dices {
            blue: self.blue.max(other.blue),
            green: self.green.max(other.green),
            red: self.red.max(other.red),
        }
    }

    fn new_with_color(
        color: Color,
        number: u16,
    ) -> Self {
        match color {
            Color::Blue => {
                Dices {
                    blue: number,
                    green: 0,
                    red: 0,
                }
            }
            Color::Green => {
                Dices {
                    blue: 0,
                    green: number,
                    red: 0,
                }
            }
            Color::Red => {
                Dices {
                    blue: 0,
                    green: 0,
                    red: number,
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Color {
    Blue,
    Green,
    Red,
}

impl Dices {
    fn parse(input: &str) -> IResult<&str, Dices> {
        map(
            many1(map(
                tuple((
                    space0,
                    character::complete::u16,
                    space0,
                    alt((
                        value(Color::Blue, tag("blue")),
                        value(Color::Green, tag("green")),
                        value(Color::Red, tag("red")),
                    )),
                    space0,
                    opt(tag(",")),
                )),
                |(_, num, _, color, _, _)| Dices::new_with_color(color, num),
            )),
            |dices| dices.iter().fold(Dices::default(), |a, b| a.merge(b)),
        )(input)
    }
}

fn parse_games(input: &str) -> IResult<&str, Vec<Game>> {
    all_consuming(many1(terminated(Game::parse, opt(line_ending))))(input)
}

fn first(
    name: &str,
    data: &str,
) {
    let global = Dices {
        blue: 14,
        green: 13,
        red: 12,
    };

    let (_, games) = parse_games(data).finish().unwrap();
    let sum_possible_games: u64 = games
        .iter()
        .filter_map(|game| {
            let has_impossible_set = game.sets.iter().any(|dice| {
                dice.blue > global.blue || dice.green > global.green || dice.red > global.red
            });

            if has_impossible_set {
                None
            } else {
                Some(game.id as u64)
            }
        })
        .sum();

    println!("[{}] Sum of possible games: '{}'", name, sum_possible_games);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, games) = parse_games(data).finish().unwrap();
    let sum_powers: u64 = games
        .iter()
        .map(|game| {
            game.sets
                .iter()
                .fold(Dices::default(), |a, b| a.max(b))
                .power()
        })
        .sum();

    println!("[{}] Sum of powers: '{}'", name, sum_powers);
}

pub fn run() {
    first("First example", include_str!("data/day2/ex1")); // 8
    first("First", include_str!("data/day2/input")); // 2528
    second("Second example", include_str!("data/day2/ex1")); // 2286
    second("Second", include_str!("data/day2/input")); // 67363
}
