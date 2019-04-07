use maplit::hashmap;
use std::cmp::max;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
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

/// The overall, padded row and column indices for piece locations.
/// For example, the topmost piece on a standard board is
/// `Point { row: 1, column: 13 }`, despite the row only having one piece,
/// because there are 12 columns to the left in other rows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    row: i32,
    column: i32,
}

impl Point {
    pub fn new(row: i32, column: i32) -> Self {
        Self { row, column }
    }
}

/// The internal vector-based row and column indices for piece locations.
/// For example, the topmost piece on a standard board is
/// `IndexPair { row: 0, column: 0 }`, e.g., `rows[0][0]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IndexPair {
    row: usize,
    column: usize,
}

impl IndexPair {
    fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}

#[derive(Clone, Debug, derive_error::Error, Eq, PartialEq)]
pub enum GameError {
    /// Tried to move piece from wrong player.
    WrongPlayer,
    /// Point does not exist on board.
    OutOfBounds,
    /// Cannot make it from source point to target point.
    NoRoute,
    /// Target point is occupied by another piece.
    OccupiedTarget,
    /// Attempt to mix single spot movement and jump chains in one turn.
    Exhausted,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Piece {
    Head,
    Tail,
    LeftHand,
    RightHand,
    LeftFoot,
    RightFoot,
    Empty,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    rows: Vec<Vec<Piece>>,
    config: Config,
}

impl Board {
    pub fn new(config: Config) -> Self {
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

    pub fn serialize(&self) -> Vec<String> {
        self.rows
            .iter()
            .map(|row| {
                format!(
                    "{}{}",
                    " ".repeat(self.config.player_lines as usize * 3 + 1 - row.len()),
                    row.iter()
                        .map(|piece| format!(" {}", self.config.symbols[piece]))
                        .collect::<String>(),
                )
            })
            .collect()
    }

    pub fn draw(&self) {
        for row in self.serialize() {
            println!("{}", row)
        }
    }

    pub fn take_turn(&mut self, points: Vec<Point>, player: Piece) -> Result<(), GameError> {
        if points.len() < 2 {
            return Err(GameError::NoRoute);
        }
        if points.len() > 2 {
            let mut distances = Vec::<i32>::new();
            for (index, point) in points[1..].iter().enumerate() {
                let previous_point = points[index];
                let distance = match point.row - previous_point.row {
                    0 => (point.column - previous_point.column).abs() / 2,
                    x => x.abs(),
                };
                distances.push(distance);
            }
            if distances.len() > 1 && distances.iter().any(|x| *x != 2) {
                return Err(GameError::Exhausted);
            }
        }
        for (index, point) in points[1..].iter().enumerate() {
            self.move_piece(points[index], *point, player)?;
        }
        Ok(())
    }

    pub fn try_turn(&self, points: Vec<Point>, player: Piece) -> Result<(), GameError> {
        let mut test_board = self.clone();
        test_board.take_turn(points, player)
    }

    fn get_index_pair(&self, point: Point) -> Option<IndexPair> {
        let max_pieces_per_row = self.config.player_lines as usize * 3 + 1;
        let row = self.rows.get(point.row as usize - 1)?;
        let mut valid_columns = Vec::<usize>::new();
        if row.len() == 1 {
            valid_columns.push(max_pieces_per_row);
        } else {
            let offset = match row.len() % 2 {
                0 => 1 + 2 * (row.len() / 2 - 1),
                _ => 2 * ((row.len() - 1) / 2),
            };
            for x in (max_pieces_per_row - offset..=max_pieces_per_row + offset).step_by(2) {
                valid_columns.push(x);
            }
        }
        Some(IndexPair::new(
            point.row as usize - 1,
            valid_columns
                .iter()
                .position(|x| *x == point.column as usize)?,
        ))
    }

    pub fn get_piece(&self, point: Point) -> Option<Piece> {
        let pair = self.get_index_pair(point)?;
        Some(*(self.rows.get(pair.row)?.get(pair.column)?))
    }

    pub fn move_piece(
        &mut self,
        source: Point,
        target: Point,
        player: Piece,
    ) -> Result<(), GameError> {
        let distance = match source.row - target.row {
            0 => (source.column - target.column).abs() / 2,
            x => x.abs(),
        };
        if source == target || distance > 2 {
            return Err(GameError::NoRoute);
        }

        let source_piece = self.get_piece(source).ok_or(GameError::OutOfBounds)?;
        if source_piece != player {
            return Err(GameError::WrongPlayer);
        }
        let target_piece = self.get_piece(target).ok_or(GameError::OutOfBounds)?;
        if target_piece != Piece::Empty {
            return Err(GameError::OccupiedTarget);
        }

        if distance == 2 {
            let middle_piece = self
                .get_piece(Point::new(
                    max(source.row, target.row) - 1,
                    max(source.column, target.column) - 1,
                ))
                .ok_or(GameError::OutOfBounds)?;
            if middle_piece == Piece::Empty {
                return Err(GameError::NoRoute);
            }
        }

        let source_indices = self.get_index_pair(source).ok_or(GameError::OutOfBounds)?;
        let target_indices = self.get_index_pair(target).ok_or(GameError::OutOfBounds)?;
        self.rows[source_indices.row][source_indices.column] = Piece::Empty;
        self.rows[target_indices.row][target_indices.column] = player;
        Ok(())
    }

    pub fn try_move_piece(
        &self,
        source: Point,
        target: Point,
        player: Piece,
    ) -> Result<(), GameError> {
        let mut test_board = self.clone();
        test_board.move_piece(source, target, player)
    }

    pub fn has_player_won(&self, piece: Piece) -> bool {
        let pl = self.config.player_lines as usize;
        let (reversed, increasing, start) = match piece {
            Piece::Head => (false, false, pl * 3 + 1),
            Piece::LeftHand => (true, true, pl * 2 + 1),
            Piece::RightHand => (false, true, pl * 2 + 1),
            Piece::LeftFoot => (true, false, pl),
            Piece::RightFoot => (false, false, pl),
            Piece::Tail => (false, true, 0),
            Piece::Empty => return false,
        };
        for (n, row) in self.rows[start..start + pl].iter().enumerate() {
            let offset = match increasing {
                true => n + 1,
                false => self.config.player_lines as usize - n,
            };
            let row_part: Vec<&Piece> = {
                match reversed {
                    true => row.iter().rev().take(offset).collect(),
                    false => row.iter().take(offset).collect(),
                }
            };
            if row_part.iter().any(|x| *x != &piece) {
                return false;
            }
        }
        true
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Game {
    pub board: Board,
    pieces: Vec<Piece>,
}

impl Game {
    pub fn new(board: Board, pieces: Vec<Piece>) -> Self {
        Self {
            board,
            pieces,
        }
    }

    pub fn play(&mut self) -> Result<(), GameError> {
        let turns = vec![
            vec![Point::new(4, 10), Point::new(5, 11)],   // Head
            vec![Point::new(14, 16), Point::new(13, 15)], // Tail
            vec![Point::new(3, 11), Point::new(5, 13), Point::new(5, 9)], // Head
        ];

        let mut total_rounds = 0;
        let mut total_turns = 0;
        let mut playing = self.pieces.clone();
        let mut victorious = Vec::<Piece>::new();

        'outer: while playing.len() > 1 {
            for piece in playing.clone() {
                if total_turns >= turns.len() {
                    break 'outer;
                }

                println!("\nNext turn by {:?}\n", &piece);
                let turn = turns.clone()[total_turns].clone();
                self.board.take_turn(turn, piece)?;
                self.board.draw();
                if self.board.has_player_won(piece) {
                    println!("\nPlayer {:?} has finished\n", &piece);
                    playing.retain(|x| x != &piece);
                    victorious.push(piece);
                    if playing.len() < 2 {
                        break 'outer;
                    }
                }

                total_turns += 1;
            }
            total_rounds += 1;
        }

        println!("\nThe game is over!");
        println!("It lasted {} rounds", total_rounds + 1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_small_board() {
        use Piece::*;
        assert_eq!(
            Board::new(Config {
                player_lines: 1,
                ..Config::default()
            }),
            Board {
                #[cfg_attr(rustfmt, rustfmt_skip)]
                rows: vec![
                    vec![              Head                 ],
                    vec![ LeftHand, Empty, Empty, RightHand ],
                    vec![       Empty, Empty, Empty         ],
                    vec![ LeftFoot, Empty, Empty, RightFoot ],
                    vec![              Tail                 ]
                ],
                config: Config {
                    player_lines: 1,
                    ..Config::default()
                },
            },
        );
    }

    #[test]
    fn test_new_standard_board() {
        use Piece::*;
        let e = Empty;
        let lhand = LeftHand;
        let rhand = RightHand;
        let lfoot = LeftFoot;
        let rfoot = RightFoot;
        assert_eq!(
            Board::new(Config::default()),
            Board {
                #[cfg_attr(rustfmt, rustfmt_skip)]
                rows: vec![
                    vec![                                 Head                                  ],
                    vec![                              Head, Head                               ],
                    vec![                           Head, Head, Head                            ],
                    vec![                        Head, Head, Head, Head                         ],
                    vec![ lhand, lhand, lhand, lhand, e, e, e, e, e, rhand, rhand, rhand, rhand ],
                    vec![      lhand, lhand, lhand, e, e, e, e, e, e, rhand, rhand, rhand       ],
                    vec![           lhand, lhand, e, e, e, e, e, e, e, rhand, rhand             ],
                    vec![                lhand, e, e, e, e, e, e, e, e, rhand                   ],
                    vec![                     e, e, e, e, e, e, e, e, e                         ],
                    vec![                lfoot, e, e, e, e, e, e, e, e, rfoot                   ],
                    vec![           lfoot, lfoot, e, e, e, e, e, e, e, rfoot, rfoot             ],
                    vec![      lfoot, lfoot, lfoot, e, e, e, e, e, e, rfoot, rfoot, rfoot       ],
                    vec![ lfoot, lfoot, lfoot, lfoot, e, e, e, e, e, rfoot, rfoot, rfoot, rfoot ],
                    vec![                        Tail, Tail, Tail, Tail                         ],
                    vec![                           Tail, Tail, Tail                            ],
                    vec![                              Tail, Tail                               ],
                    vec![                                 Tail                                  ],
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
            vec!["    1", " 3 . . 5", "  . . .", " 6 . . 4", "    2"],
        );
    }

    #[test]
    fn test_serialize_standard_board() {
        assert_eq!(
            Board::new(Config::default()).serialize(),
            vec![
                "             1",
                "            1 1",
                "           1 1 1",
                "          1 1 1 1",
                " 3 3 3 3 . . . . . 5 5 5 5",
                "  3 3 3 . . . . . . 5 5 5",
                "   3 3 . . . . . . . 5 5",
                "    3 . . . . . . . . 5",
                "     . . . . . . . . .",
                "    6 . . . . . . . . 4",
                "   6 6 . . . . . . . 4 4",
                "  6 6 6 . . . . . . 4 4 4",
                " 6 6 6 6 . . . . . 4 4 4 4",
                "          2 2 2 2",
                "           2 2 2",
                "            2 2",
                "             2",
            ],
        );
    }

    #[test]
    fn test_try_move_piece_with_success() {
        let board = Board::default();
        assert_eq!(
            board.try_move_piece(Point::new(4, 10), Point::new(5, 11), Piece::Head),
            Ok(()),
        );
    }

    #[test]
    fn test_try_move_piece_with_no_route_because_too_far() {
        let board = Board::default();
        assert_eq!(
            board.try_move_piece(Point::new(1, 13), Point::new(7, 13), Piece::Head),
            Err(GameError::NoRoute),
        );
    }

    #[test]
    fn test_try_move_piece_with_no_route_because_no_middle_piece() {
        let board = Board::default();
        assert_eq!(
            board.try_move_piece(Point::new(4, 10), Point::new(6, 12), Piece::Head),
            Err(GameError::NoRoute),
        );
    }

    #[test]
    fn test_try_move_piece_with_target_occupied() {
        let board = Board::default();
        assert_eq!(
            board.try_move_piece(Point::new(1, 13), Point::new(2, 12), Piece::Head),
            Err(GameError::OccupiedTarget),
        );
    }

    #[test]
    fn test_try_move_piece_with_out_of_bounds() {
        let board = Board::default();
        assert_eq!(
            board.try_move_piece(Point::new(1, 13), Point::new(1, 12), Piece::Head),
            Err(GameError::OutOfBounds),
        );
    }

    #[test]
    fn test_try_move_piece_with_wrong_player() {
        let board = Board::default();
        assert_eq!(
            board.try_move_piece(Point::new(4, 10), Point::new(5, 11), Piece::Tail),
            Err(GameError::WrongPlayer),
        );
    }

    #[test]
    fn test_try_turn_with_success_on_single_move() {
        let board = Board::default();
        assert_eq!(
            board.try_turn(vec![Point::new(4, 10), Point::new(5, 11)], Piece::Head,),
            Ok(()),
        );
    }

    #[test]
    fn test_try_turn_with_success_on_single_jump() {
        let board = Board::default();
        assert_eq!(
            board.try_turn(vec![Point::new(3, 11), Point::new(5, 13)], Piece::Head,),
            Ok(()),
        );
    }

    #[test]
    fn test_try_turn_with_success_on_multiple_jumps() {
        let mut board = Board::default();
        board
            .move_piece(Point::new(4, 10), Point::new(5, 11), Piece::Head)
            .unwrap();
        assert_eq!(
            board.try_turn(
                vec![Point::new(3, 11), Point::new(5, 13), Point::new(5, 9)],
                Piece::Head,
            ),
            Ok(()),
        );
    }

    #[test]
    fn test_try_turn_with_exhaustion() {
        let board = Board::default();
        assert_eq!(
            board.try_turn(
                vec![Point::new(4, 10), Point::new(5, 11), Point::new(6, 12)],
                Piece::Head,
            ),
            Err(GameError::Exhausted),
        );
    }

    #[test]
    fn test_get_piece_from_board_with_even_player_lines() {
        let board = Board::default();

        // Out of bounds.
        assert_eq!(board.get_piece(Point::new(1, 1)), None);
        assert_eq!(board.get_piece(Point::new(1, 12)), None);

        // Row with odd number of pieces.
        assert_eq!(board.get_piece(Point::new(1, 13)), Some(Piece::Head));
        assert_eq!(board.get_piece(Point::new(5, 17)), Some(Piece::Empty));
        assert_eq!(board.get_piece(Point::new(5, 19)), Some(Piece::RightHand));

        // Row with even number of pieces.
        assert_eq!(board.get_piece(Point::new(2, 12)), Some(Piece::Head));
        assert_eq!(board.get_piece(Point::new(6, 2)), Some(Piece::LeftHand));
        assert_eq!(board.get_piece(Point::new(6, 6)), Some(Piece::LeftHand));
        assert_eq!(board.get_piece(Point::new(6, 8)), Some(Piece::Empty));
    }

    #[test]
    fn test_get_piece_from_board_with_odd_player_lines() {
        let board = Board::new(Config {
            player_lines: 3,
            ..Default::default()
        });

        // Out of bounds.
        assert_eq!(board.get_piece(Point::new(1, 1)), None);
        assert_eq!(board.get_piece(Point::new(1, 12)), None);

        // Row with odd number of pieces.
        assert_eq!(board.get_piece(Point::new(1, 10)), Some(Piece::Head));
        assert_eq!(board.get_piece(Point::new(6, 15)), Some(Piece::Empty));
        assert_eq!(board.get_piece(Point::new(6, 17)), Some(Piece::RightHand));

        // Row with even number of pieces.
        assert_eq!(board.get_piece(Point::new(2, 9)), Some(Piece::Head));
        assert_eq!(board.get_piece(Point::new(4, 1)), Some(Piece::LeftHand));
        assert_eq!(board.get_piece(Point::new(4, 5)), Some(Piece::LeftHand));
        assert_eq!(board.get_piece(Point::new(4, 7)), Some(Piece::Empty));
    }

    #[test]
    fn test_has_player_won_yes() {
        use Piece::*;
        let e = Empty;
        let board = Board {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            rows: vec![
                vec![                        Tail                       ],
                vec![                     Tail, Tail                    ],
                vec![ RightFoot, RightFoot, e, e, e, LeftFoot, LeftFoot ],
                vec![          RightFoot, e, e, e, e, LeftFoot          ],
                vec![                   e, e, e, e, e                   ],
                vec![          RightHand, e, e, e, e, LeftHand          ],
                vec![ RightHand, RightHand, e, e, e, LeftHand, LeftHand ],
                vec![                     Head, Head                    ],
                vec![                        Head                       ],
            ],
            config: Config {
                player_lines: 2,
                ..Default::default()
            },
        };
        assert!(board.has_player_won(Head));
        assert!(board.has_player_won(LeftHand));
        assert!(board.has_player_won(RightHand));
        assert!(board.has_player_won(LeftFoot));
        assert!(board.has_player_won(RightFoot));
        assert!(board.has_player_won(Tail));
    }

    #[test]
    fn test_has_player_won_no() {
        use Piece::*;
        let e = Empty;
        let board = Board {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            rows: vec![
                vec![                        Head                       ],
                vec![                     Tail, Tail                    ],
                vec![ LeftHand, RightFoot, e, e, e, LeftFoot, RightHand ],
                vec![          RightFoot, e, e, e, e, LeftFoot          ],
                vec![                   e, e, e, e, e                   ],
                vec![          RightHand, e, e, e, e, LeftHand          ],
                vec![ LeftFoot, RightHand, e, e, e, LeftHand, RightFoot ],
                vec![                     Head, Head                    ],
                vec![                        Tail                       ],
            ],
            config: Config {
                player_lines: 2,
                ..Default::default()
            },
        };
        assert!(!board.has_player_won(Head));
        assert!(!board.has_player_won(LeftHand));
        assert!(!board.has_player_won(RightHand));
        assert!(!board.has_player_won(LeftFoot));
        assert!(!board.has_player_won(RightFoot));
        assert!(!board.has_player_won(Tail));
    }
}
