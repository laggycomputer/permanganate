use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::{AddAssign, IndexMut};

use itertools::Itertools;
use ndarray::{Array2, AssignElem};
use strum::VariantArray;
use varisat::{CnfFormula, Solver, Var};

use crate::common::{AffiliationID, CellAffiliation, Coord, Location, NumberlinkCell};
use crate::logic::exactly_one;

#[derive(Copy, Clone, Debug, Eq, Hash, VariantArray, PartialEq)]
pub enum BoardTraverseDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    // switch it up like nintendo
}

impl BoardTraverseDirection {
    fn is_part_of(&self, path_shape: &PathShape) -> bool {
        match self {
            BoardTraverseDirection::UP => vec![PathShape::UPDOWN, PathShape::UPLEFT, PathShape::UPRIGHT].contains(&path_shape),
            BoardTraverseDirection::DOWN => vec![PathShape::UPDOWN, PathShape::DOWNLEFT, PathShape::DOWNRIGHT].contains(&path_shape),
            BoardTraverseDirection::LEFT => vec![PathShape::LEFTRIGHT, PathShape::UPLEFT, PathShape::DOWNLEFT].contains(&path_shape),
            BoardTraverseDirection::RIGHT => vec![PathShape::LEFTRIGHT, PathShape::UPRIGHT, PathShape::DOWNRIGHT].contains(&path_shape),
        }
    }
}

// a path cell has exactly 2 neighbors in one of these six ways; we order them to make declaring variables easier
#[derive(Copy, Clone, Debug, VariantArray, PartialEq)]
enum PathShape {
    UPDOWN,
    UPLEFT,
    UPRIGHT,
    DOWNLEFT,
    DOWNRIGHT,
    LEFTRIGHT,
}

impl PathShape {
    fn possible_with(&self, possible_directions: &HashSet<BoardTraverseDirection>) -> bool {
        match self {
            PathShape::UPDOWN => possible_directions.contains(&BoardTraverseDirection::UP)
                && possible_directions.contains(&BoardTraverseDirection::DOWN),
            PathShape::UPLEFT => possible_directions.contains(&BoardTraverseDirection::UP)
                && possible_directions.contains(&BoardTraverseDirection::LEFT),
            PathShape::UPRIGHT => possible_directions.contains(&BoardTraverseDirection::UP)
                && possible_directions.contains(&BoardTraverseDirection::RIGHT),
            PathShape::DOWNLEFT => possible_directions.contains(&BoardTraverseDirection::DOWN)
                && possible_directions.contains(&BoardTraverseDirection::LEFT),
            PathShape::DOWNRIGHT => possible_directions.contains(&BoardTraverseDirection::DOWN)
                && possible_directions.contains(&BoardTraverseDirection::RIGHT),
            PathShape::LEFTRIGHT => possible_directions.contains(&BoardTraverseDirection::LEFT)
                && possible_directions.contains(&BoardTraverseDirection::RIGHT),
        }
    }
}

pub struct SimpleNumberlinkBoard {
    dims: (Coord, Coord),
    cells: Array2<NumberlinkCell>,
    last_used_aff_ident: Option<AffiliationID>,
    affiliation_displays: HashMap<AffiliationID, char>,
}

impl Default for SimpleNumberlinkBoard {
    fn default() -> Self {
        Self::with_dims((5, 5)).unwrap()
    }
}

impl SimpleNumberlinkBoard {
    pub fn with_dims(dims: (Coord, Coord)) -> Result<Self, &'static str> {
        if dims.0 <= 0 || dims.1 <= 0 {
            return Err("invalid dims");
        }

