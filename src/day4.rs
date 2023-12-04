use nom::{
    bytes::complete::tag,
    character,
    character::complete::{
        line_ending,
        space0,
        space1,
    },
    combinator::{
        all_consuming,
        map,
        opt,
    },
    multi::{
        many0,
        separated_list0,
    },
    sequence::{
        terminated,
        tuple,
    },
    Finish,
    IResult,
};
use std::collections::{
    HashMap,
    HashSet,
};

#[derive(Debug)]
struct Card {
    id: u16,
    winning: HashSet<u16>,
    played: Vec<u16>,
}

impl Card {
    fn matching_numbers_count(&self) -> usize {
        self.played
            .iter()
            .filter(|num| self.winning.contains(num))
            .count()
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("Card"),
                space0,
                character::complete::u16,
                space0,
                tag(":"),
                space0,
                map(
                    separated_list0(space1, character::complete::u16),
                    HashSet::from_iter,
                ),
                space0,
                tag("|"),
                space0,
                separated_list0(space1, character::complete::u16),
            )),
            |(_, _, id, _, _, _, winning, _, _, _, played)| {
                Card {
                    id,
                    winning,
                    played,
                }
            },
        )(input)
    }
}

fn parse(input: &str) -> IResult<&str, Vec<Card>> {
    all_consuming(many0(terminated(Card::parse, opt(line_ending))))(input)
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, cards) = parse(data).finish().unwrap();
    let sum: i32 = cards
        .iter()
        .map(|card| {
            let winning = card.matching_numbers_count();
            if winning == 0 {
                0
            } else {
                2_i32.pow((winning - 1) as u32)
            }
        })
        .sum();

    println!("[{}] Sum is '{}'", name, sum);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, cards) = parse(data).finish().unwrap();
    let mut card_numbers = cards
        .iter()
        .map(|card| (card.id as usize, 1_usize))
        .collect::<HashMap<_, _>>();

    for card in cards {
        let id = card.id as usize;
        let card_count = card_numbers[&id];
        for id in (id + 1_usize)..(id + 1_usize + card.matching_numbers_count()) {
            if let Some(count) = card_numbers.get_mut(&id) {
                *count += card_count;
            } else {
                card_numbers.insert(id, card_count);
            }
        }
    }

    let sum: usize = card_numbers.values().sum();

    println!("[{}] Card count is '{}'", name, sum);
}

pub fn run() {
    first("First example", include_str!("data/day4/ex1")); // 13
    first("First", include_str!("data/day4/input")); // 23441
    second("Second example", include_str!("data/day4/ex1")); // 30
    second("Second", include_str!("data/day4/input")); // 5923918
}
