use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

use crate::board::BoardPosition;
use Side::*;

#[derive(Copy, Clone, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Side {
    Top = 0,
    Right = 1,
    Bottom = 2,
    Left = 3,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Top => Bottom,
            Bottom => Top,
            Left => Right,
            Right => Left,
        }
    }

    /// Rotation is clockwise and 0..=3
    pub fn rotate(self, n: u8) -> Self {
        Self::try_from((self as u8 + n) % 4).unwrap()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(non_camel_case_types, clippy::enum_variant_names)]
pub enum PartialPiece {
    TopBottom_LeftRight, // Pipes top to bottom and left to right
    TopLeft_BottomRight, // Pipes top to left and bottom to right
    TopRight_BottomLeft, // Pipes top to right and bottom to left
}

impl PartialPiece {
    // returns which side we are leaving from when we pass through this partial piece
    pub fn pass(&self, side: Side) -> Side {
        use PartialPiece::*;

        match self {
            TopBottom_LeftRight => side.opposite(),
            TopLeft_BottomRight => match side {
                Top => Left,
                Left => Top,
                Bottom => Right,
                Right => Bottom,
            },
            TopRight_BottomLeft => match side {
                Top => Right,
                Right => Top,
                Bottom => Left,
                Left => Bottom,
            },
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RotatedPartialPiece {
    partial_piece: PartialPiece,
    rotation: u8,
}

impl RotatedPartialPiece {
    pub fn new(partial_piece: PartialPiece, rotation: u8) -> Self {
        RotatedPartialPiece {
            partial_piece,
            rotation,
        }
    }

    pub fn pass(&self, side: Side) -> Side {
        // Rotate into local side
        let local_side = side.rotate(4 - self.rotation);
        // Pass through piece
        let exit_side = self.partial_piece.pass(local_side);
        // Rotate back to global
        exit_side.rotate(self.rotation)
    }
}

pub enum Piece {
    // A is TopBottom_LeftRight,
    // B is TopLeft_BottomRight,
    // C is TopRight_BottomLeft
    Red,    // A A
    Green,  // B B
    Yellow, // C C
    Blue,   // A B
    Cyan,   // A C
    Pink,   // C B
}

pub struct PositionedPiece {
    pub piece: Piece,
    pub rotation: u8,
    pub position: BoardPosition,
}

impl PositionedPiece {
    pub fn positions(&self) -> (BoardPosition, BoardPosition) {
        let second_position = match self.rotation {
            0 => self.position + BoardPosition::new(1, 0), // Unrotated pieces are horizontal, and the second part is to the right
            1 => self.position + BoardPosition::new(0, 1),
            2 => self.position + BoardPosition::new(-1, 0),
            3 => self.position + BoardPosition::new(0, -1),
            _ => unreachable!("Rotation should only be 0-3"),
        };
        (self.position, second_position)
    }

    pub fn rotated_partial_pieces(&self) -> (RotatedPartialPiece, RotatedPartialPiece) {
        let (first_piece, second_piece) = self.partial_pieces();
        (
            RotatedPartialPiece::new(first_piece, self.rotation),
            RotatedPartialPiece::new(second_piece, self.rotation),
        )
    }

    fn partial_pieces(&self) -> (PartialPiece, PartialPiece) {
        use Piece::*;
        match self.piece {
            Red => (
                PartialPiece::TopBottom_LeftRight,
                PartialPiece::TopBottom_LeftRight,
            ),
            Green => (
                PartialPiece::TopLeft_BottomRight,
                PartialPiece::TopLeft_BottomRight,
            ),
            Yellow => (
                PartialPiece::TopRight_BottomLeft,
                PartialPiece::TopRight_BottomLeft,
            ),
            Blue => (
                PartialPiece::TopBottom_LeftRight,
                PartialPiece::TopLeft_BottomRight,
            ),
            Cyan => (
                PartialPiece::TopBottom_LeftRight,
                PartialPiece::TopRight_BottomLeft,
            ),
            Pink => (
                PartialPiece::TopRight_BottomLeft,
                PartialPiece::TopLeft_BottomRight,
            ),
        }
    }
}
