use std::fmt::{Display, Formatter};
use std::ops::IndexMut;

use ndarray::{Array2, AssignElem};
use unordered_pair::UnorderedPair;

mod tests;

type Coord = usize;
// x, y
type Location = (Coord, Coord);
type CellAffiliation = usize;

pub enum BoardTraverseDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    // switch it up like nintendo
}

#[derive(Clone, Copy, Debug, Default)]
pub enum NumberlinkCell {
    TERMINUS { affiliation: CellAffiliation },
    PATH { affiliation: CellAffiliation },
    #[default]
    EMPTY,
}

const PATH_CHARS: [char; 26] = [
    'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j',
    'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't',
    'u', 'v', 'w', 'x', 'y',
    'z',
];

impl Display for NumberlinkCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            NumberlinkCell::TERMINUS { affiliation } => PATH_CHARS[*affiliation].to_ascii_uppercase(),
            NumberlinkCell::PATH { affiliation } => PATH_CHARS[*affiliation],
            NumberlinkCell::EMPTY => '.'
        })
    }
}

pub struct NumberlinkBoard {
    dims: Location,
    cells: Array2<NumberlinkCell>,
    last_used_affiliation: Option<CellAffiliation>,
}

impl Default for NumberlinkBoard {
    fn default() -> Self {
        Self::with_dims((5, 5))
    }
}

impl NumberlinkBoard {
    pub fn with_dims(dims: Location) -> NumberlinkBoard {
        NumberlinkBoard {
            dims,
            // row major
            cells: Array2::from_shape_simple_fn((dims.1, dims.0), NumberlinkCell::default),
            last_used_affiliation: None,
        }
    }

    pub fn add_endpoints(&mut self, locations: UnorderedPair<Location>) {
        let first_avail_affiliation = match self.last_used_affiliation {
            None => 0,
            Some(aff) => aff + 1
        };
        for endpoint in [locations.0, locations.1] {
            self.cells.index_mut(endpoint).assign_elem(NumberlinkCell::TERMINUS { affiliation: first_avail_affiliation })
        }
        self.last_used_affiliation = Some(first_avail_affiliation);
    }

    pub fn step(&self, loc: Location, direction: BoardTraverseDirection) -> Option<Location> {
        let new_loc = match direction {
            BoardTraverseDirection::UP => (loc.0.overflowing_add_signed(0).0, loc.1.overflowing_add_signed(-1).0),
            BoardTraverseDirection::DOWN => (loc.0.overflowing_add_signed(0).0, loc.1.overflowing_add_signed(1).0),
            BoardTraverseDirection::LEFT => (loc.0.overflowing_add_signed(-1).0, loc.1.overflowing_add_signed(0).0),
            BoardTraverseDirection::RIGHT => (loc.0.overflowing_add_signed(1).0, loc.1.overflowing_add_signed(0).0),
        };

        match (0..self.dims.0).contains(&new_loc.0) && (0..self.dims.1).contains(&new_loc.1) {
            true => Some(new_loc),
            false => None
        }
    }
}

impl Display for NumberlinkBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut ret = String::new();

        for row in self.cells.rows() {
            ret.push_str(&*row.mapv(|cell| cell.to_string()).to_vec().join(""));
            ret.push('\n');
        }
        write!(f, "{}", ret)
    }
}