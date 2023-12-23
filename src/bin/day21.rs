use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let garden_map = {
            let mut garden_map_string = String::new();
            File::open(path)?.read_to_string(&mut garden_map_string)?;

            GardenMap::from_str(garden_map_string.as_str())?
        };

        println!(
            "Garden plots reachable in 64 steps: {}",
            garden_map.reachable_garden_plots(64)
        );

        Ok(())
    } else {
        Err("Usage: day21 INPUT_FILE_PATH".into())
    }
}

struct GardenMap {
    width: usize,
    tiles: Vec<Tile>,
}

impl GardenMap {
    fn reachable_garden_plots(&self, steps: u32) -> u32 {
        let start_index = self
            .tiles
            .iter()
            .position(|t| t == &Tile::Start)
            .expect("Map must have a start tile");

        let mut distances = vec![u32::MAX; self.tiles.len()];
        let mut exploration_queue = BinaryHeap::new();

        exploration_queue.push(ExplorationQueueEntry::new(start_index, 0));

        while let Some(ExplorationQueueEntry { index, distance }) = exploration_queue.pop() {
            if distance > distances[index] {
                continue;
            }

            for neighbor_index in self.neighboring_garden_plot_indices(index) {
                let neighbor_distance = distance + 1;

                if neighbor_distance < distances[neighbor_index] {
                    distances[neighbor_index] = neighbor_distance;
                    exploration_queue.push(ExplorationQueueEntry::new(
                        neighbor_index,
                        neighbor_distance,
                    ));
                }
            }
        }

        // If a tile is within the maximum distance, the elf can just keep going back and forth
        // from an adjacent tile to "run out the clock" and hit the target number of steps as long
        // as the distance is even/odd, matching whether the target number of steps is even/odd.
        distances
            .iter()
            .filter(|&&distance| distance <= steps && distance % 2 == steps % 2)
            .count() as u32
    }

    fn neighboring_garden_plot_indices(&self, index: usize) -> Vec<usize> {
        let mut neighbor_indices = Vec::with_capacity(4);

        let x = index % self.width;
        let y = index / self.width;

        if x > 0 && self.tiles[index - 1] != Tile::Rock {
            neighbor_indices.push(index - 1);
        }

        if x < self.width - 1 && self.tiles[index + 1] != Tile::Rock {
            neighbor_indices.push(index + 1);
        }

        if y > 0 && self.tiles[index - self.width] != Tile::Rock {
            neighbor_indices.push(index - self.width);
        }

        if y < self.height() - 1 && self.tiles[index + self.width] != Tile::Rock {
            neighbor_indices.push(index + self.width);
        }

        neighbor_indices
    }

    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }
}

impl FromStr for GardenMap {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(line) = string.lines().next() {
            let width = line.len();

            let tiles: Vec<Tile> = string
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(Tile::try_from)
                .collect::<Result<_, _>>()?;

            if tiles.len() % width == 0 {
                Ok(GardenMap { width, tiles })
            } else {
                Err("Non-rectangular garden map".into())
            }
        } else {
            Err("String contains no lines".into())
        }
    }
}

#[derive(Eq, PartialEq)]
struct ExplorationQueueEntry {
    index: usize,
    distance: u32,
}

impl ExplorationQueueEntry {
    fn new(index: usize, distance: u32) -> Self {
        ExplorationQueueEntry { index, distance }
    }
}

impl Ord for ExplorationQueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse comparison for a min-heap
        other.distance.cmp(&self.distance)
    }
}

impl PartialOrd for ExplorationQueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq, PartialEq)]
enum Tile {
    GardenPlot,
    Rock,
    Start,
}

impl TryFrom<char> for Tile {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::GardenPlot),
            '#' => Ok(Tile::Rock),
            'S' => Ok(Tile::Start),
            _ => Err("Unrecognized tile".into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_reachable_garden_plots() {
        let garden_map = GardenMap::from_str(indoc! {"
            ...........
            .....###.#.
            .###.##..#.
            ..#.#...#..
            ....#.#....
            .##..S####.
            .##..#...#.
            .......##..
            .##.#.####.
            .##..##.##.
            ...........
        "})
        .unwrap();

        assert_eq!(16, garden_map.reachable_garden_plots(6));
    }
}
