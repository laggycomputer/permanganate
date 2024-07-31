use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::num::NonZero;
use std::ops::Index;

use itertools::Itertools;
use ndarray::{Array2, AssignElem};
use petgraph::graphmap::UnGraphMap;
use strum::VariantArray;

use crate::board::{Edge, Node};
use crate::cell::{Cell, FrozenCell, FrozenCellType};
use crate::location::{Dimension, Location};

/// Functionality that must be implemented on a case-by-case basis for any board shape.
///
/// [`SquareStep`] and [`HexStep`] are built-in implementations.
pub trait Step: Sized + Copy + VariantArray + PartialEq + Eq + Hash + Ord + PartialOrd {
    /// Attempt the step from `location` in the direction specified by `self` and return the resultant [`Location`].
    fn attempt_from(&self, location: Location) -> Location;
    /// The static array of all "forward" directions.
    ///
    /// Forward directions should be those which, upon stepping from one location to another, cause the destination location to be indexed higher than the origin location.
    /// For example, for [`SquareStep`] and given the row-major ordering of the cell array, [`DOWN`](SquareStep::Down) and [`RIGHT`](SquareStep::Right) are forward directions.
    const FORWARD_VARIANTS: &'static [Self];
    /// Invert the direction specified by `self`.
    fn invert(&self) -> Self;
    /// Convert the graph in `board` to an array representation.
    ///
    /// New shapes should implement this and determine a scheme by which the graph can be embedded in an [`ndarray::Array2`].
    fn gph_to_array(dims: (Dimension, Dimension), board: &UnGraphMap<Node<Self>, Edge<Self>>) -> Array2<FrozenCell<Self>>;
    /// Dump the specified [`ndarray::Array2`], laying out individual characters based on the geometry of the shape [`Self`].
    fn print(board: Array2<char>) -> String;
}

/// The square cell type and rectangular board shape, as found in Numberlink puzzles, Flow Free, and the Bridges and Warps expansions.
#[derive(Copy, Clone, VariantArray, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub enum SquareStep {
    Up,
    Down,
    Left,
    Right,
    // switch it up like nintendo
}

impl Step for SquareStep {
    fn attempt_from(&self, location: Location) -> Location {
        match self {
            Self::Up => location.offset_by((0, -1)),
            Self::Down => location.offset_by((0, 1)),
            Self::Left => location.offset_by((-1, 0)),
            Self::Right => location.offset_by((1, 0)),
        }
    }

    const FORWARD_VARIANTS: &'static [Self] = &[Self::Right, Self::Down];

    fn invert(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
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
                        Cell::Terminus { affiliation } => FrozenCellType::Terminus { affiliation: NonZero::new(affiliation).unwrap() },
                        Cell::Path { affiliation } => FrozenCellType::Path { affiliation: NonZero::new(affiliation).unwrap() },
                        Cell::Empty => FrozenCellType::Empty,
                        _ => unreachable!()
                    },
                });
            } else {
                // this is a bridge
                let mut exits = HashSet::with_capacity(Self::VARIANTS.len());
                let mut affiliations = HashMap::with_capacity(Self::FORWARD_VARIANTS.len());

                for node in relevant_nodes {
                    match node.cell {
                        Cell::Bridge { affiliation, direction } => {
                            exits.insert(direction);
                            exits.insert(direction.invert());
                            affiliations.insert(
                                direction.ensure_forward(),
                                affiliation.and_then(|aff| NonZero::new(aff)),
                            );
                        }
                        _ => unreachable!()
                    }
                }

                ptr.assign_elem(FrozenCell {
                    exits,
                    cell_type: FrozenCellType::Bridge { affiliations },
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
    Up,
    UpRight,
    RightDown,
    Down,
    DownLeft,
    LeftUp,
}

impl Step for HexStep {
    fn attempt_from(&self, location: Location) -> Location {
        match self {
            Self::Up => location.offset_by((0, -2)),
            // these are more complicated; consider the parity of the rows
            Self::UpRight => location.offset_by((if location.1 & 2 == 0 { 1 } else { 0 }, -1)),
            Self::RightDown => location.offset_by((if location.1 & 2 == 0 { 1 } else { 0 }, -1)),
            Self::Down => location.offset_by((0, 2)),
            Self::DownLeft => location.offset_by((if location.1 & 2 == 0 { 0 } else { -1 }, 1)),
            Self::LeftUp => location.offset_by((if location.1 & 2 == 0 { 0 } else { -1 }, -1)),
        }
    }

    const FORWARD_VARIANTS: &'static [Self] = &[Self::Down, Self::RightDown, Self::DownLeft];

    fn invert(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::UpRight => Self::DownLeft,
            Self::RightDown => Self::LeftUp,
            Self::Down => Self::Up,
            Self::DownLeft => Self::UpRight,
            Self::LeftUp => Self::RightDown,
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
    /// For the definition of forward directions, see [`Step::FORWARD_VARIANTS`].
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
        match Self::FORWARD_VARIANTS.contains(self) {
            true => *self,
            false => self.invert(),
        }
    }
}