use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::{IndexMut, Range};
use itertools::Itertools;
use ndarray::{Array2, AssignElem};
use petgraph::graphmap::UnGraphMap;
use strum::VariantArray;
use unordered_pair::UnorderedPair;
use varisat::{CnfFormula, Lit, Var};

use crate::common::affiliation::{AffiliationID, CellAffiliation};
use crate::common::location::{Coord, Location, NumberlinkCell};
use crate::common::shape::{BoardShape, SquareStep, Step};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
struct Node {
    location: Location,
    cell: NumberlinkCell,
}

#[derive(Clone, Copy)]
struct Edge<T>
where
    T: BoardShape,
{
    affiliation: Option<CellAffiliation>,
    direction: T,
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum AffiliationHolder {
    NODE { location: Location },
    EDGE { nodes: UnorderedPair<Location> },
}

pub struct GeneralNumberlinkBoard<T>
where
    T: BoardShape,
{
    graph: UnGraphMap<Node, Edge<T>>,
    dims: (Coord, Coord),
    affiliation_displays: Vec<char>,
}

impl<T> GeneralNumberlinkBoard<T>
where
    T: Step + BoardShape + VariantArray + PartialEq
{
    fn valid_affiliations(&self) -> Range<AffiliationID> {
        0..self.affiliation_displays.len()
    }

    fn valid_non_null_affiliations(&self) -> Range<AffiliationID> {
        1..self.affiliation_displays.len()
    }

    fn affiliation_var(&self, subject: AffiliationHolder, aff_id: AffiliationID) -> Var {
        Var::from_index(match subject {
            AffiliationHolder::NODE { location } => {
                (location.1 * self.dims.0 + location.0) * self.valid_affiliations().len() + aff_id
            }
            AffiliationHolder::EDGE { nodes } => {
                // compare y-values
                let lowest_index_location = match nodes.0.1.cmp(&nodes.1.1) {
                    Ordering::Less => nodes.0,
                    // tie; compare x-values
                    Ordering::Equal => if nodes.0.0 < nodes.1.0 { nodes.0 } else { nodes.1 }
                    Ordering::Greater => nodes.1,
                };

                let actual_dir = T::direction_to(nodes.0, nodes.1).unwrap().ensure_forward();

                self.dims.1 * self.dims.0 * self.valid_affiliations().len()
                    + (lowest_index_location.1 * self.dims.0 + lowest_index_location.0)
                    * T::forward_edge_directions().len() + T::forward_edge_directions().iter().find_position(|dir| **dir == actual_dir).unwrap().0
            }
        })
    }

    fn solve(mut self) {
        // every other connected edge has no affiliation
        // todo: every non-terminus vertex has an affiliation and exactly two connected edges with the same affiliation
        // every other connected edge has no affiliation
        // an edge having an affiliation <=> its vertices have the same affiliation

        let mut assumptions: Vec<Lit> = Vec::new();
        let mut formulae: Vec<CnfFormula> = Vec::new();

        for vertex in self.graph.nodes() {
            match vertex.cell {
                // every terminus vertex has the given affiliation and no other
                NumberlinkCell::TERMINUS { affiliation: CellAffiliation { ident: aff_id, .. } } => {
                    assumptions.extend(self.valid_non_null_affiliations()
                        .map(|maybe_aff| self.affiliation_var(AffiliationHolder::NODE { location: vertex.location }, maybe_aff)
                            .lit(maybe_aff == aff_id)));

                    // every terminus vertex has exactly one connected edge and one neighbor with the same affiliation
                }
                NumberlinkCell::EMPTY => {}
                _ => {}
            }
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

    pub fn build(&self) -> GeneralNumberlinkBoard<SquareStep> {
        let mut graph = UnGraphMap::with_capacity(
            // naively allocate for a complete grid of this size, which usually isn't too far off
            self.cells.len(),
            // "horizontal" edges
            (self.dims.0 - 1) * self.dims.1
                // "vertical" edges
                + (self.dims.1 - 1) * self.dims.0,
        );

        let mut nodes = Array2::from_shape_fn(self.cells.raw_dim(), |ind| Node {
            location: Location::from(ind),
            cell: *self.cells.get(ind).unwrap(),
        });


        for x in 0..self.dims.0 {
            for y in 0..self.dims.1 {
                let location = Location(x, y);
                // add edges down and to the right, if possible
                let location_below = SquareStep::DOWN.attempt_from(location);
                let location_right = SquareStep::RIGHT.attempt_from(location);

                let node = nodes.get(location.as_index()).unwrap();
                let node_below = nodes.get(location_below.as_index());
                let node_right = nodes.get(location_right.as_index());

                node_below.and_then(|other_node| graph.add_edge(*node, *other_node, Edge { affiliation: None, direction: SquareStep::DOWN }));
                node_right.and_then(|other_node| graph.add_edge(*node, *other_node, Edge { affiliation: None, direction: SquareStep::RIGHT }));
            }
        }

        // TODO: handle bridges, warps, any shape besides simple complete rectangle graph

        let mut affiliation_displays = Vec::with_capacity(self.affiliation_displays.len() + 1);
        // affiliation 0 is unaffiliated and will display as empty
        affiliation_displays.push('.');
        affiliation_displays.extend(self.affiliation_displays.clone());

        GeneralNumberlinkBoard {
            graph,
            dims: self.dims,
            affiliation_displays,
        }
    }
}