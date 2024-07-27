use itertools::Itertools;
use strum::VariantArray;

use crate::common::location::Location;

#[derive(Copy, Clone, VariantArray, Eq, PartialEq, Hash, Debug)]
pub(crate) enum SquareStepDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    // switch it up like nintendo
}

#[derive(Copy, Clone, VariantArray)]
enum HexStepDirection {
    UP,
    UPRIGHT,
    RIGHTDOWN,
    DOWN,
    DOWNLEFT,
    LEFTUP,
}

#[derive(Copy, Clone)]
pub(crate) enum StepDirection {
    SQUARE { direction: SquareStepDirection },
    HEXAGON { direction: HexStepDirection },
}

impl StepDirection {
    pub(crate) fn attempt_from(self, location: Location) -> Location {
        match self {
            Self::SQUARE { direction } => match direction {
                SquareStepDirection::UP => location.offset_by((0, -1)),
                SquareStepDirection::DOWN => location.offset_by((0, 1)),
                SquareStepDirection::LEFT => location.offset_by((-1, 0)),
                SquareStepDirection::RIGHT => location.offset_by((1, 0)),
            }
            Self::HEXAGON { direction } => match direction {
                HexStepDirection::UP => location.offset_by((0, -2)),
                // these are more complicated; consider the parity of the rows
                HexStepDirection::UPRIGHT => location.offset_by((if location.1 & 2 == 0 { 1 } else { 0 }, -1)),
                HexStepDirection::RIGHTDOWN => location.offset_by((if location.1 & 2 == 0 { 1 } else { 0 }, -1)),
                HexStepDirection::DOWN => location.offset_by((0, 2)),
                HexStepDirection::DOWNLEFT => location.offset_by((if location.1 & 2 == 0 { 0 } else { -1 }, 1)),
                HexStepDirection::LEFTUP => location.offset_by((if location.1 & 2 == 0 { 0 } else { -1 }, -1)),
            }
        }
    }
}

pub enum BoardShape {
    SQUARE,
    // NB: we organize hexagonal grids as follows:
    //   0 1 2 3 4...
    //  0 1 2 3 4...
    //   0 1 2 3 4...
    //  0 1 2 3 4...
    HEXAGON,
}

pub trait CellNeighbors {
    fn neighbors_of(&self, location: Location) -> Vec<(StepDirection, Location)>;
}

impl CellNeighbors for BoardShape {
    fn neighbors_of(&self, location: Location) -> Vec<(StepDirection, Location)> {
        match self {
            BoardShape::SQUARE => SquareStepDirection::VARIANTS.iter()
                .map(|dir| StepDirection::SQUARE { direction: *dir })
                .map(|step_dir| (step_dir, step_dir.attempt_from(location)))
                .collect_vec(),
            BoardShape::HEXAGON => HexStepDirection::VARIANTS.iter()
                .map(|dir| StepDirection::HEXAGON { direction: *dir })
                .map(|step_dir| (step_dir, step_dir.attempt_from(location)))
                .collect_vec(),
        }
    }
}