use crate::Card::*;
use crate::HandType::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let hands: Vec<Hand> = BufReader::new(File::open(path)?)
            .lines()
            .map_while(Result::ok)
            .map(|line| Hand::from_str(line.as_str()))
            .collect::<Result<Vec<_>, _>>()?;

        println!("Total winnings: {}", total_winnings(&hands));

        println!(
            "Total winnings with jokers: {}",
            total_winnings_with_jokers(&hands)
        );

        Ok(())
    } else {
        Err("Usage: day07 INPUT_FILE_PATH".into())
    }
}

fn total_winnings(hands: &[Hand]) -> u64 {
    let mut sorted_hands = Vec::from_iter(hands);
    sorted_hands.sort();

    sorted_hands
        .iter()
        .rev()
        .enumerate()
        .map(|(i, hand)| (hands.len() - i) as u64 * hand.bid)
        .sum()
}

fn total_winnings_with_jokers(hands: &[Hand]) -> u64 {
    let mut sorted_hands = Vec::from_iter(hands.iter().map(|hand| hand.enhance_jokers()));

    sorted_hands.sort();

    sorted_hands
        .iter()
        .rev()
        .enumerate()
        .map(|(i, hand)| (hands.len() - i) as u64 * hand.bid)
        .sum()
}

#[derive(Debug, Eq, PartialEq)]
struct Hand {
    cards: [Card; 5],
    bid: u64,
}

impl Hand {
    fn hand_type(&self) -> HandType {
        let mut counts_by_card_type = HashMap::new();

        self.cards
            .iter()
            .for_each(|card| *counts_by_card_type.entry(card).or_insert(0u32) += 1);

        let mut non_zero_counts = counts_by_card_type
            .values()
            .filter(|&&count| count > 0)
            .copied()
            .collect::<Vec<u32>>();

        non_zero_counts.sort_by(|a, b| b.cmp(a));

        match non_zero_counts.as_slice() {
            [5] => FiveOfAKind,
            [4, 1] => FourOfAKind,
            [3, 2] => FullHouse,
            [3, 1, 1] => ThreeOfAKind,
            [2, 2, 1] => TwoPair,
            [2, 1, 1, 1] => OnePair,
            [1, 1, 1, 1, 1] => HighCard,
            _ => unreachable!(),
        }
    }

    fn enhance_jokers(&self) -> Self {
        static REPLACEMENTS: [Card; 12] = [
            Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Queen, King, Ace,
        ];

        REPLACEMENTS
            .iter()
            .map(|replacement| {
                let replaced_cards = self
                    .cards
                    .iter()
                    .map(|card| if card == &Jack { replacement } else { card })
                    .copied()
                    .collect::<Vec<Card>>()
                    .try_into()
                    .expect("Could not convert hand to an array of five cards");

                Hand {
                    cards: replaced_cards,
                    bid: self.bid,
                }
            })
            .max()
            .unwrap()
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        let hand_type_ordering = self.hand_type().cmp(&other.hand_type());

        if hand_type_ordering == Ordering::Equal {
            (0..self.cards.len())
                .map(|i| self.cards[i].cmp(&other.cards[i]))
                .find(|ordering| ordering != &Ordering::Equal)
                .unwrap_or(Ordering::Equal)
        } else {
            hand_type_ordering
        }
    }
}

impl FromStr for Hand {
    type Err = Box<dyn Error>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [cards, bid] = string.split(' ').collect::<Vec<&str>>().as_slice() {
            let cards: [Card; 5] = cards
                .chars()
                .filter_map(|c| Card::try_from(c).ok())
                .collect::<Vec<Card>>()
                .try_into()
                .map_err(|_| "Could not convert hand to an array of five cards")?;

            let bid = bid.parse()?;

            Ok(Hand { cards, bid })
        } else {
            Err("Could not parse hand".into())
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<char> for Card {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '2' => Ok(Two),
            '3' => Ok(Three),
            '4' => Ok(Four),
            '5' => Ok(Five),
            '6' => Ok(Six),
            '7' => Ok(Seven),
            '8' => Ok(Eight),
            '9' => Ok(Nine),
            'T' => Ok(Ten),
            'J' => Ok(Jack),
            'Q' => Ok(Queen),
            'K' => Ok(King),
            'A' => Ok(Ace),
            _ => Err("Unrecognized card".into()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_hand_type() {
        assert_eq!(OnePair, Hand::from_str("32T3K 765").unwrap().hand_type());
        assert_eq!(
            ThreeOfAKind,
            Hand::from_str("T55J5 684").unwrap().hand_type()
        );
        assert_eq!(TwoPair, Hand::from_str("KK677 28").unwrap().hand_type());
        assert_eq!(TwoPair, Hand::from_str("KTJJT 220").unwrap().hand_type());
        assert_eq!(
            ThreeOfAKind,
            Hand::from_str("QQQJA 483").unwrap().hand_type()
        );

        assert_eq!(FiveOfAKind, Hand::from_str("QQQQQ 1").unwrap().hand_type());
        assert_eq!(FourOfAKind, Hand::from_str("QQQQK 1").unwrap().hand_type());
        assert_eq!(FullHouse, Hand::from_str("AAA22 1").unwrap().hand_type());
        assert_eq!(HighCard, Hand::from_str("A2345 1").unwrap().hand_type());
    }

    #[test]
    fn test_total_winnings() {
        let hands: Vec<Hand> = indoc! {"
            32T3K 765
            T55J5 684
            KK677 28
            KTJJT 220
            QQQJA 483
        "}
        .lines()
        .map(Hand::from_str)
        .collect::<Result<_, _>>()
        .unwrap();

        assert_eq!(6440, total_winnings(&hands));
    }

    #[test]
    fn test_enhance_jokers() {
        assert_eq!(
            Hand::from_str("32T3K 765").unwrap(),
            Hand::from_str("32T3K 765").unwrap().enhance_jokers()
        );

        assert_eq!(
            Hand::from_str("KK677 28").unwrap(),
            Hand::from_str("KK677 28").unwrap().enhance_jokers()
        );

        assert_eq!(
            Hand::from_str("T5555 684").unwrap(),
            Hand::from_str("T55J5 684").unwrap().enhance_jokers()
        );

        assert_eq!(
            Hand::from_str("KTTTT 220").unwrap(),
            Hand::from_str("KTJJT 220").unwrap().enhance_jokers()
        );

        assert_eq!(
            Hand::from_str("QQQQA 483").unwrap(),
            Hand::from_str("QQQJA 483").unwrap().enhance_jokers()
        );

        assert_eq!(
            Hand::from_str("33336 955").unwrap(),
            Hand::from_str("J3J36 955").unwrap().enhance_jokers()
        );
    }

    #[test]
    fn test_total_winnings_with_jokers() {
        let hands: Vec<Hand> = indoc! {"
            32T3K 765
            T55J5 684
            KK677 28
            KTJJT 220
            QQQJA 483
        "}
        .lines()
        .map(Hand::from_str)
        .collect::<Result<_, _>>()
        .unwrap();

        assert_eq!(5905, total_winnings_with_jokers(&hands));
    }
}
