use std::fmt::{Display, Formatter};
use std::num::NonZero;

use petgraph::graphmap::UnGraphMap;
use petgraph::prelude::GraphMap;
use unordered_pair::UnorderedPair;

use crate::affiliation::AffiliationID;
use crate::cell::{Cell, FrozenCellType};
use crate::location::{Dimension, Location};
use crate::shape::FullShape;
use crate::solver;
use crate::solver::{GraphSolver, SolverFailure, Terminus};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) struct Node<Sh: FullShape> {
    pub(crate) location: Location,
    pub(crate) cell: Cell<Sh>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct Edge<Sh>
where
    Sh: FullShape,
{
    pub(crate) affiliation: AffiliationID,
    // direction from lower indexed edge
    pub(crate) direction: Sh,
}

impl<Sh> Terminus for Node<Sh>
where
    Sh: FullShape,
{
    fn is_terminus(&self) -> Option<NonZero<AffiliationID>> {
        match self.cell {
            Cell::Terminus { affiliation, .. } => Some(NonZero::new(affiliation).unwrap()),
            _ => None
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Hash)]
enum HasAffiliation<Sh> {
    Node { location: Location },
    Edge { nodes: UnorderedPair<Location>, direction: Sh },
}

impl<Sh: FullShape> From<&(Node<Sh>, Node<Sh>, &Edge<Sh>)> for HasAffiliation<Sh>
{
    fn from(value: &(Node<Sh>, Node<Sh>, &Edge<Sh>)) -> Self {
        Self::Edge { nodes: UnorderedPair::from((value.0.location, value.1.location)), direction: value.2.direction }
    }
}

impl<Sh: FullShape> From<Node<Sh>> for HasAffiliation<Sh>
{
    fn from(value: Node<Sh>) -> Self {
        Self::Node { location: value.location }
    }
}

/// A board object using cells organized as specified by `Sh`.
/// See the [`FullShape`] and [`Step`](crate::shape::Shape) traits for more information.
///
/// [`Board`]s should be built using a [`Builder`](crate::builder::Builder) such as [`SquareBoardBuilder`](crate::builder::SquareBoardBuilder).
pub struct Board<Sh>
where
    Sh: FullShape,
{
    pub(crate) graph: UnGraphMap<Node<Sh>, Edge<Sh>>,
    pub(crate) dims: (Dimension, Dimension),
    pub(crate) affiliation_displays: Vec<char>,
}

impl<Sh> Board<Sh>
where
    Sh: FullShape,
{
    /// Solves this board, deferring to a [`GraphSolver`](crate::solver::GraphSolver) and mutating and returning `self` accordingly.
    ///
    /// Returns according to the result of [`GraphSolver::solve`](crate::solver::GraphSolver::solve).
    pub fn solve(mut self) -> Result<Self, SolverFailure> {
        let solver = GraphSolver::from(&self.graph);
        let solution = solver.solve()?;

        let mut solved_graph: UnGraphMap<Node<Sh>, Edge<Sh>> = GraphMap::with_capacity(self.graph.node_count(), self.graph.edge_count());
        for node in self.graph.nodes() {
            let mut new_node = node.clone();
            if node.cell == Cell::Empty {
                new_node.cell = Cell::Path { affiliation: *solution.get(&solver::HasAffiliation::from_node(node)).unwrap() }
            }
            // existing Terminus and path cells can stay as is

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
        Ok(self)
    }
}

impl<Sh: FullShape> Display for Board<Sh> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Sh::print(Sh::gph_to_array(self.dims, &self.graph).map(|cell| match cell.cell_type {
            FrozenCellType::Terminus { affiliation } => self.affiliation_displays.get(affiliation.get()).unwrap().to_ascii_uppercase(),
            FrozenCellType::Path { affiliation } => self.affiliation_displays.get(affiliation.get()).unwrap().to_ascii_lowercase(),
            FrozenCellType::Bridge { .. } => '+',
            FrozenCellType::Empty => '.',
        })))
    }
}