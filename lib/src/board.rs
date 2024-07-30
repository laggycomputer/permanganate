use std::cmp::min;
use std::collections::HashSet;
use std::convert::identity;
use std::fmt::{Display, Formatter};
use std::ops::Range;

use itertools::Itertools;
use petgraph::graphmap::UnGraphMap;
use petgraph::prelude::GraphMap;
use petgraph::visit::IntoNodeIdentifiers;
use unordered_pair::UnorderedPair;
use varisat::{CnfFormula, Lit, Solver, Var};

use crate::affiliation::AffiliationID;
use crate::cell::{FrozenCellType, Cell};
use crate::location::{Dimension, Location};
use crate::logic::exactly_one;
use crate::shape::BoardShape;

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
/// See the [`BoardShape`] and [`Step`] traits for more information.
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
    fn valid_affiliations(&self) -> Range<AffiliationID> {
        0..self.affiliation_displays.len()
    }

    fn valid_non_null_affiliations(&self) -> Range<AffiliationID> {
        1..self.affiliation_displays.len()
    }

    fn affiliation_var(&self, subject: HasAffiliation<Sh>, affiliation: AffiliationID) -> Var {
        Var::from_index(match subject {
            HasAffiliation::NODE { location } => {
                (location.1 * self.dims.0.get() + location.0) * self.valid_affiliations().len() + affiliation
            }
            HasAffiliation::EDGE { nodes, direction } => {
                // #[derive(Ord)] on Node will give the node with lower index first here
                let lower_index_location = min(nodes.0, nodes.1);

                // offset out of addressing space for nodes
                self.dims.1.get() * self.dims.0.get() * self.valid_affiliations().len()
                    // offset for location...
                    + ((lower_index_location.1 * self.dims.0.get() + lower_index_location.0)
                    // then edge "direction"...
                    * Sh::forward_edge_directions().len() + Sh::forward_edge_directions().iter().find_position(|dir| **dir == direction.ensure_forward()).unwrap().0)
                    // then affiliation
                    * self.valid_affiliations().len() + affiliation
            }
        })
    }

    fn solved_affiliation_of(&self, model: &Vec<Lit>, subject: HasAffiliation<Sh>, can_be_null: bool) -> AffiliationID {
        (if can_be_null { self.valid_affiliations() } else { self.valid_non_null_affiliations() })
            .find(|aff| model.get(self.affiliation_var(subject, *aff).index()).unwrap().is_positive())
            .unwrap()
    }

    /// Solves this board, mutating and consuming `self` and returning a solved version of `self`.
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
        let mut assumptions: Vec<Lit> = Vec::new();
        let mut formulae: Vec<CnfFormula> = Vec::new();

        for vertex in self.graph.nodes() {
            // let this vertex be V
            match vertex.cell {
                Cell::TERMINUS { affiliation: aff } => {
                    // the affiliation of V is the one already assigned, and no other; we tell the solver to assume this is so
                    assumptions.extend(self.valid_affiliations()
                        .map(|maybe_aff| self.affiliation_var(HasAffiliation::from(vertex), maybe_aff).lit(maybe_aff == aff)));

                    // exactly one incident edge E has the same affiliation
                    formulae.push(CnfFormula::from(exactly_one(
                        self.graph.edges(vertex)
                            .map(|e_triple| self.affiliation_var(HasAffiliation::from(&e_triple), aff).positive())
                            .collect_vec()
                    )));

                    // V has deg(V) - 1 incident edges with affiliation 0 (unaffiliated)
                    // or, equivalently, exactly 1 incident edge does *not* have affiliation 0
                    formulae.push(CnfFormula::from(exactly_one(
                        self.graph.edges(vertex)
                            .map(|e_triple| self.affiliation_var(HasAffiliation::from(&e_triple), 0).negative())
                            .collect_vec()
                    )));
                }
                Cell::EMPTY => {
                    // V must have nonzero affiliation
                    assumptions.push(self.affiliation_var(HasAffiliation::from(vertex), 0).negative());

                    // V has only one affiliation
                    formulae.push(CnfFormula::from(exactly_one(
                        self.valid_non_null_affiliations()
                            .map(|aff| self.affiliation_var(HasAffiliation::from(vertex), aff).positive())
                            .collect_vec()
                    )));

                    let all_incident = self.graph.edges(vertex)
                        .collect::<HashSet<(Node<Sh>, Node<Sh>, &Edge<Sh>)>>();

                    for aff in self.valid_non_null_affiliations() {
                        {
                            let mut terms = Vec::with_capacity(1 + all_incident.len());
                            // V having affiliation A...
                            terms.push(self.affiliation_var(HasAffiliation::from(vertex), aff).negative());

                            // implies at least one incident edge E_1 has the same affiliation
                            terms.extend(all_incident.iter()
                                .map(|e_triple| self.affiliation_var(HasAffiliation::from(e_triple), aff).positive())
                            );

                            formulae.push(CnfFormula::from(vec![terms]))
                        }

                        // todo: consider adding (V does not have affiliation A) => (no incident edge has affiliation A)

                        {
                            formulae.push(CnfFormula::from(all_incident.iter()
                                .map(|e1_triple| {
                                    // some incident E_0 having affiliation A implies that another E incident to V has affiliation A
                                    // or, if we let X = (E_0 has affiliation A), Y = (E_1 has affiliation A), Z = (E_2 has affiliation A), and so on...
                                    // X => Y + Z + ...
                                    // = !X + Y + Z + ...
                                    // in other words, the variable is positive for all incident E unless E is E_1
                                    all_incident.iter()
                                        .map(|e_triple| self.affiliation_var(HasAffiliation::from(e_triple), aff).lit(e1_triple != e_triple))
                                        .collect_vec()
                                })));
                        }

                        // however, no three such E exist; i.e. for any choice of 3 incident E (E_1, E_2, E_3), at least one does not have affiliation A
                        let no_three_clauses = all_incident.iter()
                            .combinations(3)
                            // one choice for (E_1, E_2, E_3) as mentioned above
                            .map(|selection| selection.iter()
                                // for each of these three, generate the literal stating its affiliation is not A
                                .map(|e_triple| self.affiliation_var(HasAffiliation::from(*e_triple), aff).negative())
                                .collect_vec()
                            );

                        formulae.push(CnfFormula::from(no_three_clauses));
                    }
                }
                _ => {}
            }
        }

        for edge_triple in self.graph.all_edges() {
            // this edge E has exactly one affiliation, which may be 0
            formulae.push(CnfFormula::from(exactly_one(
                self.valid_affiliations()
                    .map(|aff| self.affiliation_var(HasAffiliation::from(&edge_triple), aff).positive())
                    .collect_vec()
            )));

            for aff in self.valid_non_null_affiliations() {
                // E having a non-null affiliation <=> its vertices have the same affiliation
                // let this be A <=> BC
                // A => BC = !A + BC = (!A + B)(!A + C)
                // BC => A = !(BC) + A = !B + !C + A
                // together, A <=> BC = (!A + B)(!A + C)(A + !B + !C)
                let a = self.affiliation_var(HasAffiliation::from(&edge_triple), aff);
                let b = self.affiliation_var(HasAffiliation::from(edge_triple.0), aff);
                let c = self.affiliation_var(HasAffiliation::from(edge_triple.1), aff);

                formulae.push(CnfFormula::from(vec![
                    vec![a.negative(), b.positive()],
                    vec![a.negative(), c.positive()],
                    vec![a.positive(), b.negative(), c.negative()],
                ]))
            }
        }

        let mut solver = Solver::new();
        formulae.into_iter().for_each(|formula| solver.add_formula(&formula));
        solver.assume(assumptions.into_iter().as_ref());
        if !solver.solve().is_ok_and(identity) {
            return None;
        };
        let model = solver.model().unwrap();

        let mut solved_graph: UnGraphMap<Node<Sh>, Edge<Sh>> = GraphMap::with_capacity(self.graph.node_count(), self.graph.edge_count());
        for existing_node in self.graph.node_identifiers() {
            let solved_aff = self.solved_affiliation_of(&model, HasAffiliation::from(existing_node), false);

            let mut new_node = existing_node.clone();
            if existing_node.cell == Cell::EMPTY {
                new_node.cell = Cell::PATH { affiliation: solved_aff }
            }
            // existing terminus and path cells can stay as is

            solved_graph.add_node(new_node);
        }

        for triple in self.graph.all_edges() {
            let (n1, n2, e) = triple;
            let solved_aff = self.solved_affiliation_of(&model, HasAffiliation::from(&triple), true);

            let mut new_e = *e;
            new_e.affiliation = solved_aff;

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