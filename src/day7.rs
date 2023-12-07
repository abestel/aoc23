use nom::{
    character,
    character::complete::{
        line_ending,
        space1,
    },
    combinator::{
        all_consuming,
        map,
        opt,
    },
    multi::many1,
    sequence::{
        separated_pair,
        terminated,
    },
    Finish,
    IResult,
};
use phf::{
    map::Map,
    phf_map,
};
use std::{
    cmp::Ordering,
    collections::HashMap,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HandType {
    FiveOfAKind,
    FourOfAKind,
    FullHouse,
    ThreeOfAKind,
    TwoPair,
    OnePair,
    HighCard,
}

impl HandType {
    fn order(&self) -> u8 {
        match self {
            HandType::FiveOfAKind => 7,
            HandType::FourOfAKind => 6,
            HandType::FullHouse => 5,
            HandType::ThreeOfAKind => 4,
            HandType::TwoPair => 3,
            HandType::OnePair => 2,
            HandType::HighCard => 1,
        }
    }
}

impl Ord for HandType {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.order().cmp(&other.order())
    }
}

impl PartialOrd for HandType {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum Card {
    As,
    King,
    Queen,
    Jack,
    Ten,
    Nine,
    Eight,
    Seven,
    Six,
    Five,
    Four,
    Three,
    Two,
    Joker,
}

impl Card {
    fn parse<'a>(
        char_to_card: &'a Map<char, Card>
    ) -> impl FnMut(&'a str) -> IResult<&'a str, Card> {
        map(
            character::complete::satisfy(|c| char_to_card.contains_key(&c)),
            |c| char_to_card[&c],
        )
    }

    fn order(&self) -> u8 {
        match self {
            Card::As => 13,
            Card::King => 12,
            Card::Queen => 11,
            Card::Jack => 10,
            Card::Ten => 9,
            Card::Nine => 8,
            Card::Eight => 7,
            Card::Seven => 6,
            Card::Six => 5,
            Card::Five => 4,
            Card::Four => 3,
            Card::Three => 2,
            Card::Two => 1,
            Card::Joker => 0,
        }
    }
}

impl Ord for Card {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.order().cmp(&other.order())
    }
}

impl PartialOrd for Card {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Hand {
    cards: Vec<Card>,
    bid: u64,
}

impl Hand {
    fn parse<'a>(
        char_to_card: &'a Map<char, Card>
    ) -> impl FnMut(&'a str) -> IResult<&'a str, Self> {
        map(
            separated_pair(
                many1(Card::parse(char_to_card)),
                space1,
                character::complete::u64,
            ),
            |(cards, bid)| Hand { cards, bid },
        )
    }

    fn hand_type(&self) -> HandType {
        let mut card_counts =
            self.cards
                .iter()
                .fold(HashMap::<Card, u8>::new(), |mut card_counts, card| {
                    card_counts
                        .entry(*card)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);

                    card_counts
                });

        // Distribute the joker to the biggest group, if we don't have only jokers
        if card_counts.len() > 1 {
            if let Some(jokers) = card_counts.remove(&Card::Joker) {
                if let Some((card, _)) = card_counts.iter().max_by_key(|(_, count)| **count) {
                    card_counts
                        .entry(*card)
                        .and_modify(|count| *count += jokers);
                }
            }
        }

        match card_counts.len() {
            1 => HandType::FiveOfAKind,
            2 => {
                if card_counts.values().any(|count| *count == 4) {
                    HandType::FourOfAKind
                } else {
                    HandType::FullHouse
                }
            }
            3 => {
                if card_counts.values().any(|count| *count == 3) {
                    HandType::ThreeOfAKind
                } else {
                    HandType::TwoPair
                }
            }
            4 => HandType::OnePair,
            _ => HandType::HighCard,
        }
    }
}

impl Ord for Hand {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        let hand_type_ord = self.hand_type().cmp(&other.hand_type());
        match hand_type_ord {
            Ordering::Equal => self.cards.cmp(&other.cards),
            _ => hand_type_ord,
        }
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

const CHAR_TO_CARD: Map<char, Card> = phf_map! {
    'A' => Card::As,
    'K' => Card::King,
    'Q' => Card::Queen,
    'J' => Card::Jack,
    'T' => Card::Ten,
    '9' => Card::Nine,
    '8' => Card::Eight,
    '7' => Card::Seven,
    '6' => Card::Six,
    '5' => Card::Five,
    '4' => Card::Four,
    '3' => Card::Three,
    '2' => Card::Two
};

const CHAR_TO_CARD_2: Map<char, Card> = phf_map! {
    'A' => Card::As,
    'K' => Card::King,
    'Q' => Card::Queen,
    'J' => Card::Joker,
    'T' => Card::Ten,
    '9' => Card::Nine,
    '8' => Card::Eight,
    '7' => Card::Seven,
    '6' => Card::Six,
    '5' => Card::Five,
    '4' => Card::Four,
    '3' => Card::Three,
    '2' => Card::Two
};

fn parse_hands<'a>(
    char_to_card: &'a Map<char, Card>
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Hand>> {
    all_consuming(many1(terminated(
        Hand::parse(char_to_card),
        opt(line_ending),
    )))
}

fn total_winnings(
    name: &str,
    data: &str,
    char_to_card: &Map<char, Card>,
) {
    let (_, mut hands) = parse_hands(char_to_card)(data).finish().unwrap();
    hands.sort();

    let total: u64 = hands
        .iter()
        .enumerate()
        .map(|(rank, hand)| (rank as u64 + 1) * hand.bid)
        .sum();

    println!("[{}] Total winnings: {:?}", name, total);
}

fn first(
    name: &str,
    data: &str,
) {
    total_winnings(name, data, &CHAR_TO_CARD)
}

fn second(
    name: &str,
    data: &str,
) {
    total_winnings(name, data, &CHAR_TO_CARD_2)
}

pub fn run() {
    first("First example", include_str!("data/day7/ex1")); // 6440
    first("First", include_str!("data/day7/input")); // 248569531
    second("Second example", include_str!("data/day7/ex1")); // 5905
    second("Second", include_str!("data/day7/input")); // 250382098
}
