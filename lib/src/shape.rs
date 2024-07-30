use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::num::NonZero;
use std::ops::Index;

use itertools::Itertools;
use ndarray::{Array2, AssignElem};
use petgraph::graphmap::UnGraphMap;
use strum::VariantArray;

use crate::board::{Edge, Node};
use crate::cell::{FrozenCellType, FrozenCell, Cell};
use crate::location::{Dimension, Location};

/// Functionality that must be implemented on a case-by-case basis for any board shape.
///
/// [`SquareStep`] and [`HexStep`] are built-in implementations.
pub trait Step: Sized + Copy + VariantArray + PartialEq + Eq + Hash + Ord + PartialOrd {
    /// Attempt the step from `location` in the direction specified by `self` and return the resultant [`Location`].
    fn attempt_from(&self, location: Location) -> Location;
    /// Return the static array of all "forward" directions.
    ///
    /// Forward directions should be those which, upon stepping from one location to another, cause the destination location to be indexed higher than the origin location.
    /// For example, for [`SquareStep`] and given the row-major ordering of the cell array, [`DOWN`](SquareStep::DOWN) and [`RIGHT`](SquareStep::RIGHT) are forward directions.
    fn forward_edge_directions() -> &'static [Self];
    /// Invert the direction specified by `self`.
    fn invert(&self) -> Self;
    /// Convert the graph in `board` to an array representation.
    ///
    /// New shapes should implement this and determine a scheme by which the graph can be embedded in an [`ndarray::Array2`].
    fn gph_to_array(dims: (Dimension, Dimension), board: &UnGraphMap<Node<Self>, Edge<Self>>) -> Array2<FrozenCell<Self>>;
    /// Dump the specified [`ndarray::Array2`], laying out individual characters based on the geometry of the shape [`Self`].
    fn print(board: Array2<char>) -> String;
}

#[derive(Copy, Clone, VariantArray, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub(crate) enum SquareStep {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    // switch it up like nintendo
}

impl Step for SquareStep {
    fn attempt_from(&self, location: Location) -> Location {
        match self {
            Self::UP => location.offset_by((0, -1)),
            Self::DOWN => location.offset_by((0, 1)),
            Self::LEFT => location.offset_by((-1, 0)),
            Self::RIGHT => location.offset_by((1, 0)),
        }
    }

    fn forward_edge_directions() -> &'static [Self] {
        &[Self::RIGHT, Self::DOWN]
    }

    fn invert(&self) -> Self {
        match self {
            Self::UP => Self::DOWN,
            Self::DOWN => Self::UP,
            Self::LEFT => Self::RIGHT,
            Self::RIGHT => Self::LEFT,
        }
    }

    fn gph_to_array(dims: (Dimension, Dimension), board: &UnGraphMap<Node<Self>, Edge<Self>>) -> Array2<FrozenCell<Self>> {
        let mut ret: Array2<FrozenCell<Self>> = Array2::from_shape_simple_fn((dims.1.get(), dims.0.get()), FrozenCell::default);

        for (index, ptr) in ret.indexed_iter_mut() {
            let relevant_nodes = board.nodes()
                .filter(|n| n.location == Location::from(index))
                .collect_vec();
            assert!(relevant_nodes.len() > 0);

            if relevant_nodes.len() == 1 {
                let mut exits = HashSet::with_capacity(Self::VARIANTS.len());

                let this_node = relevant_nodes.index(0);
                for edge_triple in board.edges(*this_node) {
                    let (n1, n2, e) = edge_triple;
                    let neighbor = if n1 == *this_node { n2 } else { n1 };
                    // not a warp if a "typical" step can reach the neighbor, direction_to would return Some
                    exits.insert(Self::direction_to(this_node.location, neighbor.location).unwrap_or({
                        // warp; the direction in the edge struct is correct only if this node is indexed lower than its neighbor, otherwise it is reversed
                        let mut direction = e.direction;
                        if *this_node < neighbor {
                            direction = direction.invert();
                        }

                        direction
                    }));
                }

                ptr.assign_elem(FrozenCell {
                    exits,
                    cell_type: match this_node.cell {
                        Cell::TERMINUS { affiliation } => FrozenCellType::TERMINUS { affiliation: NonZero::new(affiliation).unwrap() },
                        Cell::PATH { affiliation } => FrozenCellType::PATH { affiliation: NonZero::new(affiliation).unwrap() },
                        Cell::EMPTY => FrozenCellType::EMPTY,
                        _ => unreachable!()
                    },
                });
            } else {
                // this is a bridge
                let mut exits = HashSet::with_capacity(Self::VARIANTS.len());
                let mut affiliations = HashMap::with_capacity(Self::forward_edge_directions().len());

                for node in relevant_nodes {
                    match node.cell {
                        Cell::BRIDGE { affiliation, direction } => {
                            exits.insert(direction);
                            exits.insert(direction.invert());
                            affiliations.insert(direction.ensure_forward(), NonZero::new(affiliation.unwrap()).unwrap());
                        }
                        _ => unreachable!()
                    }
                }

                ptr.assign_elem(FrozenCell {
                    exits,
                    cell_type: FrozenCellType::BRIDGE { affiliations },
                })
            }
        }

        ret
    }

    fn print(board: Array2<char>) -> String {
        let mut out = String::with_capacity(board.nrows() * (board.ncols() + 1));

        for row in board.rows() {
            for col in row {
                out.push(*col);
            }
            out.push('\n');
        }

        out
    }
}

