use maplit::hashmap;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Config {
    player_lines: i32,
    symbols: HashMap<Piece, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            player_lines: 4,
            symbols: hashmap!(
                Piece::Head => String::from("1"),
                Piece::Tail => String::from("2"),
                Piece::LeftHand => String::from("3"),
                Piece::RightHand => String::from("5"),
                Piece::LeftFoot => String::from("6"),
                Piece::RightFoot => String::from("4"),
                Piece::Empty => String::from("."),
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Piece {
    Head,
    Tail,
    LeftHand,
    RightHand,
    LeftFoot,
    RightFoot,
    Empty,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Board {
    rows: Vec<Vec<Piece>>,
    config: Config,
}

impl Board {
    fn new(config: Config) -> Self {
        let player_lines = config.player_lines;
        Self {
            rows: {
                let mut rows = Vec::<Vec<Piece>>::new();

                for n in 1..=player_lines {
                    let mut row = Vec::<Piece>::new();
                    for _ in 0..n {
                        row.push(Piece::Head);
                    }
                    rows.push(row);
                }

                for n in 1..=player_lines {
                    let mut row = Vec::<Piece>::new();
                    for _ in 0..player_lines + 1 - n {
                        row.push(Piece::LeftHand);
                    }
                    for _ in 0..player_lines + n {
                        row.push(Piece::Empty);
                    }
                    for _ in 0..player_lines + 1 - n {
                        row.push(Piece::RightHand);
                    }
                    rows.push(row);
                }

                rows.push(vec![Piece::Empty; player_lines as usize * 2 + 1]);

                for n in (1..=player_lines).rev() {
                    let mut row = Vec::<Piece>::new();
                    for _ in 0..player_lines + 1 - n {
                        row.push(Piece::LeftFoot);
                    }
                    for _ in 0..player_lines + n {
                        row.push(Piece::Empty);
                    }
                    for _ in 0..player_lines + 1 - n {
                        row.push(Piece::RightFoot);
                    }
                    rows.push(row);
                }

                for n in (1..=player_lines).rev() {
                    let mut row = Vec::<Piece>::new();
                    for _ in 0..n {
                        row.push(Piece::Tail);
                    }
                    rows.push(row);
                }

                rows
            },
            config,
        }
    }

    fn serialize(&self) -> Vec<String> {
        self.rows
            .iter()
            .map(|row| {
                format!(
                    "{}{}",
                    " ".repeat(self.config.player_lines as usize * 3 + 2 - row.len()),
                    row.iter()
                        .map(|piece| format!(" {}", self.config.symbols[piece]))
                        .collect::<String>(),
                )
            })
            .collect()
    }

    fn draw(&self) {
        for row in self.serialize() {
            println!("{}", row)
        }
    }
}

fn main() {
    let config = Config::default();
    let board = Board::new(config);
    board.draw();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_board() {
        use Piece::*;
        assert_eq!(
            Board::new(Config {
                player_lines: 1,
                ..Config::default()
            }),
            Board {
                rows: vec![
                    vec![Head],
                    vec![LeftHand, Empty, Empty, RightHand],
                    vec![Empty, Empty, Empty],
                    vec![LeftFoot, Empty, Empty, RightFoot],
                    vec![Tail]
                ],
                config: Config {
                    player_lines: 1,
                    ..Config::default()
                },
            },
        );
    }

    #[test]
    fn test_standard_board() {
        use Piece::*;
        assert_eq!(
            Board::new(Config::default()),
            Board {
                rows: vec![
                    vec![Head],
                    vec![Head, Head],
                    vec![Head, Head, Head],
                    vec![Head, Head, Head, Head],
                    vec![
                        LeftHand, LeftHand, LeftHand, LeftHand, Empty, Empty, Empty, Empty, Empty,
                        RightHand, RightHand, RightHand, RightHand,
                    ],
                    vec![
                        LeftHand, LeftHand, LeftHand, Empty, Empty, Empty, Empty, Empty, Empty,
                        RightHand, RightHand, RightHand,
                    ],
                    vec![
                        LeftHand, LeftHand, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                        RightHand, RightHand,
                    ],
                    vec![
                        LeftHand, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                        RightHand,
                    ],
                    vec![Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty],
                    vec![
                        LeftFoot, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                        RightFoot,
                    ],
                    vec![
                        LeftFoot, LeftFoot, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                        RightFoot, RightFoot,
                    ],
                    vec![
                        LeftFoot, LeftFoot, LeftFoot, Empty, Empty, Empty, Empty, Empty, Empty,
                        RightFoot, RightFoot, RightFoot,
                    ],
                    vec![
                        LeftFoot, LeftFoot, LeftFoot, LeftFoot, Empty, Empty, Empty, Empty, Empty,
                        RightFoot, RightFoot, RightFoot, RightFoot,
                    ],
                    vec![Tail, Tail, Tail, Tail],
                    vec![Tail, Tail, Tail],
                    vec![Tail, Tail],
                    vec![Tail]
                ],
                config: Config::default(),
            }
        )
    }

    #[test]
    fn test_serialize_small_board() {
        assert_eq!(
            Board::new(Config {
                player_lines: 1,
                ..Config::default()
            })
            .serialize(),
            vec!["     1", "  3 . . 5", "   . . .", "  6 . . 4", "     2"],
        );
    }

    #[test]
    fn test_serialize_standard_board() {
        assert_eq!(
            Board::new(Config::default()).serialize(),
            vec![
                "              1",
                "             1 1",
                "            1 1 1",
                "           1 1 1 1",
                "  3 3 3 3 . . . . . 5 5 5 5",
                "   3 3 3 . . . . . . 5 5 5",
                "    3 3 . . . . . . . 5 5",
                "     3 . . . . . . . . 5",
                "      . . . . . . . . .",
                "     6 . . . . . . . . 4",
                "    6 6 . . . . . . . 4 4",
                "   6 6 6 . . . . . . 4 4 4",
                "  6 6 6 6 . . . . . 4 4 4 4",
                "           2 2 2 2",
                "            2 2 2",
                "             2 2",
                "              2",
            ],
        );
    }
}
