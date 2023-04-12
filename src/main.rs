#![allow(unused)]
use csv::{Reader, StringRecord};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::{format, Display};
use std::str::FromStr;

#[derive(Debug)]
enum GameParseError {
    Color(String),
    Archetype(String),
    Other,
    StrumError(strum::ParseError),
}

impl Error for GameParseError {}
impl From<strum::ParseError> for GameParseError {
    fn from(value: strum::ParseError) -> Self {
        Self::StrumError(value)
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, PartialOrd, strum_macros::Display, strum_macros::EnumString,
)]
enum Player {
    Grant,
    Isaac,
    Eamonn,
    Noah,
    Random,
}

impl Display for GameParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameParseError, check data")
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    strum_macros::Display,
    strum_macros::EnumString,
)]
enum ColorIdentity {
    White,
    Black,
    Red,
    Green,
    Blue,
    Uw,
    Ub,
    Ur,
    Ug,
    Rg,
    Rw,
    Rb,
    Gw,
    Gb,
    Bw,
    Naya,
    Grixis,
    Esper,
    Bant,
    Jund,
    Abzan,
    Jeskai,
    Sultai,
    Mardu,
    Temur,
    #[strum(serialize = "4c")]
    FourColor,
    #[strum(serialize = "5c")]
    FiveColor,
}

#[derive(
    Debug, Copy, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord, strum_macros::Display,
)]
enum Archetype {
    Aggro,
    Midrange,
    Combo,
    Legends,
    Toxic,
    Atraxa,
    Tempo,
    Vehicles,
    Domain,
}

impl FromStr for Archetype {
    type Err = GameParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_ref() {
            "AGGRO" => Ok(Archetype::Aggro),
            "MIDRANGE" => Ok(Archetype::Midrange),
            "COMBO" => Ok(Archetype::Combo),
            "LEGENDS" => Ok(Archetype::Legends),
            "TOXIC" => Ok(Archetype::Toxic),
            "ATRAXA" => Ok(Archetype::Atraxa),
            "TEMPO" => Ok(Archetype::Tempo),
            _ => Ok(Archetype::Midrange),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct Deck {
    color_id: ColorIdentity,
    archetype: Archetype,
}

impl Deck {
    fn new(color_id: ColorIdentity, archetype: Option<Archetype>) -> Self {
        Deck {
            color_id,
            archetype: archetype.unwrap_or(Archetype::Midrange),
        }
    }
}

impl Display for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.color_id, self.archetype)
    }
}

impl FromStr for Deck {
    type Err = GameParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(' ');
        let color_id = ColorIdentity::from_str(parts.next().unwrap_or(""))?;
        let archetype = Archetype::from_str(parts.next().unwrap_or(""))?;
        Ok(Deck {
            color_id,
            archetype,
        })
    }
}

#[derive(Debug, Deserialize)]
struct GameLog {
    player: String,
    deck: String,
    won: u32,
    lost: u32,
    opp_deck: String,
    notes: String,
}

#[derive(Debug, Copy, Clone)]
struct Matchup {
    deck: Deck,
    opponent: Deck,
    win: u32,
    loss: u32,
}

impl Matchup {
    fn new(deck: Deck, opponent: Deck) -> Self {
        Self {
            deck,
            opponent,
            win: 0,
            loss: 0,
        }
    }

    fn key(&self) -> (Deck, Deck) {
        (self.deck, self.opponent)
    }

    fn complement(&self) -> Self {
        Self {
            deck: self.opponent,
            opponent: self.deck,
            win: self.loss,
            loss: self.win,
        }
    }

    fn add(&mut self, other: Self) -> Result<&mut Self, GameParseError> {
        if self.key() == other.key() {
            self.win += other.win;
            self.loss += other.loss;
            Ok(self)
        } else {
            Err(GameParseError::Other)
        }
    }
}

impl Display for Matchup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} - {} {}",
            self.deck, self.win, self.loss, self.opponent
        )
    }
}

impl GameLog {
    fn matchups(&self) -> Vec<Matchup> {
        let mut matchups = Vec::new();
        let deck = Deck::from_str(&self.deck).ok();
        let opponent = Deck::from_str(&self.opp_deck).ok();
        let player_won = self.won > self.lost;

        match (deck, opponent) {
            (Some(player_deck), Some(opp_deck)) => {
                let matchup = Matchup {
                    deck: player_deck,
                    opponent: opp_deck,
                    win: if player_won { 1 } else { 0 },
                    loss: if player_won { 0 } else { 1 },
                };
                matchups.push(matchup.complement());
                matchups.push(matchup);
            }
            (_, _) => {
                println!("bad game log record: {:?}", self);
            }
        }
        matchups
    }
}

fn deck_record(matchups: &BTreeMap<(Deck, Deck), Matchup>, deck: Deck) {
    let (wins, losses) = matchups
        .iter()
        .filter(|(k, _)| k.0 == deck)
        .fold((0, 0), |(wins, losses), (_, matchup)| {
            (wins + matchup.win, losses + matchup.loss)
        });
    println!("{} vs. field: {} - {}", deck, wins, losses);
}

fn player_record(games: &[GameLog], player: Player) {
    let (wins, losses) = games
        .iter()
        .filter(|game| game.player == player.to_string())
        .fold((0, 0), |(wins, losses), game| {
            if game.won > game.lost {
                (wins + 1, losses)
            } else {
                (wins, losses + 1)
            }
        });
    println!("{}'s record: {} - {}", player, wins, losses);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path("data3.csv")?;
    let mut games = Vec::new();
    for row in rdr.deserialize() {
        let game: GameLog = row?;
        games.push(game);
    }
    let mut matchups: BTreeMap<(Deck, Deck), Matchup> = BTreeMap::new();

    games.iter().for_each(|game| {
        game.matchups().iter().for_each(|matchup| {
            let mut entry = matchups.entry(matchup.key()).or_insert(Matchup::new(
                matchup.deck,
                matchup.opponent,
            ));
            let result = entry.add(*matchup);
            if result.is_err() {
                eprintln!("Error adding matchup, keys not matched");
            }
        });
    });

    println!("Raw Matchup data:");
    matchups
        .values()
        .for_each(|matchup| println!("{}", matchup));

    print!("\n\n");
    deck_record(&matchups, Deck::new(ColorIdentity::White, None));
    deck_record(&matchups, Deck::new(ColorIdentity::Rb, None));
    deck_record(&matchups, Deck::new(ColorIdentity::Grixis, None));
    deck_record(&matchups, Deck::new(ColorIdentity::FiveColor, Some(Archetype::Atraxa)));
    print!("\n\n");

    player_record(&games, Player::Grant);
    player_record(&games, Player::Noah);
    player_record(&games, Player::Eamonn);
    player_record(&games, Player::Isaac);
    Ok(())
}
