use nom::{
    character::complete::{
        line_ending,
        satisfy,
    },
    combinator::{
        all_consuming,
        map,
    },
    multi::many1,
    sequence::terminated,
    Finish,
    IResult,
};
use std::{
    cmp::Ordering,
    collections::{
        BinaryHeap,
        HashMap,
    },
};

#[derive(Debug)]
struct Grid {
    points: Vec<Vec<u8>>,
}

impl Grid {
    fn parse(input: &str) -> IResult<&str, Self> {
        all_consuming(map(
            many1(terminated(
                many1(map(satisfy(|c| c.is_numeric()), |c| c as u8 - b'0')),
                line_ending,
            )),
            |points| Self { points },
        ))(input)
    }

    fn shortest_path(
        &self,
        start: (i32, i32),
        end: (i32, i32),
        min_step: u8,
        max_step: u8,
    ) -> Option<u32> {
        let mut distances = HashMap::<Key, u32>::new();
        let mut heap = BinaryHeap::new();

        // Initialize
        for direction in [Direction::Down, Direction::Right] {
            let state = State {
                cost: 0,
                coords: start,
                direction,
                steps: 0,
            };

            distances.insert(state.into(), state.cost);
            heap.push(state);
        }

        while let Some(
            state @ State {
                cost,
                coords,
                direction,
                steps,
            },
        ) = heap.pop()
        {
            // We reached the final point
            if coords == end && steps >= min_step {
                return Some(cost);
            }

            // Otherwise check if we got a better distance
            if distances
                .get(&state.into())
                .is_some_and(|current_cost| *current_cost < state.cost)
            {
                continue;
            }

            for (next_direction, (x, y)) in self.adjacent(coords, direction) {
                let next = State {
                    cost: cost + self.points[y as usize][x as usize] as u32,
                    coords: (x, y),
                    direction: next_direction,
                    steps: if next_direction == direction {
                        steps + 1
                    } else {
                        1
                    },
                };

                if
                // We have too long of a streak
                next.steps > max_step ||
                    // We already have a shorter path
                    distances.get(&next.into()).is_some_and(|current_cost| *current_cost <= next.cost) ||
                    // The streak is too short
                    (next.direction != direction && steps < min_step)
                {
                    continue;
                }

                //println!("{:?} -> {:?}", coords, next.coords);

                // We continue checking paths and register the distances
                heap.push(next);
                distances.insert(next.into(), next.cost);
            }
        }

        None
    }

    fn contains(
        &self,
        coords: (i32, i32),
    ) -> bool {
        let (x, y) = coords;
        0 <= x
            && x < self
                .points
                .first()
                .map(|line| line.len())
                .unwrap_or_default() as i32
            && 0 <= y
            && y < self.points.len() as i32
    }

    fn adjacent(
        &self,
        coords: (i32, i32),
        coming_from: Direction,
    ) -> Vec<(Direction, (i32, i32))> {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .iter()
        .filter_map(|direction| {
            // Don't want to go back
            if *direction == coming_from.opposite() {
                None
            } else {
                let next = direction.next(coords);
                // Filter out points outside of the grid
                if self.contains(next) {
                    Some((*direction, next))
                } else {
                    None
                }
            }
        })
        .collect()
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
    fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct State {
    cost: u32,
    coords: (i32, i32),
    direction: Direction,
    steps: u8,
}

impl PartialOrd<Self> for State {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Key {
    coords: (i32, i32),
    direction: Direction,
    steps: u8,
}

impl From<State> for Key {
    fn from(value: State) -> Self {
        Key {
            coords: value.coords,
            direction: value.direction,
            steps: value.steps,
        }
    }
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, grid) = Grid::parse(data).finish().unwrap();
    let result = grid.shortest_path(
        (0, 0),
        (
            grid.points[0].len() as i32 - 1,
            grid.points.len() as i32 - 1,
        ),
        1,
        3,
    );
    println!("[{}] Shortest path: {:?}", name, result);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, grid) = Grid::parse(data).finish().unwrap();
    let result = grid.shortest_path(
        (0, 0),
        (
            grid.points[0].len() as i32 - 1,
            grid.points.len() as i32 - 1,
        ),
        4,
        10,
    );
    println!("[{}] Shortest path: {:?}", name, result);
}

pub fn run() {
    first("First example", include_str!("data/day17/ex1")); // 102
    first("First", include_str!("data/day17/input")); // 1263
    second("Second example", include_str!("data/day17/ex1")); // 94
    second("Second", include_str!("data/day17/input")); // 94
}
