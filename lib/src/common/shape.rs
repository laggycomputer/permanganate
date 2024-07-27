use itertools::Itertools;
use strum::VariantArray;

use crate::common::location::Location;

pub trait Step {
    fn attempt_from(&self, location: Location) -> Location;
}

#[derive(Copy, Clone, VariantArray, Eq, PartialEq, Hash, Debug)]
pub(crate) enum SquareStepDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    // switch it up like nintendo
}

impl Step for SquareStepDirection {
    fn attempt_from(&self, location: Location) -> Location {
        match self {
            Self::UP => location.offset_by((0, -1)),
            Self::DOWN => location.offset_by((0, 1)),
            Self::LEFT => location.offset_by((-1, 0)),
            Self::RIGHT => location.offset_by((1, 0)),
        }
    }
}

// NB: we organize hexagonal grids as follows:
// 0   1   2   3
//   0   1   2   3
// 0   1   2   3
//   0   1   2   3
#[derive(Copy, Clone, VariantArray)]
pub(crate) enum HexStepDirection {
    UP,
    UPRIGHT,
    RIGHTDOWN,
    DOWN,
    DOWNLEFT,
    LEFTUP,
}

impl Step for HexStepDirection {
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
}

pub trait BoardShape {
    fn neighbors_of(&self, location: Location) -> Vec<(Self, Location)> where Self: Sized;
    // directions which result in an index increase in a 2d array representation
    fn forward_edge_directions(&self) -> &[Self] where Self: Sized;
    fn ensure_forward_direction(direction: Self);
    fn direction_to(&self, a: Location, b: Location) -> Option<Self> where Self: Sized;
}

impl<T> BoardShape for T
where
    T: Copy + Clone + Step + VariantArray,
{
    fn neighbors_of(&self, location: Location) -> Vec<(Self, Location)> {
        Self::VARIANTS.iter()
            .map(|dir| (*dir, dir.attempt_from(location)))
            .collect_vec()
    }

    fn forward_edge_directions(&self) -> &[Self] {
        todo!();
        // match self {
        //     BoardShape::SQUARE => &[
        //         StepDirection::SQUARE { direction: SquareStepDirection::DOWN },
        //         StepDirection::SQUARE { direction: SquareStepDirection::RIGHT },
        //     ],
        //     BoardShape::HEXAGON => &[
        //         StepDirection::HEXAGON { direction: HexStepDirection::DOWN },
        //         StepDirection::HEXAGON { direction: HexStepDirection::RIGHTDOWN },
        //         StepDirection::HEXAGON { direction: HexStepDirection::DOWNLEFT },
        //     ]
        // }
    }

    fn ensure_forward_direction(direction: Self<>) {
        todo!();
    }

    fn direction_to(&self, a: Location, b: Location) -> Option<Self<>> {
        Self::VARIANTS.iter().find(|dir| dir.attempt_from(a) == b).and_then(|dir| Some(*dir))
    }
}