use std::hash::Hash;

use itertools::Itertools;
use ndarray::Array2;
use strum::VariantArray;

use crate::common::location::Location;

pub trait Step {
    fn attempt_from(&self, location: Location) -> Location;
    // directions which result in an index increase in a 2d array representation
    fn forward_edge_directions() -> &'static [Self]
    where
        Self: Sized;
    fn invert(&self) -> Self;
    fn print(board: Array2<char>) -> String;
}

#[derive(Copy, Clone, VariantArray, Eq, PartialEq, Hash, Debug)]
pub enum SquareStep {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    // switch it up like nintendo
}

impl Step for SquareStep {
    fn attempt_from(&self, location: Location) -> Location {
        match self {
            Self::UP => location.offset_by((0, -1)),
            Self::DOWN => location.offset_by((0, 1)),
            Self::LEFT => location.offset_by((-1, 0)),
            Self::RIGHT => location.offset_by((1, 0)),
        }
    }

    fn forward_edge_directions() -> &'static [Self]
    where
        Self: Sized,
    {
        &[Self::DOWN, Self::RIGHT]
    }

    fn invert(&self) -> Self {
        match self {
            Self::UP => Self::DOWN,
            Self::DOWN => Self::UP,
            Self::LEFT => Self::RIGHT,
            Self::RIGHT => Self::LEFT,
        }
    }

    fn print(board: Array2<char>) -> String {
        let mut out = String::with_capacity(board.nrows() * (board.ncols() + 1));

        for row in board.rows() {
            for col in row {
                out.push(*col);
            }
            out.push('\n');
        }

        out
    }
}

// NB: we organize hexagonal grids as follows:
// 0   1   2   3
//   0   1   2   3
// 0   1   2   3
//   0   1   2   3
#[derive(Copy, Clone, VariantArray)]
pub enum HexStep {
    UP,
    UPRIGHT,
    RIGHTDOWN,
    DOWN,
    DOWNLEFT,
    LEFTUP,
}

impl Step for HexStep {
    fn attempt_from(&self, location: Location) -> Location {
        match self {
            Self::UP => location.offset_by((0, -2)),
            // these are more complicated; consider the parity of the rows
            Self::UPRIGHT => location.offset_by((if location.1 & 2 == 0 { 1 } else { 0 }, -1)),
            Self::RIGHTDOWN => location.offset_by((if location.1 & 2 == 0 { 1 } else { 0 }, -1)),
            Self::DOWN => location.offset_by((0, 2)),
            Self::DOWNLEFT => location.offset_by((if location.1 & 2 == 0 { 0 } else { -1 }, 1)),
            Self::LEFTUP => location.offset_by((if location.1 & 2 == 0 { 0 } else { -1 }, -1)),
        }
    }

    fn forward_edge_directions() -> &'static [Self]
    where
        Self: Sized,
    {
        &[Self::DOWN, Self::RIGHTDOWN, Self::DOWNLEFT]
    }

    fn invert(&self) -> Self {
        match self {
            Self::UP => Self::DOWN,
            Self::UPRIGHT => Self::DOWNLEFT,
            Self::RIGHTDOWN => Self::LEFTUP,
            Self::DOWN => Self::UP,
            Self::DOWNLEFT => Self::UPRIGHT,
            Self::LEFTUP => Self::RIGHTDOWN,
        }
    }

    fn print(board: Array2<char>) -> String {
        todo!()
    }
}

pub trait BoardShape: Copy + Step + VariantArray + PartialEq + Eq + Hash {
    fn neighbors_of(&self, location: Location) -> Vec<(Self, Location)>
    where
        Self: Sized;
    fn direction_to(a: Location, b: Location) -> Option<Self>
    where
        Self: Sized;
    fn ensure_forward(&self) -> Self;
}

impl<T> BoardShape for T
where
    T: Copy + Step + VariantArray + PartialEq + Eq + Hash,
{
    fn neighbors_of(&self, location: Location) -> Vec<(Self, Location)> {
        Self::VARIANTS.iter()
            .map(|dir| (*dir, dir.attempt_from(location)))
            .collect_vec()
    }

    fn direction_to(a: Location, b: Location) -> Option<Self <>> {
        Self::VARIANTS.iter().find(|dir| dir.attempt_from(a) == b).and_then(|dir| Some(*dir))
    }

    fn ensure_forward(&self) -> Self {
        match Self::forward_edge_directions().contains(self) {
            true => *self,
            false => self.invert(),
        }
    }
}