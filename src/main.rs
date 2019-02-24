use stelsalto::{Board, Config, Piece, Point};

fn main() -> Result<(), Box<std::error::Error>> {
    let config = Config::default();
    let mut board = Board::new(config);
    board.draw();

    for (moves, player) in vec![
        (vec![Point::new(4, 10), Point::new(5, 11)], Piece::Head),
        (vec![Point::new(14, 16), Point::new(13, 15)], Piece::Tail),
        (
            vec![Point::new(3, 11), Point::new(5, 13), Point::new(5, 9)],
            Piece::Head,
        ),
    ] {
        board.take_turn(moves, player)?;
        println!();
        board.draw();
    }

    Ok(())
}
