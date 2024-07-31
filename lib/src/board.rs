use std::fmt::{Display, Formatter};
use std::num::NonZero;

use petgraph::graphmap::UnGraphMap;
use petgraph::prelude::GraphMap;
use unordered_pair::UnorderedPair;

use crate::affiliation::AffiliationID;
use crate::cell::{Cell, FrozenCellType};
use crate::location::{Dimension, Location};
use crate::shape::BoardShape;
use crate::solver;
use crate::solver::{GraphSolver, Terminus};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) struct Node<Sh: BoardShape> {
    pub(crate) location: Location,
    pub(crate) cell: Cell<Sh>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct Edge<Sh>
where
    Sh: BoardShape,
{
    pub(crate) affiliation: AffiliationID,
    // direction from lower indexed edge
    pub(crate) direction: Sh,
}

impl<Sh> Terminus for Node<Sh>
where
    Sh: BoardShape,
{
    fn is_terminus(&self) -> Option<NonZero<AffiliationID>> {
        match self.cell {
            Cell::TERMINUS { affiliation, .. } => Some(NonZero::new(affiliation).unwrap()),
            _ => None
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Hash)]
enum HasAffiliation<Sh> {
    NODE { location: Location },
    EDGE { nodes: UnorderedPair<Location>, direction: Sh },
}

impl<Sh: BoardShape> From<&(Node<Sh>, Node<Sh>, &Edge<Sh>)> for HasAffiliation<Sh>
{
    fn from(value: &(Node<Sh>, Node<Sh>, &Edge<Sh>)) -> Self {
        Self::EDGE { nodes: UnorderedPair::from((value.0.location, value.1.location)), direction: value.2.direction }
    }
}

impl<Sh: BoardShape> From<Node<Sh>> for HasAffiliation<Sh>
{
    fn from(value: Node<Sh>) -> Self {
        Self::NODE { location: value.location }
    }
}

/// A board object using cells organized as specified by `Sh`.
/// See the [`BoardShape`] and [`Step`](crate::shape::Step) traits for more information.
///
/// [`Board`]s should be built using a [`Builder`](crate::builder::Builder) such as [`SquareBoardBuilder`](crate::builder::SquareBoardBuilder).
pub struct Board<Sh>
where
    Sh: BoardShape,
{
    pub(crate) graph: UnGraphMap<Node<Sh>, Edge<Sh>>,
    pub(crate) dims: (Dimension, Dimension),
    pub(crate) affiliation_displays: Vec<char>,
}

impl<Sh> Board<Sh>
where
    Sh: BoardShape,
{
    /// Solves this board, mutating and consuming `self` and returning a solved version of `self`.
    /// If this board is unsolvable, return [`None`].
    /// Otherwise, return `Some(self)`.
    ///
    /// # Logical setup
    /// Suppose this board is undirected graph G.
    ///
    /// ## Vertices
    /// Every vertex V on G must have exactly one nonzero affiliation.
    /// If V is a terminus, its affiliation is known and all other affiliations are incorrect.
    /// Exactly one incident edge has the same affiliation (the edge by which the path exits this terminus).
    /// Every other incident edge has no affiliation (i.e. affiliation 0).
    ///
    /// If V is not a terminus, it must have exactly one (not yet known) affiliation A.
    /// Then V is on the path between the two termini with affiliation A and has two incident edges with affiliation A.
    /// Every other incident edge has no affiliation.
    ///
    /// ## Edges
    /// Every edge E on G has exactly one affiliation, which may be 0.
    ///
    /// The two endpoints of E have the same affiliation if and only if E has the same nonzero affiliation.
    /// So, by complement, the two endpoints of E have different affiliation if and only if E has no affiliation.
    /// We encode the former of these two biconditionals.
    pub fn solve(mut self) -> Option<Self> {
        let solver = GraphSolver::from(&self.graph);
        let solution = solver.solve().unwrap();

        let mut solved_graph: UnGraphMap<Node<Sh>, Edge<Sh>> = GraphMap::with_capacity(self.graph.node_count(), self.graph.edge_count());
        for node in self.graph.nodes() {
            let mut new_node = node.clone();
            if node.cell == Cell::EMPTY {
                new_node.cell = Cell::PATH { affiliation: *solution.get(&solver::HasAffiliation::from_node(node)).unwrap() }
            }
            // existing terminus and path cells can stay as is

            solved_graph.add_node(new_node);
        }

        for triple in self.graph.all_edges() {
            let (n1, n2, e) = triple;

            let mut new_e = *e;
            new_e.affiliation = *solution.get(&solver::HasAffiliation::from_edge(triple)).unwrap();

            solved_graph.add_edge(
                solved_graph.nodes().find(|n| n.location == n1.location).unwrap(),
                solved_graph.nodes().find(|n| n.location == n2.location).unwrap(),
                new_e);
        }

        self.graph = solved_graph;
        Some(self)
    }
}

impl<Sh: BoardShape> Display for Board<Sh> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Sh::print(Sh::gph_to_array(self.dims, &self.graph).map(|cell| match cell.cell_type {
            FrozenCellType::TERMINUS { affiliation } => self.affiliation_displays.get(affiliation.get()).unwrap().to_ascii_uppercase(),
            FrozenCellType::PATH { affiliation } => self.affiliation_displays.get(affiliation.get()).unwrap().to_ascii_lowercase(),
            FrozenCellType::BRIDGE { .. } => '+',
            FrozenCellType::EMPTY => '.',
        })))
    }
}