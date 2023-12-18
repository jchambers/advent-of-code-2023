use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::ops::Not;
use std::str::FromStr;
use std::{env, iter};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let cooling_map = {
            let mut cooling_map_string = String::new();
            File::open(path)?.read_to_string(&mut cooling_map_string)?;

            CoolingMap::from_str(cooling_map_string.as_str())?
        };

        println!(
            "Minimum cooling along path to exit: {}",
            cooling_map.minimum_heat_loss()
        );

        Ok(())
    } else {
        Err("Usage: day17 INPUT_FILE_PATH".into())
    }
}

struct CoolingMap {
    width: usize,
    losses: Vec<u8>,
}

impl CoolingMap {
    fn minimum_heat_loss(&self) -> u32 {
        let mut exploration_queue = BinaryHeap::new();
        let mut best_cooling_values = vec![[u32::MAX, u32::MAX]; self.losses.len()];
        best_cooling_values[0] = [0, 0];

        for direction in [Direction::Horizontal, Direction::Vertical] {
            exploration_queue.push(ExplorationQueueEntry {
                destination: PositionAndDirection {
                    position: (0, 0),
                    direction,
                },
                cooling: 0,
            });
        }

        while let Some(ExplorationQueueEntry {
            destination,
            cooling,
        }) = exploration_queue.pop()
        {
            let (x, y) = destination.position;
            let index = self.index(x, y);
            let direction_index = destination.direction as usize;

            if index == self.losses.len() - 1 {
                return cooling;
            }

            if cooling > best_cooling_values[index][direction_index] {
                continue;
            }

            self.next_exploration_positions(&destination)
                .iter()
                .map(|position_and_direction| ExplorationQueueEntry {
                    destination: *position_and_direction,
                    cooling: cooling
                        + self
                            .cooling_between(destination.position, position_and_direction.position),
                })
                .for_each(|queue_entry| {
                    let (entry_x, entry_y) = queue_entry.destination.position;
                    let entry_index = self.index(entry_x, entry_y);

                    if queue_entry.cooling
                        < best_cooling_values[entry_index]
                            [queue_entry.destination.direction as usize]
                    {
                        best_cooling_values[entry_index]
                            [queue_entry.destination.direction as usize] = queue_entry.cooling;
                        exploration_queue.push(queue_entry);
                    }
                })
        }

        panic!("Rectangular, fully-connected map must have a path to exit");
    }

    fn next_exploration_positions(
        &self,
        start: &PositionAndDirection,
    ) -> Vec<PositionAndDirection> {
        let (start_x, start_y) = start.position;

        let positions: Box<dyn Iterator<Item = (usize, usize)>> = match start.direction {
            Direction::Horizontal => {
                let min_x = if start_x < 3 { 0 } else { start_x - 3 };

                let max_x = if start_x > self.width - 1 - 3 {
                    self.width - 1
                } else {
                    start_x + 3
                };

                Box::new((min_x..=max_x).zip(iter::repeat(start_y)))
            }
            Direction::Vertical => {
                let min_y = if start_y < 3 { 0 } else { start_y - 3 };

                let max_y = if start_y > self.height() - 1 - 3 {
                    self.height() - 1
                } else {
                    start_y + 3
                };

                Box::new(iter::repeat(start_x).zip(min_y..=max_y))
            }
        };

        positions
            .filter(|&(x, y)| x != start_x || y != start_y)
            .map(|position| PositionAndDirection {
                position,
                direction: !start.direction,
            })
            .collect()
    }

    fn cooling_between(&self, start: (usize, usize), destination: (usize, usize)) -> u32 {
        if start == destination {
            0
        } else {
            let (start_x, start_y) = start;
            let (destination_x, destination_y) = destination;

            let positions: Box<dyn Iterator<Item = (usize, usize)>> = if start_x == destination_x {
                Box::new(
                    iter::repeat(start_x)
                        .zip(start_y.min(destination_y)..=start_y.max(destination_y)),
                )
            } else if start_y == destination_y {
                Box::new(
                    (start_x.min(destination_x)..=start_x.max(destination_x))
                        .zip(iter::repeat(start_y)),
                )
            } else {
                panic!("Cannot calculate cooling along a non-horizontal or -vertical path")
            };

            // In a deviation from how these things often work, the start position is _not_ counted,
            // but the destination position is.
            positions
                .filter(|position| position != &start)
                .map(|(x, y)| self.losses[x + (y * self.width)] as u32)
                .sum()
        }
    }

    fn height(&self) -> usize {
        self.losses.len() / self.width
    }

    fn index(&self, x: usize, y: usize) -> usize {
        x + (y * self.width)
    }
}

impl FromStr for CoolingMap {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(line) = string.lines().next() {
            let width = line.len();

            let losses: Vec<u8> = string
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| {
                    c.to_digit(10)
                        .ok_or("Could not parse digit")
                        .map(|d| d as u8)
                })
                .collect::<Result<_, _>>()?;

            if losses.len() % width == 0 {
                Ok(CoolingMap { width, losses })
            } else {
                Err("Non-rectangular map".into())
            }
        } else {
            Err("String contains no lines".into())
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct PositionAndDirection {
    position: (usize, usize),
    direction: Direction,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Horizontal,
    Vertical,
}

impl Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Direction::Horizontal => Direction::Vertical,
            Direction::Vertical => Direction::Horizontal,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct ExplorationQueueEntry {
    destination: PositionAndDirection,
    cooling: u32,
}

impl Ord for ExplorationQueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse comparison to produce a "lowest first" priority queue
        other.cooling.cmp(&self.cooling)
    }
}

impl PartialOrd for ExplorationQueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const TEST_MAP_STRING: &str = indoc! {"
        2413432311323
        3215453535623
        3255245654254
        3446585845452
        4546657867536
        1438598798454
        4457876987766
        3637877979653
        4654967986887
        4564679986453
        1224686865563
        2546548887735
        4322674655533
    "};

    #[test]
    fn test_minimum_heat_loss() {
        let cooling_map = CoolingMap::from_str(TEST_MAP_STRING).unwrap();

        assert_eq!(102, cooling_map.minimum_heat_loss());
    }

    #[test]
    fn test_cooling_between() {
        let cooling_map = CoolingMap::from_str(TEST_MAP_STRING).unwrap();

        assert_eq!(4, cooling_map.cooling_between((0, 0), (1, 0)));
        assert_eq!(2, cooling_map.cooling_between((1, 0), (0, 0)));

        assert_eq!(8, cooling_map.cooling_between((0, 0), (3, 0)));
        assert_eq!(7, cooling_map.cooling_between((3, 0), (0, 0)));
        assert_eq!(9, cooling_map.cooling_between((0, 0), (0, 3)));
        assert_eq!(8, cooling_map.cooling_between((0, 3), (0, 0)));
    }

    #[test]
    fn test_next_exploration_positions() {
        let cooling_map = CoolingMap::from_str(TEST_MAP_STRING).unwrap();

        let next_positions = cooling_map.next_exploration_positions(&PositionAndDirection {
            position: (0, 0),
            direction: Direction::Horizontal,
        });

        assert_eq!(3, next_positions.len());

        assert!(next_positions.contains(&PositionAndDirection {
            position: (1, 0),
            direction: Direction::Vertical
        }));

        assert!(next_positions.contains(&PositionAndDirection {
            position: (2, 0),
            direction: Direction::Vertical
        }));

        assert!(next_positions.contains(&PositionAndDirection {
            position: (3, 0),
            direction: Direction::Vertical
        }));
    }
}
