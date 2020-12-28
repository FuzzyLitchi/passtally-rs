use rand::{prelude::SliceRandom, thread_rng};
use thiserror::Error;

use crate::board::Board;
use crate::piece::{Piece, PositionedPiece};

/// A complete passtally game.
pub struct Game {
    board: Board,
    player_markers: [Option<u8>; 24],
    player_count: u8,
    /// Amount of rounds played
    round: u32,
    /// The three decks. Each deck starts at 14 cards for a total of 42.
    decks: [Vec<Piece>; 3],
}

impl Game {
    pub fn new(player_count: u8) -> Game {
        use Piece::*;
        let mut rng = thread_rng();
        let mut deck1 = [Red, Green, Yellow, Blue, Cyan, Pink].repeat(7);
        deck1.shuffle(&mut rng);
        let mut deck2 = deck1.split_off(14);
        let deck3 = deck2.split_off(14);

        Game {
            board: Board::default(),
            player_markers: [None; 24],
            player_count,
            round: 0,
            decks: [deck1, deck2, deck3],
        }
    }

    pub fn next_player(&self) -> u8 {
        (self.round % (self.player_count as u32)) as u8
    }

    pub fn play_turn(&mut self, turn: Turn) -> Result<(), PasstallyError> {
        let backup = (self.board.clone(), self.player_markers);

        let Turn(action1, action2) = turn;
        let res = self
            .do_action(action1)
            .and_then(|_| self.do_action(action2));

        match res {
            Ok(_) => {
                self.round += 1;
                Ok(())
            }
            Err(err) => {
                self.board = backup.0;
                self.player_markers = backup.1;
                Err(err)
            }
        }
    }

    fn do_action(&mut self, action: Action) -> Result<(), PasstallyError> {
        match action {
            Action::PlacePiece(piece) => self.board.place_piece(piece),
            Action::MovePlayerMarker(from, to) => self.move_player_marker(from, to),
        }
    }

    fn move_player_marker(&mut self, from: u8, to: u8) -> Result<(), PasstallyError> {
        assert!(matches!(from, 0..=23));
        assert!(matches!(to, 0..=23));

        // Check that "from" isn't empty
        if self.player_markers[from as usize].is_none() {
            return Err(PasstallyError::NoPlayerMarker);
        }

        // Check that "to" isn't occupied
        if self.player_markers[to as usize].is_some() {
            return Err(PasstallyError::HasPlayerMarker);
        }

        // Check that there is at most one empty space between the two positions
        // (we actually check both directions because maybe there's 22 filled
        //  spaces in the long direction and 2 empty)
        // Imagine the player markers are placed like this. If we only checked the
        // short end it would look like it is too far.
        //
        //   X X X X X X
        // X             X
        // X             X
        // X             X
        // X             X
        // X             X
        // X             X
        //   X F _ _ _ X
        //           ^

        let valid_move = {
            let min = from.min(to);
            let max = from.max(to);

            // Iter between min and max the short way
            let empty_spaces = (min + 1..max)
                .into_iter()
                .filter(|&i| self.player_markers[i as usize].is_none())
                .count();
            if empty_spaces <= 1 {
                true
            } else {
                // Iter between them the long way
                let empty_spaces = (max + 1..min + 24)
                    .into_iter()
                    .map(|v| v % 24)
                    .filter(|&i| self.player_markers[i as usize].is_none())
                    .count();
                empty_spaces <= 1
            }
        };

        if !valid_move {
            return Err(PasstallyError::TooFar);
        }

        // Move player marker
        self.player_markers[to as usize] = self.player_markers[from as usize].take();
        Ok(())
    }
}

pub enum Action {
    PlacePiece(PositionedPiece),
    MovePlayerMarker(u8, u8), // 0..=23
}

pub struct Turn(pub Action, pub Action);

#[derive(Error, Debug)]
pub enum PasstallyError {
    #[error("The piece is outside of the board.")]
    InvalidPosition,
    #[error("The height for the two positions aren't the same.")]
    BadHeight,
    #[error("You cannot place a piece directly ontop of another piece.")]
    BadPiece,
    #[error("There is no player marker at the from position.")]
    NoPlayerMarker,
    #[error("There is already a player marker at the to position.")]
    HasPlayerMarker,
    #[error("There is more than one empty player marker field between the from and to position.")]
    TooFar,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn construct_game() {
        let _game = Game::new(2);
    }
}
