use std::collections::HashSet;
use std::num::NonZero;
use std::ops::IndexMut;

use itertools::Itertools;
use ndarray::{Array2, AssignElem};
use petgraph::graphmap::UnGraphMap;
use unordered_pair::UnorderedPair;

use crate::cell::NumberlinkCell;
use crate::board::{Edge, Board, Node};
use crate::location::{Dimension, Location};
use crate::shape::{BoardShape, SquareStep, Step};

/// Reasons a builder may become invalid while building.
#[derive(Copy, Clone, Debug)]
pub enum BuilderInvalidReason {
    /// A feature like a bridge was inserted outside the bounds specified by `dims` on a builder.
    FeatureOutOfBounds,
}

/// Functionality all builders must implement, parametrised over the grid shape `Sh` of the resulting board.
pub trait Builder<Sh: BoardShape> {
    /// Construct a new [`Self`] with the specified dimensions, specified in `(x, y)` order.
    fn with_dims(dims: (Dimension, Dimension)) -> Self;
    /// Add termini or "flow endpoints". The order in which `locations` are specified does not matter.
    ///
    /// May cause the builder to enter a [`FeatureOutOfBounds`](BuilderInvalidReason::FeatureOutOfBounds) invalid state if either location is out of bounds.
    /// If the builder is already in an invalid state, this function does nothing.
    fn add_termini(&mut self, display: char, locations: (Location, Location)) -> &mut Self;
    /// Remove termini by their assigned display character.
    ///
    /// If the builder is in an invalid state, this function does nothing.
    fn remove_termini(&mut self, display: char) -> &mut Self;
    /// Check the validity of this builder, ensuring no [`BuilderInvalidReason`] condition has arisen.
    ///
    /// Returns `None` if the builder is valid, `Some(&Vec<BuilderInvalidReason>)` otherwise.
    fn is_valid(&self) -> Option<&Vec<BuilderInvalidReason>>;
    /// Convert the state of this builder into a [`Board`].
    /// If the builder is invalid for any reason, a reference to a [`Vec`] of [`BuilderInvalidReason`] will indicate why.
    fn build(&self) -> Result<Board<Sh>, &Vec<BuilderInvalidReason>>;
}

/// A builder for boards with square-shaped cells, i.e. the rectangular boards found in Numberlink puzzles and in Flow Free and the Bridges and Warps expansions.
pub struct SquareNumberlinkBoardBuilder {
    // width, height
    dims: (Dimension, Dimension),
    cells: Array2<NumberlinkCell<SquareStep>>,
    invalid_reasons: Vec<BuilderInvalidReason>,
    // TODO
    edge_blacklist: HashSet<UnorderedPair<Location>>,
    node_blacklist: HashSet<Location>,
    bridges: HashSet<Location>,
    edge_whitelist: HashSet<UnorderedPair<Location>>,
    affiliation_displays: Vec<char>,
}

impl Default for SquareNumberlinkBoardBuilder {
    fn default() -> Self {
        Self::with_dims((NonZero::new(5).unwrap(), NonZero::new(5).unwrap()))
    }
}

impl Builder<SquareStep> for SquareNumberlinkBoardBuilder {
    fn with_dims(dims: (Dimension, Dimension)) -> Self {
        Self {
            dims,
            cells: Array2::from_shape_simple_fn((dims.1.get(), dims.0.get()), NumberlinkCell::default),

            invalid_reasons: Default::default(),
            edge_blacklist: Default::default(),
            node_blacklist: Default::default(),
            bridges: Default::default(),
            edge_whitelist: Default::default(),
            affiliation_displays: Default::default(),
        }
    }

    fn add_termini(&mut self, display: char, locations: (Location, Location)) -> &mut Self {
        if !self.invalid_reasons.is_empty() {
            return self;
        }

        for location in [locations.0, locations.1] {
            if location.0 >= self.dims.0.get() || location.1 >= self.dims.1.get() {
                self.invalid_reasons.push(BuilderInvalidReason::FeatureOutOfBounds);
                return self;
            }
        }

        // non-null affiliation IDs start at 1
        let aff = self.affiliation_displays.len() + 1;
        self.affiliation_displays.push(display);
        for location in [locations.0, locations.1] {
            self.cells.index_mut(location.as_index()).assign_elem(NumberlinkCell::TERMINUS { affiliation: aff })
        }

        self
    }

    fn remove_termini(&mut self, display: char) -> &mut Self {
        if !self.invalid_reasons.is_empty() {
            return self;
        }

        match self.affiliation_displays.iter().find_position(|disp| **disp == display) {
            None => {}
            Some((index, _)) => self.cells.map_inplace(|cell| {
                match cell {
                    NumberlinkCell::TERMINUS { affiliation } => if *affiliation == (index + 1) {
                        cell.assign_elem(NumberlinkCell::EMPTY);
                    },
                    _ => {}
                }
            })
        }

        self
    }

    fn is_valid(&self) -> Option<&Vec<BuilderInvalidReason>> {
        if self.invalid_reasons.is_empty() {
            None
        } else {
            Some(&self.invalid_reasons)
        }
    }

    fn build(&self) -> Result<Board<SquareStep>, &Vec<BuilderInvalidReason>> {
        if !self.invalid_reasons.is_empty() {
            return Err(&self.invalid_reasons);
        }

        let mut graph = UnGraphMap::with_capacity(
            // naively allocate for a complete grid of this size, which usually isn't too far off
            self.cells.len(),
            // "horizontal" edges
            (self.dims.0.get() - 1) * self.dims.1.get()
                // "vertical" edges
                + (self.dims.1.get() - 1) * self.dims.0.get(),
        );

        let mut nodes = Array2::from_shape_fn(self.cells.raw_dim(), |ind| Node {
            location: Location::from(ind),
            cell: *self.cells.get(ind).unwrap(),
        });

        for x in 0..self.dims.0.get() {
            for y in 0..self.dims.1.get() {
                let location = Location(x, y);
                // add edges down and to the right, if possible
                let location_below = SquareStep::DOWN.attempt_from(location);
                let location_right = SquareStep::RIGHT.attempt_from(location);

                let node = nodes.get(location.as_index()).unwrap();
                let node_below = nodes.get(location_below.as_index());
                let node_right = nodes.get(location_right.as_index());

                node_below.and_then(|other_node| graph.add_edge(*node, *other_node, Edge { affiliation: 0, direction: SquareStep::DOWN }));
                node_right.and_then(|other_node| graph.add_edge(*node, *other_node, Edge { affiliation: 0, direction: SquareStep::RIGHT }));
            }
        }

        // TODO: handle bridges, warps, any shape besides simple complete rectangle graph

        let mut affiliation_displays = Vec::with_capacity(self.affiliation_displays.len() + 1);
        // affiliation 0 is unaffiliated and will display as empty
        affiliation_displays.push('.');
        affiliation_displays.extend(self.affiliation_displays.clone());

        Ok(Board {
            graph,
            dims: self.dims,
            affiliation_displays,
        })
    }
}