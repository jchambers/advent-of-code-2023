use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let mut spring_groups: Vec<SpringGroup> = BufReader::new(File::open(path)?)
            .lines()
            .map_while(Result::ok)
            .map(|line| SpringGroup::from_str(line.as_str()))
            .collect::<Result<Vec<_>, _>>()?;

        println!(
            "Sum of possible states: {}",
            spring_groups
                .iter_mut()
                .map(|spring_group| spring_group.possible_arrangements())
                .sum::<u64>()
        );

        println!(
            "Sum of possible states with unfolded groups: {}",
            spring_groups
                .iter()
                .map(|spring_group| spring_group.possible_arrangements_unfolded())
                .sum::<u64>()
        );

        Ok(())
    } else {
        Err("Usage: day12 INPUT_FILE_PATH".into())
    }
}

struct SpringGroup {
    initial_state: SpringGroupState,
}

impl SpringGroup {
    fn possible_arrangements(&mut self) -> u64 {
        Self::possible_arrangements_from_initial_state(&self.initial_state)
    }

    fn possible_arrangements_unfolded(&self) -> u64 {
        Self::possible_arrangements_from_initial_state(&self.initial_state.unfold())
    }

    fn possible_arrangements_from_initial_state(initial_state: &SpringGroupState) -> u64 {
        let mut exploration_queue = VecDeque::from([initial_state.clone()]);
        let mut explored_transitions = HashSet::new();
        let mut paths_to_states = HashMap::new();

        paths_to_states.insert(initial_state.clone(), 1);

        while let Some(start_state) = exploration_queue.pop_front() {
            start_state
                .next_states()
                .iter()
                .for_each(|(next_state, count)| {
                    if explored_transitions.insert((start_state.clone(), next_state.clone())) {
                        let paths_to_start_state = *paths_to_states.get(&start_state).unwrap();
                        *paths_to_states.entry(next_state.clone()).or_insert(0) +=
                            paths_to_start_state * count;

                        if !next_state.is_valid_end_state() {
                            exploration_queue.push_back(next_state.clone());
                        }
                    }
                });
        }

        *paths_to_states
            .get(&SpringGroupState::success_state())
            .unwrap_or(&0)
    }
}

impl FromStr for SpringGroup {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            initial_state: SpringGroupState::from_str(string)?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct SpringGroupState {
    springs: Vec<Spring>,
    group_sizes: Vec<usize>,
}

impl SpringGroupState {
    fn success_state() -> Self {
        Self {
            springs: vec![],
            group_sizes: vec![],
        }
    }

    fn unfold(&self) -> Self {
        let mut unfolded_states = self.springs.clone();
        let mut unfolded_group_sizes = self.group_sizes.clone();

        for _ in 0..4 {
            unfolded_states.push(Spring::Unknown);
            unfolded_states.extend_from_slice(self.springs.as_slice());

            unfolded_group_sizes.extend_from_slice(self.group_sizes.as_slice());
        }

        Self {
            springs: unfolded_states,
            group_sizes: unfolded_group_sizes,
        }
    }

    fn next_states(&self) -> HashMap<SpringGroupState, u64> {
        let mut next_states = HashMap::new();

        for leading_unknowns in 0..=self
            .springs
            .iter()
            .take_while(|&&spring| spring != Spring::Damaged)
            .filter(|&&spring| spring == Spring::Unknown)
            .count()
        {
            let replaced_springs =
                Self::springs_with_first_unknowns_operational(&self.springs, leading_unknowns);

            if let Some(prefix_length) =
                Self::prefix_length_with_group_of_size(&replaced_springs, self.group_sizes[0])
            {
                // Trim "leading whitespace"
                let springs: Vec<Spring> = self.springs[prefix_length..]
                    .iter()
                    .skip_while(|spring| spring == &&Spring::Operational)
                    .copied()
                    .collect();

                let candidate_state = SpringGroupState {
                    springs,
                    group_sizes: Vec::from(&self.group_sizes[1..]),
                };

                if candidate_state.is_valid_end_state() {
                    *next_states.entry(Self::success_state()).or_insert(0) += 1;
                } else if candidate_state.is_plausible() {
                    *next_states.entry(candidate_state).or_insert(0) += 1;
                }
            }
        }

        next_states
    }

    fn is_plausible(&self) -> bool {
        if self.group_sizes.is_empty() {
            if self.springs.iter().any(|&spring| spring == Spring::Damaged) {
                // We don't want to find any more damaged springs, but there are still some
                // remaining; there are no possible arrangements to be had below this point.
                false
            } else {
                // This is a valid end state, and there's only one possible arrangement that leads
                // to success: all of the remaining unknowns are operational (or we're at the end of
                // the list of springs).
                true
            }
        } else if self.springs.len()
            < self.group_sizes.iter().sum::<usize>() + self.group_sizes.len() - 1
        {
            // Regardless of the arrangement, there is not enough space to fit the remaining
            // groups of damaged springs
            false
        } else {
            // We can't easily refute the viability of this state, so it's at least plausible and
            // worthy of further exploration
            true
        }
    }

