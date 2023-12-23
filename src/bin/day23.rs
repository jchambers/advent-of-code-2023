use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let hiking_map = {
            let mut hiking_map_string = String::new();
            File::open(path)?.read_to_string(&mut hiking_map_string)?;

            HikingMap::from_str(hiking_map_string.as_str())?
        };

        println!("Longest hike: {}", hiking_map.longest_hike());

        Ok(())
    } else {
        Err("Usage: day23 INPUT_FILE_PATH".into())
    }
}

struct HikingMap {
    width: usize,
    tiles: Vec<Tile>,
}

impl HikingMap {
    fn longest_hike(&self) -> usize {
        // Subtract 1 from the total distance because we're counting steps, not tiles visited, and
        // the starting tile doesn't count as a "step"
        self.explore_from_state(1, vec![false; self.tiles.len()])
            .unwrap()
            - 1
    }

    fn explore_from_state(
        &self,
        mut position: usize,
        mut explored_tiles: Vec<bool>,
    ) -> Option<usize> {
        loop {
            explored_tiles[position] = true;

            if position == self.tiles.len() - 2 {
                // We've reached the exit!
                return Some(explored_tiles.iter().filter(|&&t| t).count());
            }

            let mut neighbors = self.explorable_neighbor_indices(position);
            neighbors.retain(|&neighbor| !explored_tiles[neighbor]);

            if neighbors.is_empty() {
                // We've reached a dead end
                return None;
            } else if neighbors.len() == 1 {
                // Continue down the path
                position = neighbors[0];
            } else {
                // We've reached an intersection; explore all branches
                return neighbors
                    .iter()
                    .filter_map(|neighbor| {
                        self.explore_from_state(*neighbor, explored_tiles.clone())
                    })
                    .max();
            }
        }
    }

    fn explorable_neighbor_indices(&self, index: usize) -> Vec<usize> {
        let mut neighbor_indices = Vec::with_capacity(4);

        let x = index % self.width;
        let y = index / self.width;

        if x > 0
            && (self.tiles[index - 1] == Tile::Path
                || self.tiles[index - 1] == Tile::Slope(Direction::Left))
        {
            neighbor_indices.push(index - 1);
        }

        if x < self.width - 1
            && (self.tiles[index + 1] == Tile::Path
                || self.tiles[index + 1] == Tile::Slope(Direction::Right))
        {
            neighbor_indices.push(index + 1);
        }

        if y > 0
            && (self.tiles[index - self.width] == Tile::Path
                || self.tiles[index - self.width] == Tile::Slope(Direction::Up))
        {
            neighbor_indices.push(index - self.width);
        }

        if y < self.height() - 1
            && (self.tiles[index + self.width] == Tile::Path
                || self.tiles[index + self.width] == Tile::Slope(Direction::Down))
        {
            neighbor_indices.push(index + self.width);
        }

        neighbor_indices
    }

    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }
}

impl FromStr for HikingMap {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(line) = s.lines().next() {
            let width = line.len();

            let tiles: Vec<Tile> = s
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(Tile::try_from)
                .collect::<Result<_, _>>()?;

            if tiles.len() % width == 0 {
                Ok(HikingMap { width, tiles })
            } else {
                Err("Non-rectangular hiking map".into())
            }
        } else {
            Err("String contains no lines".into())
        }
    }
}

#[derive(Eq, PartialEq)]
enum Tile {
    Path,
    Forest,
    Slope(Direction),
}

impl TryFrom<char> for Tile {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Path),
            '#' => Ok(Tile::Forest),
            '^' => Ok(Tile::Slope(Direction::Up)),
            'v' => Ok(Tile::Slope(Direction::Down)),
            '<' => Ok(Tile::Slope(Direction::Left)),
            '>' => Ok(Tile::Slope(Direction::Right)),
            _ => Err("Unrecognized tile".into()),
        }
    }
}

#[derive(Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const TEST_MAP_STRING: &str = indoc! {"
        #.#####################
        #.......#########...###
        #######.#########.#.###
        ###.....#.>.>.###.#.###
        ###v#####.#v#.###.#.###
        ###.>...#.#.#.....#...#
        ###v###.#.#.#########.#
        ###...#.#.#.......#...#
        #####.#.#.#######.#.###
        #.....#.#.#.......#...#
        #.#####.#.#.#########v#
        #.#...#...#...###...>.#
        #.#.#v#######v###.###v#
        #...#.>.#...>.>.#.###.#
        #####v#.#.###v#.#.###.#
        #.....#...#...#.#.#...#
        #.#########.###.#.#.###
        #...###...#...#...#.###
        ###.###.#.###v#####v###
        #...#...#.#.>.>.#.>.###
        #.###.###.#.###.#.#v###
        #.....###...###...#...#
        #####################.#
    "};

    #[test]
    fn test_longest_hike() {
        assert_eq!(
            94,
            HikingMap::from_str(TEST_MAP_STRING).unwrap().longest_hike()
        );
    }
}
