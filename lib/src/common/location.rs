use std::num::NonZero;

use ndarray::Ix;

type Coord = usize;
pub(crate) type Dimension = NonZero<Coord>;

#[derive(Clone, Eq, Hash, Copy, PartialEq, Ord, PartialOrd, Debug)]
// x, y
pub(crate) struct Location(pub(crate) Coord, pub(crate) Coord);

impl Location {
    pub(crate) fn as_index(&self) -> (Coord, Coord) {
        (self.1, self.0)
    }
    pub(crate) fn offset_by(self, rhs: (isize, isize)) -> Self {
        Self(self.0.wrapping_add_signed(rhs.0), self.1.wrapping_add_signed(rhs.1))
    }
}

impl From<(Ix, Ix)> for Location {
    fn from(value: (Ix, Ix)) -> Self {
        Self(value.1, value.0)
    }
}
