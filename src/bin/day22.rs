use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::{env, iter};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let brick_stack: BrickStack = BufReader::new(File::open(path)?)
            .lines()
            .map_while(Result::ok)
            .map(|line| Brick::from_str(line.as_str()))
            .collect::<Result<_, _>>()?;

        println!(
            "Disintegratable bricks: {}",
            brick_stack.removable_bricks().len()
        );

        println!("Falling brick sum: {}", brick_stack.disintegration_sum());

        Ok(())
    } else {
        Err("Usage: day22 INPUT_FILE_PATH".into())
    }
}

#[derive(Clone)]
struct BrickStack {
    bricks: Vec<Brick>,
}

impl BrickStack {
    fn settle_bricks(&mut self) -> usize {
        // Reverse sort order so we can easily pop the bricks closest to the ground from the end
        // of the list
        self.bricks.sort_by_key(|b| std::cmp::Reverse(b.min_z()));

        let mut settled_bricks: Vec<Brick> = Vec::with_capacity(self.bricks.len());
        let mut bricks_moved = 0;

        while let Some(mut brick) = self.bricks.pop() {
            let floor_z = settled_bricks
                .iter()
                .filter(|settled_brick| settled_brick.shares_vertical_column(&brick))
                .map(|settled_brick| settled_brick.max_z())
                .max()
                .unwrap_or(0);

            if brick.lower_to(floor_z + 1) {
                bricks_moved += 1;
            }

            settled_bricks.push(brick);
        }

        self.bricks = settled_bricks;

        bricks_moved
    }

    fn removable_bricks(&self) -> HashSet<&Brick> {
        let max_z = self
            .bricks
            .iter()
            .map(|brick| brick.max_z())
            .max()
            .unwrap_or(0) as usize;

        let mut bricks_by_top_z: Vec<Vec<&Brick>> =
            Vec::from_iter(iter::repeat_with(Vec::new).take(max_z + 1));

        self.bricks.iter().for_each(|brick| {
            bricks_by_top_z[brick.max_z() as usize].push(brick);
        });

        let mut removable_bricks: HashSet<&Brick> = HashSet::from_iter(self.bricks.iter());

        for brick in &self.bricks {
            let supporting_bricks: Vec<&&Brick> = bricks_by_top_z[brick.min_z() as usize - 1]
                .iter()
                .filter(|supporting_brick| supporting_brick.shares_vertical_column(brick))
                .collect();

            if supporting_bricks.len() == 1 {
                // This brick is supported only by a single brick, and so we can't remove that one
                // supporting brick.
                removable_bricks.remove(supporting_bricks[0]);
            }
        }

        removable_bricks
    }

    fn chaos_bricks(&self) -> HashSet<&Brick> {
        let mut chaos_bricks = HashSet::from_iter(self.bricks.iter());

        self.removable_bricks().iter().for_each(|b| {
            chaos_bricks.remove(b);
        });

        chaos_bricks
    }

    fn disintegration_sum(&self) -> usize {
        self.chaos_bricks()
            .iter()
            .map(|removable_brick| {
                let mut cloned_stack = self.clone();

                cloned_stack.bricks.remove(
                    cloned_stack
                        .bricks
                        .iter()
                        .position(|b| &b == removable_brick)
                        .expect("Cloned stack must contain removable brick"),
                );

                cloned_stack.settle_bricks()
            })
            .sum()
    }
}

impl FromIterator<Brick> for BrickStack {
    fn from_iter<T: IntoIterator<Item = Brick>>(iter: T) -> Self {
        let mut brick_stack = BrickStack {
            bricks: iter.into_iter().collect(),
        };

        brick_stack.settle_bricks();

        brick_stack
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Brick {
    ends: [(u32, u32, u32); 2],
}

impl Brick {
    fn min_z(&self) -> u32 {
        self.ends[0].2.min(self.ends[1].2)
    }

    fn max_z(&self) -> u32 {
        self.ends[0].2.max(self.ends[1].2)
    }

    fn lower_to(&mut self, target_z: u32) -> bool {
        debug_assert!(
            target_z <= self.min_z(),
            "Brick already below target Z coordinate"
        );

        debug_assert_ne!(0, target_z, "Cannot lower brick below ground level");

        let delta_z = self.min_z() - target_z;

        self.ends[0].2 -= delta_z;
        self.ends[1].2 -= delta_z;

        delta_z > 0
    }

    fn shares_vertical_column(&self, other: &Brick) -> bool {
        let self_x_range = (
            self.ends[0].0.min(self.ends[1].0),
            self.ends[0].0.max(self.ends[1].0),
        );
        let other_x_range = (
            other.ends[0].0.min(other.ends[1].0),
            other.ends[0].0.max(other.ends[1].0),
        );

        let self_y_range = (
            self.ends[0].1.min(self.ends[1].1),
            self.ends[0].1.max(self.ends[1].1),
        );
        let other_y_range = (
            other.ends[0].1.min(other.ends[1].1),
            other.ends[0].1.max(other.ends[1].1),
        );

        Self::range_overlaps(self_x_range, other_x_range)
            && Self::range_overlaps(self_y_range, other_y_range)
    }

    fn range_overlaps(a: (u32, u32), b: (u32, u32)) -> bool {
        a.0 <= b.1 && a.1 >= b.0
    }
}

impl FromStr for Brick {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let [x1, y1, z1, x2, y2, z2] = s
            .replace('~', ",")
            .split(',')
            .collect::<Vec<&str>>()
            .as_slice()
        {
            Ok(Brick {
                ends: [
                    (x1.parse()?, y1.parse()?, z1.parse()?),
                    (x2.parse()?, y2.parse()?, z2.parse()?),
                ],
            })
        } else {
            Err("Could not parse brick definition".into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const TEST_BRICKS_STRING: &str = indoc! {"
        1,0,1~1,2,1
        0,0,2~2,0,2
        0,2,3~2,2,3
        0,0,4~0,2,4
        2,0,5~2,2,5
        0,1,6~2,1,6
        1,1,8~1,1,9
    "};

    #[test]
    fn test_removable_bricks() {
        let brick_stack: BrickStack = TEST_BRICKS_STRING
            .lines()
            .map(Brick::from_str)
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(5, brick_stack.removable_bricks().len());
    }

    #[test]
    fn test_disintegration_sum() {
        let brick_stack: BrickStack = TEST_BRICKS_STRING
            .lines()
            .map(Brick::from_str)
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(7, brick_stack.disintegration_sum());
    }
}