// NB: we organize hexagonal grids as follows:
// 0   1   2   3
//   0   1   2   3
// 0   1   2   3
//   0   1   2   3
#[derive(Copy, Clone, VariantArray, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
enum HexStep {
    UP,
    UPRIGHT,
    RIGHTDOWN,
    DOWN,
    DOWNLEFT,
    LEFTUP,
}

impl Step for HexStep {
    fn attempt_from(&self, location: Location) -> Location {
        match self {
            Self::UP => location.offset_by((0, -2)),
            // these are more complicated; consider the parity of the rows
            Self::UPRIGHT => location.offset_by((if location.1 & 2 == 0 { 1 } else { 0 }, -1)),
            Self::RIGHTDOWN => location.offset_by((if location.1 & 2 == 0 { 1 } else { 0 }, -1)),
            Self::DOWN => location.offset_by((0, 2)),
            Self::DOWNLEFT => location.offset_by((if location.1 & 2 == 0 { 0 } else { -1 }, 1)),
            Self::LEFTUP => location.offset_by((if location.1 & 2 == 0 { 0 } else { -1 }, -1)),
        }
    }

    fn forward_edge_directions() -> &'static [Self] {
        &[Self::DOWN, Self::RIGHTDOWN, Self::DOWNLEFT]
    }

    fn invert(&self) -> Self {
        match self {
            Self::UP => Self::DOWN,
            Self::UPRIGHT => Self::DOWNLEFT,
            Self::RIGHTDOWN => Self::LEFTUP,
            Self::DOWN => Self::UP,
            Self::DOWNLEFT => Self::UPRIGHT,
            Self::LEFTUP => Self::RIGHTDOWN,
        }
    }

    fn gph_to_array(dims: (Dimension, Dimension), board: &UnGraphMap<Node<Self>, Edge<Self>>) -> Array2<FrozenCell<Self>> {
        todo!()
    }

    fn print(board: Array2<char>) -> String {
        todo!()
    }
}

/// Functionality on top of [`Step`] required by [`Board`](crate::Board)s with identical implementation across all `Sh`.
pub trait BoardShape: Step {
    /// Get all neighbors of a [`Location`] in "theory", by attempting every step direction in `Self::VARIANTS`.
    fn neighbors_of(&self, location: Location) -> Vec<(Self, Location)>;
    /// Determine the direction from `a` to `b` by calling [`attempt_from`](Step::attempt_from) until one works.
    ///
    /// This is not exhaustive since it does not consider any graph-based information.
    /// It works only on two [`Location`]s which are adjacent in the array representation of their [`Board`](crate::Board) and will return [`None`] otherwise.
    fn direction_to(a: Location, b: Location) -> Option<Self>;
    /// Convert this [`Self`] to a "forward" direction, if it is not already such a direction.
    ///
    /// For the definition of forward directions, see [`Step::forward_edge_directions`].
    fn ensure_forward(&self) -> Self;
}

impl<Sh> BoardShape for Sh
where
    Sh: Step,
{
    fn neighbors_of(&self, location: Location) -> Vec<(Self, Location)> {
        Self::VARIANTS.iter()
            .map(|dir| (*dir, dir.attempt_from(location)))
            .collect_vec()
    }

    fn direction_to(a: Location, b: Location) -> Option<Self> {
        Self::VARIANTS.iter().find(|dir| dir.attempt_from(a) == b).and_then(|dir| Some(*dir))
    }

    fn ensure_forward(&self) -> Self {
        match Self::forward_edge_directions().contains(self) {
            true => *self,
            false => self.invert(),
        }
    }
}