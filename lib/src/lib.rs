use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::{AddAssign, IndexMut};

use itertools::Itertools;
use ndarray::{Array2, AssignElem};
use strum::VariantArray;
use unordered_pair::UnorderedPair;
use varisat::{CnfFormula, Var};

use crate::logic::exactly_one;

mod tests;
mod logic;

type Coord = usize;
// x, y
type Location = (Coord, Coord);
type AffiliationID = usize;

#[derive(Clone, Copy, Debug)]
struct CellAffiliation {
    ident: AffiliationID,
    display: char,
}

#[derive(Copy, Clone, Debug, Eq, Hash, VariantArray, PartialEq)]
pub enum BoardTraverseDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    // switch it up like nintendo
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
            NumberlinkCell::PATH { affiliation } => affiliation.display,
            NumberlinkCell::EMPTY => '.'
        })
    }
}

pub struct NumberlinkBoard {
    dims: Location,
    cells: Array2<NumberlinkCell>,
    logic: Array2<CnfFormula>,
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
            logic: Array2::from_shape_simple_fn((dims.1, dims.0), CnfFormula::default),
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

    pub fn num_affiliations(&self) -> usize {
        // if ID n is used, then n+1 IDs exist
        match self.last_used_aff_ident {
            None => 0,
            Some(aff_id) => aff_id + 1
        }
    }

    fn affiliation_var(&self, location: Location, affiliation_id: AffiliationID) -> Var {
        Var::from_index((location.1 * self.dims.0 + location.0) * self.num_affiliations() + affiliation_id)
    }

    fn _add_termini(&mut self, aff_id: AffiliationID, display: char, locations: UnorderedPair<Location>) {
        for endpoint_loc in [locations.0, locations.1] {
            self.cells.index_mut((endpoint_loc.1, endpoint_loc.0)).assign_elem(NumberlinkCell::TERMINUS {
                affiliation: CellAffiliation { ident: aff_id, display }
            });
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

    pub fn neighbors_of(&self, loc: Location) -> (HashSet<Location>, HashSet<BoardTraverseDirection>) {
        let mut neighbor_locs: HashSet<Location> = HashSet::with_capacity(4);
        let mut possible_directions: HashSet<BoardTraverseDirection> = HashSet::with_capacity(4);
        for dir in BoardTraverseDirection::VARIANTS {
            if let Some(neighbor_loc) = self.step(loc, *dir) {
                neighbor_locs.insert(neighbor_loc);
                possible_directions.insert(*dir);
            }
        }

        return (neighbor_locs, possible_directions);
    }

    // check that every affiliation with termini has exactly 2 termini
    pub fn is_valid_problem(&self) -> bool {
        let mut found_termini: HashMap<AffiliationID, u8> = HashMap::new();
        for cell in self.cells.iter() {
            if let NumberlinkCell::TERMINUS { affiliation } = cell {
                if let Some(count) = found_termini.get_mut(&affiliation.ident) {
                    count.add_assign(1)
                } else {
                    found_termini.insert(affiliation.ident, 1);
                }
            }
        }

        return found_termini.into_values().all(|c| c == 2);
    }

    pub fn solve_bsat(&mut self) -> Option<NumberlinkBoard> {
        if !self.is_valid_problem() || self.num_affiliations() == 0 {
            return None;
        }

        for row in 0..self.dims.1 {
            for col in 0..self.dims.0 {
                match self.cells.get((row, col)).unwrap() {
                    NumberlinkCell::TERMINUS { affiliation: affiliation_here } => {
                        let mut clauses = Vec::with_capacity(self.num_affiliations());

                        for aff_id in 0..self.num_affiliations() {
                            let var_here = self.affiliation_var((col, row), aff_id);
                            // this cell has the correct affiliation and does not have any other affiliation
                            clauses.push(vec![var_here.lit(aff_id == affiliation_here.ident)])
                        }

                        // there exists exactly one neighbor with the same affiliation
                        clauses.extend(exactly_one(
                            self.neighbors_of((col, row)).0.into_iter()
                                .map(|loc| self.affiliation_var(loc, affiliation_here.ident))
                                .collect::<Vec<_>>()
                        ));

                        self.logic.index_mut((row, col)).assign_elem(CnfFormula::from(clauses))
                    }
                    NumberlinkCell::EMPTY => {
                        // this cell has exactly one affiliation
                        self.logic.index_mut((row, col)).assign_elem(CnfFormula::from(exactly_one(
                            (0..=self.last_used_aff_ident.unwrap())
                                .map(|aff_id| self.affiliation_var((col, row), aff_id))
                                .collect_vec())
                        ));

                        // todo: exactly two neighbors have this affiliation
                        // todo: these same neighbors have no other affiliation
                        // todo: the remaining 2 neighbors have a different affiliation
                    }
                    _ => {}
                }
            }
        }
        todo!();
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