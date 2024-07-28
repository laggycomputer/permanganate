use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::num::NonZero;
use std::ops::{AddAssign, IndexMut};

use itertools::Itertools;
use ndarray::{Array2, AssignElem};
use strum::VariantArray;
use varisat::{CnfFormula, Solver, Var};

use crate::common::affiliation::{Affiliation, AffiliationID};
use crate::common::location::{Dimension, Location, NumberlinkCell};
use crate::common::logic::exactly_one;
use crate::common::shape::{SquareStep, Step};

impl SquareStep {
    fn is_part_of(&self, path_shape: &SquarePathShape) -> bool {
        match self {
            Self::UP => vec![SquarePathShape::UPDOWN, SquarePathShape::UPLEFT, SquarePathShape::UPRIGHT].contains(&path_shape),
            Self::DOWN => vec![SquarePathShape::UPDOWN, SquarePathShape::DOWNLEFT, SquarePathShape::DOWNRIGHT].contains(&path_shape),
            Self::LEFT => vec![SquarePathShape::LEFTRIGHT, SquarePathShape::UPLEFT, SquarePathShape::DOWNLEFT].contains(&path_shape),
            Self::RIGHT => vec![SquarePathShape::LEFTRIGHT, SquarePathShape::UPRIGHT, SquarePathShape::DOWNRIGHT].contains(&path_shape),
        }
    }
}

// a path cell has exactly 2 neighbors in one of these six ways; we order them to make declaring variables easier
#[derive(Copy, Clone, Debug, VariantArray, PartialEq)]
enum SquarePathShape {
    UPDOWN,
    UPLEFT,
    UPRIGHT,
    DOWNLEFT,
    DOWNRIGHT,
    LEFTRIGHT,
}

impl SquarePathShape {
    fn possible_with(&self, possible_directions: &HashSet<SquareStep>) -> bool {
        match self {
            SquarePathShape::UPDOWN => possible_directions.contains(&SquareStep::UP)
                && possible_directions.contains(&SquareStep::DOWN),
            SquarePathShape::UPLEFT => possible_directions.contains(&SquareStep::UP)
                && possible_directions.contains(&SquareStep::LEFT),
            SquarePathShape::UPRIGHT => possible_directions.contains(&SquareStep::UP)
                && possible_directions.contains(&SquareStep::RIGHT),
            SquarePathShape::DOWNLEFT => possible_directions.contains(&SquareStep::DOWN)
                && possible_directions.contains(&SquareStep::LEFT),
            SquarePathShape::DOWNRIGHT => possible_directions.contains(&SquareStep::DOWN)
                && possible_directions.contains(&SquareStep::RIGHT),
            SquarePathShape::LEFTRIGHT => possible_directions.contains(&SquareStep::LEFT)
                && possible_directions.contains(&SquareStep::RIGHT),
        }
    }
}

pub struct SimpleNumberlinkBoard {
    dims: (Dimension, Dimension),
    cells: Array2<NumberlinkCell>,
    last_used_aff_ident: Option<AffiliationID>,
    affiliation_displays: HashMap<AffiliationID, char>,
}

impl Default for SimpleNumberlinkBoard {
    fn default() -> Self {
        Self::with_dims((NonZero::new(5).unwrap(), NonZero::new(5).unwrap())).unwrap()
    }
}

impl SimpleNumberlinkBoard {
    pub fn with_dims(dims: (Dimension, Dimension)) -> Result<Self, &'static str> {
        Ok(Self {
            dims,
            // row major
            cells: Array2::from_shape_simple_fn((dims.1.get(), dims.0.get()), NumberlinkCell::default),
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
        Var::from_index((location.1 * self.dims.0.get() + location.0) * self.num_affiliations() + affiliation_id)
    }

    fn shape_var(&self, location: Location, path_shape: SquarePathShape) -> Var {
        Var::from_index(
            // highest possible affiliation var
            self.dims.1.get() * self.dims.0.get() * self.num_affiliations()
                // now, build new index
                + (location.1 * self.dims.0.get() + location.0) * SquarePathShape::VARIANTS.len()
                + SquarePathShape::VARIANTS.iter()
                .find_position(|shape| **shape == path_shape)
                .unwrap().0
        )
    }

    fn _add_termini(&mut self, aff_id: AffiliationID, display: char, locations: (Location, Location)) {
        for endpoint_loc in [locations.0, locations.1] {
            self.cells.index_mut((endpoint_loc.1, endpoint_loc.0)).assign_elem(NumberlinkCell::TERMINUS {
                affiliation: Affiliation { ident: aff_id, display }
            });
        }

        self.affiliation_displays.insert(aff_id, display);

        self.last_used_aff_ident = Some(aff_id);
    }

    pub fn step(&self, loc: Location, direction: SquareStep) -> Option<Location> {
        let new_loc = direction.attempt_from(loc);

        match (0..self.dims.0.get()).contains(&new_loc.0) && (0..self.dims.1.get()).contains(&new_loc.1) {
            true => Some(new_loc),
            false => None
        }
    }

    pub fn neighbors_of(&self, loc: Location) -> (HashSet<Location>, HashSet<SquareStep>) {
        let mut neighbor_locs: HashSet<Location> = HashSet::with_capacity(4);
        let mut possible_directions: HashSet<SquareStep> = HashSet::with_capacity(4);
        for dir in SquareStep::VARIANTS {
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

        let mut logic = Array2::from_shape_simple_fn((self.dims.1.get(), self.dims.0.get()), CnfFormula::default);
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
                            .map(|loc| self.affiliation_var(loc, affiliation_here.ident).positive())
                            .collect_vec()
                    ));

                    logic.index_mut(index).assign_elem(CnfFormula::from(clauses))
                }
                NumberlinkCell::EMPTY => {
                    let mut clauses = Vec::new();

                    // this cell has exactly one affiliation
                    clauses.extend(exactly_one(
                        (0..=self.last_used_aff_ident.unwrap())
                            .map(|aff_id| self.affiliation_var(location, aff_id).positive())
                            .collect_vec()
                    ));

                    // this cell has exactly one shape
                    clauses.extend(exactly_one(
                        SquarePathShape::VARIANTS.iter()
                            .map(|shape| self.shape_var(location, *shape).positive())
                            .collect_vec()
                    ));

                    let (locations, directions) = self.neighbors_of(location);
                    // for every possible path shape S on cell A...
                    'shape: for path_shape in SquarePathShape::VARIANTS.iter() {
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
                        affiliation: Affiliation {
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