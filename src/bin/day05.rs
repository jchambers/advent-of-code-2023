use std::collections::HashSet;
use crate::Resource::*;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::ops::Add;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let almanac = {
            let mut almanac_string = String::new();
            File::open(path)?.read_to_string(&mut almanac_string)?;

            Almanac::from_str(almanac_string.as_str())?
        };

        println!("Lowest seed location: {}", almanac.lowest_seed_location());

        println!(
            "Lowest seed location using range rules: {}",
            almanac.lowest_seed_location_ranges()
        );

        Ok(())
    } else {
        Err("Usage: day05 INPUT_FILE_PATH".into())
    }
}

struct Almanac {
    seeds: Vec<u64>,
    range_map: RangeMap,
}

impl Almanac {
    fn lowest_seed_location(&self) -> u64 {
        self.seeds
            .iter()
            .map(|seed| self.range_map.map(*seed))
            .min()
            .unwrap()
    }

    fn lowest_seed_location_ranges(&self) -> u64 {
        let mut points_of_interest = HashSet::new();

        for chunk in self.seeds.chunks_exact(2) {
            if let &[start, length] = chunk {
                // Add the start and end of the seed range as points of interest
                points_of_interest.insert(start);
                points_of_interest.insert(start + length);

                // Add any range boundaries that occur within a seed range
                self.range_map.ranges
                    .iter()
                    .flat_map(|range| vec![range.start, range.end])
                    .filter(|&point| point >= start && point < start + length)
                    .for_each(|point| {
                        points_of_interest.insert(point);
                    });
            }
        }

        points_of_interest.iter()
            .map(|&point| self.range_map.map(point))
            .min()
            .unwrap()
    }
}

impl FromStr for Almanac {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut blocks = string.split("\n\n");

        let seeds = if let Some(seeds_block) = blocks.next() {
            if let ["seeds", seeds] = seeds_block.split(": ").collect::<Vec<&str>>().as_slice() {
                seeds
                    .split(' ')
                    .map(|seed| seed.parse())
                    .collect::<Result<_, _>>()?
            } else {
                return Err("Could not find seeds block".into());
            }
        } else {
            return Err("Empty almanac".into());
        };

        let range_maps: Vec<RangeMap> = blocks.map(RangeMap::from_str).collect::<Result<_, _>>()?;

        let mut combined_range_map = range_maps
            .iter()
            .find(|range_map| range_map.source == Seed)
            .ok_or("Could not find initial seed-to-* range map")?
            .to_owned();

        while let Some(next_range_map) = range_maps
            .iter()
            .find(|&next_range_map| next_range_map.source == combined_range_map.destination) {

            combined_range_map = &combined_range_map + next_range_map;
        }

        Ok(Almanac { seeds, range_map: combined_range_map })
    }
}

#[derive(Clone)]
struct RangeMap {
    source: Resource,
    destination: Resource,

    ranges: Vec<Range>,
}

impl RangeMap {
    fn map(&self, value: u64) -> u64 {
        self.ranges
            .iter()
            .filter_map(|range| range.map(value))
            .next()
            .unwrap_or(value)
    }

    // // What input value leads to the given output value?
    fn invert(&self, value: u64) -> u64 {
        self.ranges
            .iter()
            .filter_map(|range| range.invert(value))
            .next()
            .unwrap_or(value)
    }
}

impl FromStr for RangeMap {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut lines = string.lines();

        let (source, destination) = if let [map_type, "map:"] = lines
            .next()
            .ok_or::<&str>("Could not parse range map header")?
            .split(' ')
            .collect::<Vec<&str>>()
            .as_slice()
        {
            if let [source, destination] = map_type.split("-to-").collect::<Vec<&str>>().as_slice()
            {
                (
                    Resource::from_str(source)?,
                    Resource::from_str(destination)?,
                )
            } else {
                return Err("Could not parse map type".into());
            }
        } else {
            return Err("Could not parse range map header".into());
        };

        Ok(RangeMap {
            source,
            destination,

            ranges: lines.map(Range::from_str).collect::<Result<_, _>>()?,
        })
    }
}

impl Add for &RangeMap {
    type Output = RangeMap;

    fn add(self, addend: Self) -> Self::Output {
        if self.destination != addend.source {
            panic!("Incompatible resource types");
        }

        let mut boundaries = HashSet::new();
        boundaries.insert(0);
        boundaries.insert(u64::MAX);

        self.ranges
            .iter()
            .for_each(|range| {
                boundaries.insert(range.start);
                boundaries.insert(range.end);
            });

        addend.ranges
            .iter()
            .for_each(|range| {
                boundaries.insert(self.invert(range.start));
                boundaries.insert(self.invert(range.end));
            });

        let mut boundaries = Vec::from_iter(boundaries);
        boundaries.sort();

        let ranges = boundaries.windows(2)
            .filter_map(|window| if let [start, end] = window {
                let offset = addend.map(self.map(*start)) as i64 - *start as i64;

                Some(Range { start: *start, end: *end, offset })
            } else {
                None
            })
            .collect();

        // TODO: "Defragment" ranges by merging adjacent ranges with identical offsets

        RangeMap {
            source: self.source,
            destination: addend.destination,

            ranges
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Resource {
    Seed,
    Soil,
    Fertilizer,
    Water,
    Light,
    Temperature,
    Humidity,
    Location,
}

impl FromStr for Resource {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "seed" => Ok(Seed),
            "soil" => Ok(Soil),
            "fertilizer" => Ok(Fertilizer),
            "water" => Ok(Water),
            "light" => Ok(Light),
            "temperature" => Ok(Temperature),
            "humidity" => Ok(Humidity),
            "location" => Ok(Location),
            _ => Err("Unrecognized resource".into()),
        }
    }
}

#[derive(Copy, Clone)]
struct Range {
    start: u64,

