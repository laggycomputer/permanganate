use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{AddAssign, IndexMut};

use ndarray::{Array2, AssignElem};
use unordered_pair::UnorderedPair;

mod tests;

type Coord = usize;
// x, y
type Location = (Coord, Coord);
type AffiliationID = usize;

#[derive(Clone, Copy, Debug)]
struct CellAffiliation {
    ident: AffiliationID,
    display: char,
}

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

impl Display for NumberlinkCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            NumberlinkCell::TERMINUS { affiliation } => affiliation.display.to_ascii_uppercase(),
            NumberlinkCell::PATH { affiliation } => affiliation.display,
            NumberlinkCell::EMPTY => '.'
        })
    }
}

#[derive(Clone)]
pub struct NumberlinkBoard {
    dims: Location,
    cells: Array2<NumberlinkCell>,
    last_used_aff_ident: Option<AffiliationID>,
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
            last_used_aff_ident: None,
        }
    }

    fn next_avail_aff_ident(&self) -> AffiliationID {
        match self.last_used_aff_ident {
            None => 0,
            Some(aff) => aff + 1
        }
    }

    pub fn add_termini(&mut self, locations: UnorderedPair<Location>) {
        self._add_termini(
            self.next_avail_aff_ident(),
            ('A' as usize + self.next_avail_aff_ident()) as u8 as char,
            locations)
    }

    pub fn add_termini_with_display(&mut self, display: char, locations: UnorderedPair<Location>) {
        self._add_termini(
            self.next_avail_aff_ident(),
            display,
            locations)
    }

    fn _add_termini(&mut self, aff_id: AffiliationID, display: char, locations: UnorderedPair<Location>) {
        for endpoint in [locations.0, locations.1] {
            self.cells.index_mut(endpoint).assign_elem(NumberlinkCell::TERMINUS {
                affiliation: CellAffiliation { ident: aff_id, display }
            })
        }

        self.last_used_aff_ident = Some(aff_id);
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