        Ok(Self {
            dims,
            // row major
            cells: Array2::from_shape_simple_fn((dims.1, dims.0), NumberlinkCell::default),
            last_used_aff_ident: None,
            affiliation_displays: HashMap::new(),
        })
    }

    fn next_avail_aff_ident(&self) -> AffiliationID {
        match self.last_used_aff_ident {
            None => 0,
            Some(aff) => aff + 1
        }
    }

    pub fn add_termini(&mut self, locations: (Location, Location)) {
        self._add_termini(
            self.next_avail_aff_ident(),
            ('A' as usize + self.next_avail_aff_ident()) as u8 as char,
            locations)
    }

    pub fn add_termini_with_display(&mut self, display: char, locations: (Location, Location)) {
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

    pub(crate) fn affiliation_var(&self, location: Location, affiliation_id: AffiliationID) -> Var {
        Var::from_index((location.1 * self.dims.0 + location.0) * self.num_affiliations() + affiliation_id)
    }

    fn shape_var(&self, location: Location, path_shape: PathShape) -> Var {
        Var::from_index(
            // highest possible affiliation var
            self.dims.1 * self.dims.0 * self.num_affiliations()
                // now, build new index
                + (location.1 * self.dims.0 + location.0) * PathShape::VARIANTS.len()
                + PathShape::VARIANTS.iter()
                .find_position(|shape| **shape == path_shape)
                .unwrap().0
        )
    }

    fn _add_termini(&mut self, aff_id: AffiliationID, display: char, locations: (Location, Location)) {
        for endpoint_loc in [locations.0, locations.1] {
            self.cells.index_mut((endpoint_loc.1, endpoint_loc.0)).assign_elem(NumberlinkCell::TERMINUS {
                affiliation: CellAffiliation { ident: aff_id, display }
            });
        }

        self.affiliation_displays.insert(aff_id, display);

        self.last_used_aff_ident = Some(aff_id);
    }

    pub fn step(&self, loc: Location, direction: BoardTraverseDirection) -> Option<Location> {
        let new_loc = match direction {
            BoardTraverseDirection::UP => loc.offset_by((0, -1)),
            BoardTraverseDirection::DOWN => loc.offset_by((0, 1)),
            BoardTraverseDirection::LEFT => loc.offset_by((-1, 0)),
            BoardTraverseDirection::RIGHT => loc.offset_by((1, 0)),
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

    pub fn solve_bsat(&self) -> Option<SimpleNumberlinkBoard> {
        if !self.is_valid_problem() || self.num_affiliations() == 0 {
            return None;
        }

        let mut logic = Array2::from_shape_simple_fn((self.dims.1, self.dims.0), CnfFormula::default);
        let mut assumptions = Vec::new();

        for (index, cell) in self.cells.indexed_iter() {
            let location = Location::from(index);
            match cell {
                NumberlinkCell::TERMINUS { affiliation: affiliation_here } => {
                    let mut clauses = Vec::with_capacity(self.num_affiliations());

                    for aff_id in 0..self.num_affiliations() {
                        let var_here = self.affiliation_var(location, aff_id);
                        // this cell has the correct affiliation and does not have any other affiliation
                        assumptions.push(var_here.lit(aff_id == affiliation_here.ident));
                    }

                    // there exists exactly one neighbor with the same affiliation
                    clauses.extend(exactly_one(
                        self.neighbors_of(location).0.into_iter()
                            .map(|loc| self.affiliation_var(loc, affiliation_here.ident))
                            .collect_vec()
                    ));

                    logic.index_mut(index).assign_elem(CnfFormula::from(clauses))
                }
                NumberlinkCell::EMPTY => {
                    let mut clauses = Vec::new();

                    // this cell has exactly one affiliation
                    clauses.extend(exactly_one(
                        (0..=self.last_used_aff_ident.unwrap())
                            .map(|aff_id| self.affiliation_var(location, aff_id))
                            .collect_vec()
                    ));

                    // this cell has exactly one shape
                    clauses.extend(exactly_one(
                        PathShape::VARIANTS.iter()
                            .map(|shape| self.shape_var(location, *shape))
                            .collect_vec()
                    ));

                    let (locations, directions) = self.neighbors_of(location);
                    // for every possible path shape S on cell A...
                    'shape: for path_shape in PathShape::VARIANTS.iter() {
                        // let X be the statement "cell A has shape S"
                        let x = self.shape_var(location, *path_shape);
                        if !path_shape.possible_with(&directions) {
                            // this path shape would imply A connects with cells not on the grid; impossible!
                            assumptions.push(x.negative());
                            continue 'shape;
                        }

                        // for each affiliation this cell (cell A) may hold...
                        for affiliation in 0..=self.last_used_aff_ident.unwrap() {
                            // for each neighbor of cell A, call it cell B...
                            for (neighbor_location, direction) in locations.iter().zip(directions.clone()) {
                                // let Y be the statement "cell A has affiliation C", Z be the statement "cell B has affiliation C"
                                let y = self.affiliation_var(location, affiliation);
                                let z = self.affiliation_var(*neighbor_location, affiliation);
                                if direction.is_part_of(path_shape) {
                                    /*
                                    when cell B is on shape S, Y must equal Z
                                    we seek X => Y*Z + !Y*!Z
                                    law of excluded middle:
                                    = X => (!Y*Y + Y*Z + !Y*!Z + !Z*Z)
                                    factor:
                                    X => (Y + !Z) * (!Y + Z)
                                    by definition of imply:
                                    = (!X + Y + !Z) * (!X + !Y + Z)
                                    */

                                    clauses.extend(vec![
                                        vec![x.negative(), y.positive(), z.negative()],
                                        vec![x.negative(), y.negative(), z.positive()],
                                    ])
                                } else {
                                    // if cell B is not on shape S, then X => !Y + !Z
                                    // (A must have exactly one shape)
                                    clauses.push(vec![x.negative(), y.negative(), z.negative()])
                                }
                            }
                        }
                    }

                    logic.index_mut(index).assign_elem(CnfFormula::from(clauses));
                }
                _ => {}
            }
        }

        let mut solver = Solver::new();
        logic.iter().for_each(|formula| solver.add_formula(formula));
        solver.assume(assumptions.as_slice());
        solver.solve().unwrap();
        let solved = solver.model().unwrap();

        let mut new_board = Self::with_dims(self.dims).unwrap();

        for (index, cell) in self.cells.indexed_iter() {
            let location = Location::from(index);
            match cell {
                NumberlinkCell::TERMINUS { affiliation: _ } => {
                    new_board.cells.index_mut(index).assign_elem(*cell);
                }
                NumberlinkCell::EMPTY => {
                    let solved_affiliation = (0..=self.last_used_aff_ident.unwrap())
                        .find(|aff| {
                            let var = self.affiliation_var(location, *aff);
                            solved.get(var.index()).unwrap().is_positive()
                        }).unwrap();
                    new_board.cells.index_mut(index).assign_elem(NumberlinkCell::PATH {
                        affiliation: CellAffiliation {
                            ident: solved_affiliation,
                            display: *self.affiliation_displays.get(&solved_affiliation).unwrap(),
                        }
                    });

                    // todo: eliminate cycles if found
                    // println!("{:?}", PathShape::VARIANTS.iter()
                    //     .find(|shape| {
                    //         let var = self.shape_var(location, **shape);
                    //         solved.get(var.index()).unwrap().is_positive()
                    //     }))
                }
                _ => {}
            }
        }

        Some(new_board)
    }
}

impl Display for SimpleNumberlinkBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut ret = String::new();

        for row in self.cells.rows() {
            ret.push_str(&*row.mapv(|cell| cell.to_string()).to_vec().join(""));
            ret.push('\n');
        }
        write!(f, "{}", ret)
    }
}