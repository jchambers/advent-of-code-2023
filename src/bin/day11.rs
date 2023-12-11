use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let telescope_image = {
            let mut image_string = String::new();
            File::open(path)?.read_to_string(&mut image_string)?;

            TelescopeImage::from_str(image_string.as_str())?
        };

        println!(
            "Sum of shortest distances: {}",
            telescope_image.min_distance_sum()
        );

        Ok(())
    } else {
        Err("Usage: day11 INPUT_FILE_PATH".into())
    }
}

struct TelescopeImage {
    galaxies: Vec<(u32, u32)>,
}

impl TelescopeImage {
    fn min_distance_sum(&self) -> u32 {
        let expanded_galaxy_positions = self.expanded_galaxy_positions();

        (0..expanded_galaxy_positions.len() - 1)
            .flat_map(|start| {
                let (start_x, start_y) = expanded_galaxy_positions[start];

                expanded_galaxy_positions[start + 1..]
                    .iter()
                    .map(move |&(x, y)| start_x.abs_diff(x) + start_y.abs_diff(y))
            })
            .sum()
    }

    fn expanded_galaxy_positions(&self) -> Vec<(u32, u32)> {
        let empty_columns = self.empty_columns();
        let empty_rows = self.empty_rows();

        // This could all be WAY more efficient, but at the scales we're dealing with in practice,
        // it's fiiiiiiine.
        self.galaxies
            .iter()
            .map(|(x, y)| {
                let delta_x = empty_columns.iter().filter(|&c| c < x).count();

                let delta_y = empty_rows.iter().filter(|&r| r < y).count();

                (*x + delta_x as u32, *y + delta_y as u32)
            })
            .collect()
    }

    fn empty_rows(&self) -> Vec<u32> {
        Self::empty_spans(
            self.galaxies
                .iter()
                .map(|&(_, y)| y)
                .collect::<Vec<u32>>()
                .as_slice(),
        )
    }

    fn empty_columns(&self) -> Vec<u32> {
        Self::empty_spans(
            self.galaxies
                .iter()
                .map(|&(x, _)| x)
                .collect::<Vec<u32>>()
                .as_slice(),
        )
    }

    fn empty_spans(populated_positions: &[u32]) -> Vec<u32> {
        if let Some(&max) = populated_positions.iter().max() {
            let mut empty_spans = vec![true; (max + 1) as usize];

            populated_positions
                .iter()
                .for_each(|&position| empty_spans[position as usize] = false);

            empty_spans
                .iter()
                .enumerate()
                .filter(|(_, empty)| **empty)
                .map(|(position, _)| position as u32)
                .collect()
        } else {
            vec![]
        }
    }
}

impl FromStr for TelescopeImage {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let galaxies = string
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .filter(|(_, c)| c == &'#')
                    .map(move |(x, _)| (x as u32, y as u32))
            })
            .collect();

        Ok(TelescopeImage { galaxies })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const TEST_IMAGE_STRING: &str = indoc! {"
        ...#......
        .......#..
        #.........
        ..........
        ......#...
        .#........
        .........#
        ..........
        .......#..
        #...#.....
    "};

    #[test]
    fn test_empty_rows() {
        assert_eq!(
            vec![3, 7],
            TelescopeImage::from_str(TEST_IMAGE_STRING)
                .unwrap()
                .empty_rows()
        );
    }

    #[test]
    fn test_empty_columns() {
        assert_eq!(
            vec![2, 5, 8],
            TelescopeImage::from_str(TEST_IMAGE_STRING)
                .unwrap()
                .empty_columns()
        );
    }

    #[test]
    fn test_min_distance_sum() {
        assert_eq!(
            374,
            TelescopeImage::from_str(TEST_IMAGE_STRING)
                .unwrap()
                .min_distance_sum()
        );
    }
}