    // End exclusive
    end: u64,

    offset: i64,
}

impl Range {
    fn map(&self, value: u64) -> Option<u64> {
        if value >= self.start && value < self.end {
            Some((value as i64 + self.offset) as u64)
        } else {
            None
        }
    }

    // What input value, if any, leads to the given output value?
    fn invert(&self, value: u64) -> Option<u64> {
        if value >= (self.start as i64 + self.offset) as u64 && value < (self.end as i64 + self.offset) as u64 {
            Some((value as i64 - self.offset) as u64)
        } else {
            None
        }
    }
}

impl FromStr for Range {
    type Err = Box<dyn Error>;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if let [destination_start, source_start, length] =
            line.split(' ').collect::<Vec<&str>>().as_slice()
        {
            let source_start = source_start.parse()?;
            let destination_start: u64 = destination_start.parse()?;
            let length: u64 = length.parse()?;

            Ok(Range {
                start: source_start,
                end: source_start + length,
                offset: destination_start as i64 - source_start as i64,
            })
        } else {
            Err("Could not parse range string".into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const TEST_ALMANAC_STRING: &str = indoc! {"
        seeds: 79 14 55 13

        seed-to-soil map:
        50 98 2
        52 50 48

        soil-to-fertilizer map:
        0 15 37
        37 52 2
        39 0 15

        fertilizer-to-water map:
        49 53 8
        0 11 42
        42 0 7
        57 7 4

        water-to-light map:
        88 18 7
        18 25 70

        light-to-temperature map:
        45 77 23
        81 45 19
        68 64 13

        temperature-to-humidity map:
        0 69 1
        1 0 69

        humidity-to-location map:
        60 56 37
        56 93 4
    "};

    #[test]
    fn test_range_map() {
        let range_map = RangeMap::from_str(indoc! {"
            seed-to-soil map:
            50 98 2
            52 50 48
        "})
        .unwrap();

        assert_eq!(50, range_map.map(98));
        assert_eq!(51, range_map.map(99));
        assert_eq!(55, range_map.map(53));
        assert_eq!(10, range_map.map(10));
    }

    #[test]
    fn test_lowest_seed_location() {
        let almanac = Almanac::from_str(TEST_ALMANAC_STRING).unwrap();

        assert_eq!(35, almanac.lowest_seed_location());
    }

    #[test]
    fn test_range_invert() {
        {
            let range = Range::from_str("50 98 2").unwrap();

            assert_eq!(Some(99), range.invert(51));
            assert_eq!(None, range.invert(49));
        }

        {
            let range = Range::from_str("52 50 48").unwrap();

            assert_eq!(Some(50), range.invert(52));
            assert_eq!(None, range.invert(100));
        }
    }

    #[test]
    fn test_range_map_invert() {
        let range_map = RangeMap::from_str(indoc! {"
            seed-to-soil map:
            50 98 2
            52 50 48
        "})
            .unwrap();

        assert_eq!(99, range_map.invert(51));
        assert_eq!(49, range_map.invert(49));
        assert_eq!(50, range_map.invert(52));
        assert_eq!(100, range_map.invert(100));
    }

    #[test]
    fn test_range_map_add() {
        let seed_to_soil= RangeMap::from_str(indoc! {"
            seed-to-soil map:
            50 98 2
            52 50 48
        "})
            .unwrap();

        let soil_to_fertilizer = RangeMap::from_str(indoc! {"
            soil-to-fertilizer map:
            0 15 37
            37 52 2
            39 0 15
        "})
            .unwrap();

        let seed_to_fertilizer = &seed_to_soil + &soil_to_fertilizer;

        assert_eq!(Seed, seed_to_fertilizer.source);
        assert_eq!(Fertilizer, seed_to_fertilizer.destination);

        for seed in 0..=100 {
            assert_eq!(soil_to_fertilizer.map(seed_to_soil.map(seed)), seed_to_fertilizer.map(seed));
        }
    }

    #[test]
    fn test_lowest_seed_location_ranges() {
        let almanac = Almanac::from_str(TEST_ALMANAC_STRING).unwrap();

        assert_eq!(46, almanac.lowest_seed_location_ranges());
    }
}
