use std::fmt::{Display, Formatter};
use std::num::NonZero;
use ndarray::Ix;

use crate::common::affiliation::Affiliation;

pub type Coord = usize;
pub type Dimension = NonZero<Coord>;

#[derive(Clone, Eq, Hash, Copy, PartialEq, Ord, PartialOrd, Debug)]
// x, y
pub struct Location(pub Coord, pub Coord);

impl Location {
    pub(crate) fn as_index(&self) -> (Coord, Coord) {
        (self.1, self.0)
    }
    pub fn offset_by(self, rhs: (isize, isize)) -> Self {
        Self(self.0.wrapping_add_signed(rhs.0), self.1.wrapping_add_signed(rhs.1))
    }
}

impl From<(Ix, Ix)> for Location {
    fn from(value: (Ix, Ix)) -> Self {
        Self(value.1, value.0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum NumberlinkCell {
    TERMINUS { affiliation: Affiliation },
    PATH { affiliation: Affiliation },
    #[default]
    EMPTY,
}

impl Display for NumberlinkCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            NumberlinkCell::TERMINUS { affiliation } => affiliation.display.to_ascii_uppercase(),
            NumberlinkCell::PATH { affiliation } => affiliation.display.to_ascii_lowercase(),
            NumberlinkCell::EMPTY => '.'
        })
    }
}