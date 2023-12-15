use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let spring_groups: Vec<SpringGroup> = BufReader::new(File::open(path)?)
            .lines()
            .map_while(Result::ok)
            .map(|line| SpringGroup::from_str(line.as_str()))
            .collect::<Result<Vec<_>, _>>()?;

        println!(
            "Sum of possible states: {}",
            spring_groups
                .iter()
                .map(|spring_group| spring_group.possible_arrangements())
                .sum::<usize>()
        );

        println!(
            "Sum of possible states with unfolded groups: {}",
            spring_groups
                .iter()
                .map(|spring_group| spring_group.possible_arrangements_unfolded())
                .sum::<usize>()
        );

        Ok(())
    } else {
        Err("Usage: day12 INPUT_FILE_PATH".into())
    }
}

#[derive(Debug, Eq, PartialEq)]
struct SpringGroup {
    states: Vec<SpringState>,
    group_sizes: Vec<usize>,
}

impl SpringGroup {
    fn possible_arrangements(&self) -> usize {
        Self::possible_sub_arrangements(&self.states, &self.group_sizes)
    }

    fn possible_arrangements_unfolded(&self) -> usize {
        self.unfold().possible_arrangements()
    }

    fn unfold(&self) -> Self {
        let mut unfolded_states = self.states.clone();
        let mut unfolded_group_sizes = self.group_sizes.clone();

        for _ in 0..4 {
            unfolded_states.push(SpringState::Unknown);
            unfolded_states.extend_from_slice(self.states.as_slice());

            unfolded_group_sizes.extend_from_slice(self.group_sizes.as_slice());
        }

        Self {
            states: unfolded_states,
            group_sizes: unfolded_group_sizes,
        }
    }

    fn possible_sub_arrangements(states: &[SpringState], group_sizes: &[usize]) -> usize {
        if group_sizes.is_empty() {
            if states.iter().any(|&state| state == SpringState::Damaged) {
                // We don't want to find any more damaged springs, but there are still some
                // remaining; there are no possible arrangements to be had below this point.
                0
            } else {
                // This is a valid end state, and there's only one possible arrangement that leads
                // to success: all of the remaining unknowns are operational (or we're at the end of
                // the list of states).
                1
            }
        } else if states.len() < group_sizes.iter().sum::<usize>() + group_sizes.len() - 1 {
            // Regardless of the arrangement, there is not enough space to fit the remaining
            // groups of damaged springs
            0
        } else {
            // We may opt to strip "leading zeroes" (i.e. treat unknowns as operational springs) up
            // until we hit a known-broken spring. At that point, we're forced to start counting a
            // contiguous group.
            let mut valid_substates = 0;

            for leading_unknowns in 0..=states
                .iter()
                .take_while(|&&state| state != SpringState::Damaged)
                .filter(|&&state| state == SpringState::Unknown)
                .count()
            {
                let replaced_states =
                    Self::states_with_first_unknowns_operational(states, leading_unknowns);

                if let Some(prefix_length) =
                    Self::prefix_length_with_group_of_size(&replaced_states, group_sizes[0])
                {
                    valid_substates += Self::possible_sub_arrangements(
                        &states[prefix_length..],
                        &group_sizes[1..],
                    );
                }
            }

            valid_substates
        }
    }

    fn states_with_first_unknowns_operational(
        states: &[SpringState],
        unknowns_to_replace: usize,
    ) -> Vec<SpringState> {
        let mut replaced = 0;
        let mut replaced_states = Vec::with_capacity(states.len());

        for state in states {
            replaced_states.push(match state {
                SpringState::Operational => SpringState::Operational,
                SpringState::Damaged => SpringState::Damaged,
                SpringState::Unknown => {
                    if replaced < unknowns_to_replace {
                        replaced += 1;
                        SpringState::Operational
                    } else {
                        SpringState::Unknown
                    }
                }
            });
        }

        replaced_states
    }

    fn prefix_length_with_group_of_size(
        states: &[SpringState],
        group_size: usize,
    ) -> Option<usize> {
        if group_size == 0 {
            panic!("Group size must be positive");
        }

        if let Some(start) = states
            .iter()
            .enumerate()
            .find(|(_, state)| state != &&SpringState::Operational)
            .map(|(i, _)| i)
        {
            if start + group_size <= states.len() {
                if !states[start..start + group_size]
                    .iter()
                    .any(|&state| state == SpringState::Operational)
                {
                    // We've found a group of damaged or potentially-damaged springs; can we
                    // "terminate" the group with the end of the states, an operational spring, or
                    // an unknown spring that we can assume is operational?
                    if states.len() == start + group_size {
                        Some(states.len())
                    } else if states[start + group_size] != SpringState::Damaged {
                        Some(start + group_size + 1)
                    } else {
                        None
                    }
                } else {
                    // One or more of the springs in the group
                    None
                }
            } else {
                // There aren't enough candidates left to fill out the group
                None
            }
        } else {
            // We couldn't find any damaged or potentially-damaged springs
            None
        }
    }
}

impl FromStr for SpringGroup {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [states, groups] = string.split(' ').collect::<Vec<&str>>().as_slice() {
            let states = states
                .chars()
                .map(SpringState::try_from)
                .collect::<Result<_, _>>()?;

