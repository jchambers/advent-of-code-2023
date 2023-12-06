use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        {
            let races = {
                let mut races_string = String::new();
                File::open(path)?.read_to_string(&mut races_string)?;

                Race::races_from_str(races_string.as_str())?
            };

            println!(
                "Product of ways to beat record: {}",
                races
                    .iter()
                    .map(|race| race.ways_to_beat_record())
                    .product::<u32>()
            );
        }

        {
            let race = {
                let mut races_string = String::new();
                File::open(path)?.read_to_string(&mut races_string)?;

                Race::long_race_from_str(races_string.as_str())?
            };

            println!(
                "Ways to beat record in a long race: {}",
                race.ways_to_beat_record()
            );
        }

        Ok(())
    } else {
        Err("Usage: day06 INPUT_FILE_PATH".into())
    }
}

struct Race {
    time: u64,
    distance: u64,
}

impl Race {
    fn races_from_str(string: &str) -> Result<Vec<Race>, Box<dyn Error>> {
        let mut lines = string.lines();

        let times: Vec<u64> = if let Some(times_line) = lines.next() {
            if let Some(times) = times_line.strip_prefix("Time:") {
                times
                    .split(' ')
                    .filter(|time| !time.is_empty())
                    .map(|time| time.parse())
                    .collect::<Result<_, _>>()?
            } else {
                return Err("Could not parse times".into());
            }
        } else {
            return Err("Could not find line with times".into());
        };

        let distances: Vec<u64> = if let Some(times_line) = lines.next() {
            if let Some(times) = times_line.strip_prefix("Distance:") {
                times
                    .split(' ')
                    .filter(|time| !time.is_empty())
                    .map(|time| time.parse())
                    .collect::<Result<_, _>>()?
            } else {
                return Err("Could not parse distances".into());
            }
        } else {
            return Err("Could not find distances line".into());
        };

        Ok(times
            .iter()
            .zip(distances.iter())
            .map(|(&time, &distance)| Race { time, distance })
            .collect())
    }

    fn long_race_from_str(string: &str) -> Result<Race, Box<dyn Error>> {
        let mut lines = string.lines();

        let time: u64 = if let Some(time_line) = lines.next() {
            if let Some(times) = time_line.strip_prefix("Time:") {
                times.replace(' ', "").parse()?
            } else {
                return Err("Could not parse times".into());
            }
        } else {
            return Err("Could not find line with times".into());
        };

        let distance: u64 = if let Some(distance_line) = lines.next() {
            if let Some(times) = distance_line.strip_prefix("Distance:") {
                times.replace(' ', "").parse()?
            } else {
                return Err("Could not parse distance".into());
            }
        } else {
            return Err("Could not find distance line".into());
        };

        Ok(Race { time, distance })
    }

    fn ways_to_beat_record(&self) -> u32 {
        (1..self.time)
            .map(|charge_time| charge_time * (self.time - charge_time))
            .filter(|&distance| distance > self.distance)
            .count() as u32
    }
}

#[cfg(test)]
mod test {
    use crate::Race;
    use indoc::indoc;

    const TEST_RACES_STRING: &str = indoc! {"
        Time:      7  15   30
        Distance:  9  40  200
    "};

    #[test]
    fn test_ways_to_beat_record() {
        let races = Race::races_from_str(TEST_RACES_STRING).unwrap();

        assert_eq!(4, races.get(0).unwrap().ways_to_beat_record());
        assert_eq!(8, races.get(1).unwrap().ways_to_beat_record());
        assert_eq!(9, races.get(2).unwrap().ways_to_beat_record());
    }

    #[test]
    fn test_ways_to_beat_record_long_race() {
        let race = Race::long_race_from_str(TEST_RACES_STRING).unwrap();

        assert_eq!(71503, race.ways_to_beat_record());
    }
}
