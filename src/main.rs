use stelsalto::{Board, Game, Piece};

fn main() -> Result<(), Box<std::error::Error>> {
    let mut game = Game::new(Board::default(), vec![Piece::Head, Piece::Tail]);
    game.play()?;
    Ok(())
}
