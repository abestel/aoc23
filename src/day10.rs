use nom::{
    branch::alt,
    character,
    character::complete::line_ending,
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
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Pipe {
    first: Direction,
    second: Direction,
}

impl Pipe {
    fn new(
        first: Direction,
        second: Direction,
    ) -> Self {
        Pipe { first, second }
    }
}

#[derive(Clone, Debug)]
enum Tile {
    Pipe(Pipe),
    Ground,
    Animal,
}

impl Tile {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            value(
                Tile::Pipe(Pipe::new(Direction::North, Direction::South)),
                character::complete::char('|'),
            ),
            value(
                Tile::Pipe(Pipe::new(Direction::East, Direction::West)),
                character::complete::char('-'),
            ),
            value(
                Tile::Pipe(Pipe::new(Direction::North, Direction::East)),
                character::complete::char('L'),
            ),
            value(
                Tile::Pipe(Pipe::new(Direction::North, Direction::West)),
                character::complete::char('J'),
            ),
            value(
                Tile::Pipe(Pipe::new(Direction::South, Direction::West)),
                character::complete::char('7'),
            ),
            value(
                Tile::Pipe(Pipe::new(Direction::South, Direction::East)),
                character::complete::char('F'),
            ),
            value(Tile::Ground, character::complete::char('.')),
            value(Tile::Animal, character::complete::char('S')),
        ))(input)
    }

    fn can_connect(
        &self,
        direction: Direction,
    ) -> bool {
        match self {
            Tile::Pipe(Pipe { first, second }) => *first == direction || *second == direction,
            _ => false,
        }
    }
}

#[derive(Debug)]
struct Tiles {
    tiles: Vec<Vec<Tile>>,
}

impl Tiles {
    fn tile_at(
        &self,
        coords: (i64, i64),
    ) -> Option<&Tile> {
        let (x, y) = coords;
        if x < 0 || y < 0 {
            return None;
        }

        self.tiles
            .get(y as usize)
            .and_then(|line| line.get(x as usize))
    }

    fn pipe_at(
        &self,
        coords: (i64, i64),
    ) -> Option<Pipe> {
        self.tile_at(coords).and_then(|tile| {
            match tile {
                Tile::Pipe(pipe) => Some(*pipe),
                _ => None,
            }
        })
    }
}

#[derive(Debug)]
struct Grid {
    tiles: Tiles,
    animal_position: (i64, i64),
}

impl Grid {
    fn parse(input: &str) -> IResult<&str, Grid> {
        map(
            all_consuming(many1(terminated(many1(Tile::parse), opt(line_ending)))),
            |tiles| {
                // Find the animal in the grid
                let (x, y) = tiles
                    .iter()
                    .enumerate()
                    .find_map(|(y, line)| {
                        line.iter().enumerate().find_map(|(x, tile)| {
                            match tile {
                                Tile::Animal => Some((x as i64, y as i64)),
                                _ => None,
                            }
                        })
                    })
                    .unwrap();

                // Check the neighbours of the animal
                let mut tiles = Tiles { tiles };
                let west = tiles
                    .tile_at((x - 1, y))
                    .filter(|tile| tile.can_connect(Direction::East))
                    .map(|_| Direction::West);
                let east = tiles
                    .tile_at((x + 1, y))
                    .filter(|tile| tile.can_connect(Direction::West))
                    .map(|_| Direction::East);
                let north = tiles
                    .tile_at((x, y - 1))
                    .filter(|tile| tile.can_connect(Direction::South))
                    .map(|_| Direction::North);
                let south = tiles
                    .tile_at((x, y + 1))
                    .filter(|tile| tile.can_connect(Direction::North))
                    .map(|_| Direction::South);

                // Create the pipe and replace the animal
                let directions: Vec<_> = [west, east, north, south]
                    .iter()
                    .filter_map(|dir| *dir)
                    .collect();
                let pipe = Tile::Pipe(Pipe::new(directions[0], directions[1]));
                tiles.tiles[y as usize][x as usize] = pipe;

                Grid {
                    tiles,
                    animal_position: (x, y),
                }
            },
        )(input)
    }

    fn main_loop(&self) -> Vec<(i64, i64)> {
        let find_next_coords = |coords: (i64, i64), direction: Direction| {
            let (x, y) = coords;
            match direction {
                Direction::North => (x, y - 1),
                Direction::South => (x, y + 1),
                Direction::East => (x + 1, y),
                Direction::West => (x - 1, y),
            }
        };

        let mut visited = Vec::new();
        visited.push(self.animal_position);

        // Start at the animal position
        let start = self.tiles.pipe_at(self.animal_position).unwrap();

        // We take the first direction of the pipe arbitrarily
        let mut current_direction = start.first;
        let mut current_coords = find_next_coords(self.animal_position, current_direction);

        // And then we loop through the main loop until we come back to the animal position
        loop {
            if current_coords == self.animal_position {
                break;
            }

            visited.push(current_coords);

            // (Unsafe) get of the current pipe
            let current_pipe = self.tiles.pipe_at(current_coords).unwrap();

            // Find the next direction based on the last direction taken
            let next_direction = if current_pipe.first == current_direction.opposite() {
                current_pipe.second
            } else {
                current_pipe.first
            };

            // Get the coordinates of the next pipe
            let next = find_next_coords(current_coords, next_direction);

            current_direction = next_direction;
            current_coords = next;
        }

        visited
    }
}

fn first(
    name: &str,
    data: &str,
) {
    let (_, grid) = Grid::parse(data).finish().unwrap();
    // println!("[{}] {:#?}", name, grid);

    let main_loop = grid.main_loop();

    // Furthest point is half of the main loop size
    let furthest = main_loop.len() / 2;

    //println!("[{}] {:#?}", name, visited);
    println!("[{}] Furthest {:?}", name, furthest);
}

fn second(
    name: &str,
    data: &str,
) {
    let (_, grid) = Grid::parse(data).finish().unwrap();
    let mut main_loop = grid.main_loop();

    // Shoelace algo
    // Magic happening here
    main_loop.push(grid.animal_position);
    let sum = main_loop
        .as_slice()
        .windows(2)
        .map(|window| {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];
            x1 * y2 - y1 * x2
        })
        .sum::<i64>()
        .abs();

    let count = (sum - (main_loop.len() as i64 - 1)) / 2 + 1;

    println!("[{}] Cells inside the loop: {}", name, count);
}

pub fn run() {
    first("First example 1", include_str!("data/day10/ex1")); // 4
    first("First example 2", include_str!("data/day10/ex2")); // 8
    first("First", include_str!("data/day10/input")); // 6640
    second("Second example 3", include_str!("data/day10/ex3")); // 10
    second("Second example 4", include_str!("data/day10/ex4")); // 8
    second("Second", include_str!("data/day10/input")); // 411
}
