use std::fmt::{Display, Formatter};

use ndarray::Ix;

pub type Coord = usize;
pub type AffiliationID = usize;

#[derive(Clone, Eq, Hash, Copy, PartialEq)]
// x, y
pub struct Location(pub Coord, pub Coord);
impl Location {
    fn as_index(&self) -> (Coord, Coord) {
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

#[derive(Clone, Copy, Debug)]
pub struct CellAffiliation {
    pub(crate) ident: AffiliationID,
    pub(crate) display: char,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum NumberlinkCell {
    TERMINUS { affiliation: CellAffiliation },
    PATH { affiliation: CellAffiliation },
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