use nom::{
    branch::alt,
    bytes::complete::tag,
    character,
    character::complete::{
        alphanumeric1,
        line_ending,
        space0,
    },
    combinator::{
        all_consuming,
        map,
        opt,
        value,
    },
    multi::many1,
    sequence::{
        delimited,
        separated_pair,
        terminated,
        tuple,
    },
    Finish,
    IResult,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
enum Direction {
    Left,
    Right,
}

impl Direction {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            value(Direction::Left, character::complete::char('L')),
            value(Direction::Right, character::complete::char('R')),
        ))(input)
    }
}

#[derive(Clone, Debug)]
struct Node<'a> {
    label: &'a str,
    left: &'a str,
    right: &'a str,
}

impl Node<'_> {
    fn parse(input: &str) -> IResult<&str, Node<'_>> {
        map(
            separated_pair(
                alphanumeric1::<&str, nom::error::Error<&str>>,
                tuple((space0, tag("="), space0)),
                delimited(
                    tag("("),
                    separated_pair(
                        alphanumeric1,
                        tuple((space0, tag(","), space0)),
                        alphanumeric1,
                    ),
                    tag(")"),
                ),
            ),
            |(label, (left, right))| Node { label, left, right },
        )(input)
    }
}

#[derive(Debug)]
struct Network<'a> {
    directions: Vec<Direction>,
    nodes: HashMap<&'a str, Node<'a>>,
}

impl Network<'_> {
    fn parse(input: &str) -> IResult<&str, Network<'_>> {
        all_consuming(map(
            tuple((
                terminated(many1(Direction::parse), many1(line_ending)),
                many1(terminated(Node::parse, opt(line_ending))),
            )),
            |(directions, nodes)| {
                let nodes = nodes.iter().fold(HashMap::new(), move |mut map, node| {
                    map.insert(node.label, node.to_owned());
                    map
                });

                Network { directions, nodes }
            },
        ))(input)
    }

    fn node_for_label(
        &self,
        label: &str,
    ) -> &Node {
        self.nodes.get(label).unwrap()
    }

    fn follow_until<'a>(
        &'a self,
        start: &'a Node,
        stop: fn(&Node) -> bool,
    ) -> Vec<&'a str> {
        let mut node = start;
        let mut visited = Vec::new();
        for direction in self.directions.iter().cycle() {
            visited.push(node.label);

            if stop(node) {
                break;
            }

            match direction {
                Direction::Left => node = self.node_for_label(node.left),
                Direction::Right => node = self.node_for_label(node.right),
            }
        }

        visited
    }
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, network) = Network::parse(data).finish().unwrap();
    // println!("[{}] Network: {:?}", name, network);

    let visited = network.follow_until(network.node_for_label("AAA"), |node| node.label == "ZZZ");

    // println!("[{}] Visited: {:?}", name, visited);
    println!("[{}] Steps: {:?}", name, visited.len() - 1);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, network) = Network::parse(data).finish().unwrap();
    // println!("[{}] Network: {:?}", name, network);

    // For each starting node
    let nodes: Vec<_> = network
        .nodes
        .iter()
        .filter_map(|(label, node)| {
            if label.ends_with('A') {
                Some(node)
            } else {
                None
            }
        })
        .collect();

    // We compute the path for each starting node to a ending node
    let visited: Vec<Vec<&str>> = nodes
        .iter()
        .map(|node| network.follow_until(node, |node| node.label.ends_with('Z')))
        .collect();
    // println!("[{}] Visited: {:?}", name, visited);

    // And then we compute the LCM to get the moment all starting nodes are at an ending node
    let lcm = visited
        .iter()
        .fold(1, |lcm, visited| num::integer::lcm(lcm, visited.len() - 1));
    println!("[{}] Step: {}", name, lcm);
}

pub fn run() {
    first("First example", include_str!("data/day8/ex1")); // 2
    first("First example", include_str!("data/day8/ex2")); // 6
    first("First", include_str!("data/day8/input")); // 22 411
    second("Second example", include_str!("data/day8/ex3")); // 6
    second("Second", include_str!("data/day8/input")); // 11 188 774 513 823
}