    fn is_valid_end_state(&self) -> bool {
        self.group_sizes.is_empty() && !self.springs.iter().any(|spring| spring == &Spring::Damaged)
    }

    fn springs_with_first_unknowns_operational(
        states: &[Spring],
        unknowns_to_replace: usize,
    ) -> Vec<Spring> {
        let mut replaced = 0;
        let mut replaced_states = Vec::with_capacity(states.len());

        for state in states {
            replaced_states.push(match state {
                Spring::Operational => Spring::Operational,
                Spring::Damaged => Spring::Damaged,
                Spring::Unknown => {
                    if replaced < unknowns_to_replace {
                        replaced += 1;
                        Spring::Operational
                    } else {
                        Spring::Unknown
                    }
                }
            });
        }

        replaced_states
    }

    fn prefix_length_with_group_of_size(states: &[Spring], group_size: usize) -> Option<usize> {
        if group_size == 0 {
            panic!("Group size must be positive");
        }

        if let Some(start) = states
            .iter()
            .enumerate()
            .find(|(_, state)| state != &&Spring::Operational)
            .map(|(i, _)| i)
        {
            if start + group_size <= states.len() {
                if !states[start..start + group_size]
                    .iter()
                    .any(|&state| state == Spring::Operational)
                {
                    // We've found a group of damaged or potentially-damaged springs; can we
                    // "terminate" the group with the end of the states, an operational spring, or
                    // an unknown spring that we can assume is operational?
                    if states.len() == start + group_size {
                        Some(states.len())
                    } else if states[start + group_size] != Spring::Damaged {
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

impl FromStr for SpringGroupState {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [states, groups] = string.split(' ').collect::<Vec<&str>>().as_slice() {
            let states = states
                .chars()
                .map(Spring::try_from)
                .collect::<Result<_, _>>()?;

            let contiguous_damaged_groups = groups
                .split(',')
                .map(|count| count.parse())
                .collect::<Result<_, _>>()?;

            Ok(SpringGroupState {
                springs: states,
                group_sizes: contiguous_damaged_groups,
            })
        } else {
            Err("Could not parse line".into())
        }
    }
}

impl Display for SpringGroupState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let springs: String = self
            .springs
            .iter()
            .map(|spring| match spring {
                Spring::Operational => '.',
                Spring::Damaged => '#',
                Spring::Unknown => '?',
            })
            .collect();

        let group_sizes = self
            .group_sizes
            .iter()
            .map(|size| format!("{}", size))
            .collect::<Vec<String>>()
            .join(",");

        write!(f, "{} [{}]", springs, group_sizes)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Spring {
    Operational,
    Damaged,
    Unknown,
}

impl TryFrom<char> for Spring {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Spring::Operational),
            '#' => Ok(Spring::Damaged),
            '?' => Ok(Spring::Unknown),
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
            springs_from_str("###"),
            SpringGroupState::springs_with_first_unknowns_operational(&springs_from_str("###"), 1)
        );

        assert_eq!(
            springs_from_str("...??"),
            SpringGroupState::springs_with_first_unknowns_operational(
                &springs_from_str("..???"),
                1
            )
        );
    }

    #[test]
    fn test_prefix_length_with_group_of_size() {
        assert_eq!(
            None,
            SpringGroupState::prefix_length_with_group_of_size(&springs_from_str("....."), 2)
        );

        assert_eq!(
            Some(3),
            SpringGroupState::prefix_length_with_group_of_size(&springs_from_str("##..."), 2)
        );

        assert_eq!(
            Some(3),
            SpringGroupState::prefix_length_with_group_of_size(&springs_from_str("##?.."), 2)
        );

        assert_eq!(
            None,
            SpringGroupState::prefix_length_with_group_of_size(&springs_from_str("###..."), 2)
        );

        assert_eq!(
            Some(3),
            SpringGroupState::prefix_length_with_group_of_size(&springs_from_str("##?#..."), 2)
        );

        assert_eq!(
            None,
            SpringGroupState::prefix_length_with_group_of_size(&springs_from_str(".?##..."), 2)
        );

        assert_eq!(
            Some(4),
            SpringGroupState::prefix_length_with_group_of_size(&springs_from_str(".?#?..."), 2)
        );

        assert_eq!(
            Some(3),
            SpringGroupState::prefix_length_with_group_of_size(&springs_from_str("###"), 3)
        );
    }

    #[test]
    fn test_unfold() {
        assert_eq!(
            SpringGroupState::from_str(".#?.#?.#?.#?.# 1,1,1,1,1").unwrap(),
            SpringGroupState::from_str(".# 1").unwrap().unfold()
        );

        assert_eq!(
            SpringGroupState::from_str(
                "???.###????.###????.###????.###????.### 1,1,3,1,1,3,1,1,3,1,1,3,1,1,3"
            )
            .unwrap(),
            SpringGroupState::from_str("???.### 1,1,3")
                .unwrap()
                .unfold()
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

    fn springs_from_str(states: &str) -> Vec<Spring> {
        states
            .chars()
            .map(|c| match c {
                '.' => Spring::Operational,
                '#' => Spring::Damaged,
                '?' => Spring::Unknown,
                _ => panic!(),
            })
            .collect()
    }
}
