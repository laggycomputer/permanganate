use std::num::NonZero;
use ndarray::{Array2, AssignElem};
use std::collections::HashSet;
use unordered_pair::UnorderedPair;
use petgraph::graphmap::UnGraphMap;
use std::ops::IndexMut;
use crate::cell::NumberlinkCell;
use crate::graph::{Edge, GeneralNumberlinkBoard, Node};
use crate::location::{Dimension, Location};
use crate::shape::{SquareStep, Step};

#[derive(Copy, Clone, Debug)]
pub(crate) enum BuilderInvalidReason {
    FeatureOutOfBounds,
}

pub(crate) struct SquareNumberlinkBoardBuilder {
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

impl SquareNumberlinkBoardBuilder {
    pub(crate) fn with_dims(dims: (Dimension, Dimension)) -> Self {
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

    pub(crate) fn add_termini(&mut self, display: char, locations: (Location, Location)) -> &mut SquareNumberlinkBoardBuilder {
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

    pub(crate) fn build(&self) -> Result<GeneralNumberlinkBoard<SquareStep>, Vec<BuilderInvalidReason>> {
        if !self.invalid_reasons.is_empty() {
            return Err(self.invalid_reasons.clone());
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

        Ok(GeneralNumberlinkBoard {
            graph,
            dims: self.dims,
            affiliation_displays,
        })
    }
}