            let contiguous_damaged_groups = groups
                .split(',')
                .map(|count| count.parse())
                .collect::<Result<_, _>>()?;

            Ok(SpringGroup {
                states,
                group_sizes: contiguous_damaged_groups,
            })
        } else {
            Err("Could not parse line".into())
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SpringState {
    Operational,
    Damaged,
    Unknown,
}

impl TryFrom<char> for SpringState {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(SpringState::Operational),
            '#' => Ok(SpringState::Damaged),
            '?' => Ok(SpringState::Unknown),
            _ => Err("Unrecognized spring state".into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_possible_arrangements() {
        assert_eq!(
            1,
            SpringGroup::from_str("???.### 1,1,3")
                .unwrap()
                .possible_arrangements()
        );

        assert_eq!(
            4,
            SpringGroup::from_str(".??..??...?##. 1,1,3")
                .unwrap()
                .possible_arrangements()
        );

        assert_eq!(
            1,
            SpringGroup::from_str("?#?#?#?#?#?#?#? 1,3,1,6")
                .unwrap()
                .possible_arrangements()
        );

        assert_eq!(
            1,
            SpringGroup::from_str("????.#...#... 4,1,1")
                .unwrap()
                .possible_arrangements()
        );

        assert_eq!(
            4,
            SpringGroup::from_str("????.######..#####. 1,6,5")
                .unwrap()
                .possible_arrangements()
        );

        assert_eq!(
            10,
            SpringGroup::from_str("?###???????? 3,2,1")
                .unwrap()
                .possible_arrangements()
        );
    }

    #[test]
    fn test_states_with_first_unknowns_operational() {
        assert_eq!(
            states_from_str("###"),
            SpringGroup::states_with_first_unknowns_operational(&states_from_str("###"), 1)
        );

        assert_eq!(
            states_from_str("...??"),
            SpringGroup::states_with_first_unknowns_operational(&states_from_str("..???"), 1)
        );
    }

    #[test]
    fn test_prefix_length_with_group_of_size() {
        assert_eq!(
            None,
            SpringGroup::prefix_length_with_group_of_size(&states_from_str("....."), 2)
        );

        assert_eq!(
            Some(3),
            SpringGroup::prefix_length_with_group_of_size(&states_from_str("##..."), 2)
        );

        assert_eq!(
            Some(3),
            SpringGroup::prefix_length_with_group_of_size(&states_from_str("##?.."), 2)
        );

        assert_eq!(
            None,
            SpringGroup::prefix_length_with_group_of_size(&states_from_str("###..."), 2)
        );

        assert_eq!(
            Some(3),
            SpringGroup::prefix_length_with_group_of_size(&states_from_str("##?#..."), 2)
        );

        assert_eq!(
            None,
            SpringGroup::prefix_length_with_group_of_size(&states_from_str(".?##..."), 2)
        );

        assert_eq!(
            Some(4),
            SpringGroup::prefix_length_with_group_of_size(&states_from_str(".?#?..."), 2)
        );

        assert_eq!(
            Some(3),
            SpringGroup::prefix_length_with_group_of_size(&states_from_str("###"), 3)
        );
    }

    fn states_from_str(states: &str) -> Vec<SpringState> {
        states
            .chars()
            .map(|c| match c {
                '.' => SpringState::Operational,
                '#' => SpringState::Damaged,
                '?' => SpringState::Unknown,
                _ => panic!(),
            })
            .collect()
    }

    #[test]
    fn test_unfold() {
        assert_eq!(
            SpringGroup::from_str(".#?.#?.#?.#?.# 1,1,1,1,1").unwrap(),
            SpringGroup::from_str(".# 1").unwrap().unfold()
        );

        assert_eq!(
            SpringGroup::from_str("???.###????.###????.###????.###????.### 1,1,3,1,1,3,1,1,3,1,1,3,1,1,3").unwrap(),
            SpringGroup::from_str("???.### 1,1,3").unwrap().unfold()
        );
    }

    #[test]
    fn test_possible_states_unfolded() {
        assert_eq!(
            1,
            SpringGroup::from_str("???.### 1,1,3")
                .unwrap()
                .possible_arrangements_unfolded()
        );

        assert_eq!(
            16384,
            SpringGroup::from_str(".??..??...?##. 1,1,3")
                .unwrap()
                .possible_arrangements_unfolded()
        );

        assert_eq!(
            1,
            SpringGroup::from_str("?#?#?#?#?#?#?#? 1,3,1,6")
                .unwrap()
                .possible_arrangements_unfolded()
        );

        assert_eq!(
            16,
            SpringGroup::from_str("????.#...#... 4,1,1")
                .unwrap()
                .possible_arrangements_unfolded()
        );

        assert_eq!(
            2500,
            SpringGroup::from_str("????.######..#####. 1,6,5")
                .unwrap()
                .possible_arrangements_unfolded()
        );

        assert_eq!(
            506250,
            SpringGroup::from_str("?###???????? 3,2,1")
                .unwrap()
                .possible_arrangements_unfolded()
        );
    }
}
