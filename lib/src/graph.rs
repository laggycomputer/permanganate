use std::collections::HashSet;
use std::ops::IndexMut;

use itertools::Itertools;
use ndarray::{Array2, AssignElem};
use petgraph::graphmap::UnGraphMap;
use unordered_pair::UnorderedPair;
use varisat::Var;

use crate::common::affiliation::{AffiliationID, CellAffiliation};
use crate::common::location::{Coord, Location, NumberlinkCell};
use crate::common::shape::{BoardShape, SquareStepDirection, StepDirection};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
struct Node {
    location: Location,
    cell: NumberlinkCell,
}

#[derive(Clone, Copy)]
struct Edge {
    affiliation: Option<CellAffiliation>,
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum AffiliationHolderType {
    EDGE { nodes: UnorderedPair<Node> },
    NODE,
}

#[derive(Default)]
pub struct GeneralNumberlinkBoard {
    graph: UnGraphMap<Node, Edge>,
    shape: BoardShape,
    dims: (Coord, Coord),
    affiliation_vars: Vec<(AffiliationHolderType, AffiliationID)>,
    affiliation_displays: Vec<char>,
}

impl GeneralNumberlinkBoard {
    fn edge_affiliation_var(&mut self, a: Node, b: Node, aff_id: AffiliationID) -> Var {
        let aff_holder = AffiliationHolderType::EDGE { nodes: UnorderedPair(a, b) };

        if !self.affiliation_vars.contains(&(aff_holder, aff_id)) {
            self.affiliation_vars.push((aff_holder, aff_id));
        }

        self.edge_affiliation_var_unchecked(a, b, aff_id)
    }

    fn edge_affiliation_var_unchecked(&self, a: Node, b: Node, aff_id: AffiliationID) -> Var {
        let aff_holder = AffiliationHolderType::EDGE { nodes: UnorderedPair(a, b) };

        Var::from_index(self.affiliation_vars.iter()
            .find_position(|(nodes, existing_aff)| (*nodes).eq(&aff_holder) && *existing_aff == aff_id)
            .and_then(|(pos, val)| Some(pos))
            .unwrap()
        )
    }

    fn solve(self) {
        // todo: every terminus vertex has an affiliation and exactly one connected edge and one neighbor with the same affiliation
        // every other connected edge has no affiliation
        // todo: every non-terminus vertex has an affiliation and exactly two connected edges and two neighbors with the same affiliation
        // every other connected edge has no affiliation
        // an edge having an affiliation <=> its vertices have the same affiliation
    }
}

impl From<&SquareNumberlinkBoardBuilder> for GeneralNumberlinkBoard {
    fn from(builder: &SquareNumberlinkBoardBuilder) -> Self {
        let shape = BoardShape::SQUARE;
        let mut graph = UnGraphMap::with_capacity(
            // naively allocate for a complete grid of this size, which usually isn't too far off
            builder.cells.len(),
            // "horizontal" edges
            (builder.dims.0 - 1) * builder.dims.1
                // "vertical" edges
                + (builder.dims.1 - 1) * builder.dims.0,
        );

        let mut nodes = Array2::from_shape_fn(builder.cells.raw_dim(), |ind| Node {
            location: Location::from(ind),
            cell: *builder.cells.get(ind).unwrap(),
        });


        for x in 0..builder.dims.0 {
            for y in 0..builder.dims.1 {
                let location = Location(x, y);
                // add edges down and to the right, if possible
                let location_below = (StepDirection::SQUARE { direction: SquareStepDirection::DOWN }).attempt_from(location);
                let location_right = (StepDirection::SQUARE { direction: SquareStepDirection::RIGHT }).attempt_from(location);

                let node = nodes.get(location.as_index()).unwrap();
                let node_below = nodes.get(location_below.as_index());
                let node_right = nodes.get(location_right.as_index());

                let edge = Edge { affiliation: None };
                node_below.and_then(|other_node| graph.add_edge(*node, *other_node, edge));
                node_right.and_then(|other_node| graph.add_edge(*node, *other_node, edge));
            }
        }

        // TODO: handle bridges, warps, any shape besides simple complete rectangle graph

        let mut affiliation_displays = Vec::with_capacity(builder.affiliation_displays.len() + 1);
        // affiliation 0 is unaffiliated and will display as empty
        affiliation_displays.push('.');
        affiliation_displays.extend(builder.affiliation_displays.clone());

        Self {
            graph,
            shape,
            dims: builder.dims,
            affiliation_displays,

            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct SquareNumberlinkBoardBuilder {
    // width, height
    pub dims: (Coord, Coord),
    cells: Array2<NumberlinkCell>,
    // TODO
    edge_blacklist: HashSet<UnorderedPair<Location>>,
    node_blacklist: HashSet<Location>,
    bridges: HashSet<Location>,
    edge_whitelist: HashSet<UnorderedPair<Location>>,
    affiliation_displays: Vec<char>,
}

impl SquareNumberlinkBoardBuilder {
    pub fn with_dims(dims: (Coord, Coord)) -> Self {
        Self {
            dims,
            cells: Array2::from_shape_simple_fn((dims.1, dims.0), NumberlinkCell::default),
            ..Default::default()
        }
    }

    pub fn add_termini(&mut self, display: char, locations: (Location, Location)) -> &mut SquareNumberlinkBoardBuilder {
        // non-null affiliation IDs start at 1
        let aff_id = self.affiliation_displays.len() + 1;
        self.affiliation_displays.push(display);
        for location in [locations.0, locations.1] {
            self.cells.index_mut(location.as_index()).assign_elem(NumberlinkCell::TERMINUS {
                affiliation: CellAffiliation {
                    ident: aff_id,
                    display,
                }
            })
        }

        self
    }

    pub fn build(&self) -> GeneralNumberlinkBoard {
        GeneralNumberlinkBoard::from(self)
    }
}