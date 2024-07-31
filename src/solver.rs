use std::collections::HashMap;
use std::convert::identity;
use std::hash::Hash;
use std::num::NonZero;
use std::ops::RangeInclusive;

use itertools::Itertools;
use petgraph::graphmap::{NodeTrait, UnGraphMap};
use unordered_pair::UnorderedPair;
use varisat::{CnfFormula, Lit, Solver, Var};

use crate::affiliation::AffiliationID;
use crate::logic::exactly_one;

/// Constraint on node types given to [`GraphSolver`].
pub trait Terminus: NodeTrait /* constraints on GraphMap */ {
    fn is_terminus(&self) -> Option<NonZero<AffiliationID>>;
}

/// Reasons a [`GraphSolver`] may fail.
#[derive(Debug)]
pub enum SolverFailure {
    /// The SAT solver detected a logical inconsistency, i.e. the graph as stated is unsolvable.
    Inconsistent,
    /// The SAT solver could not solve the affiliation of at least one node and/or edge.
    /// This should probably never happen.
    NoAffFound,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum HasAffiliation<N, E>
where
    N: Terminus,
{
    Node { node: N },
    Edge { edge: E, endpoints: UnorderedPair<N> },
}

impl<N, E> HasAffiliation<N, E>
where
    N: Terminus,
    E: Copy,
{
    pub(crate) fn from_node(node: N) -> Self {
        Self::Node { node }
    }

    pub(crate) fn from_edge(triple: (N, N, &E)) -> Self {
        let (n1, n2, e) = triple;
        Self::Edge { edge: *e, endpoints: UnorderedPair(n1, n2) }
    }
}

/// The most general implementation of the logic necessary to solve a graph in accordance with the rules for Numberlink.
/// Use [`Self::solve`] to attempt to find a solution.
///
/// The only requirement is that the node struct on the input graph implements [`Terminus`], so it may be noted as a Terminus.
pub struct GraphSolver<'a, N, E>
where
    N: Terminus,
{
    graph: &'a UnGraphMap<N, E>,
    affiliation_holders: Vec<HasAffiliation<N, E>>,
    max_affiliation: AffiliationID,
}

impl<'a, N, E> From<&'a UnGraphMap<N, E>> for GraphSolver<'a, N, E>
where
    N: Terminus,
    E: Copy,
{
    fn from(graph: &'a UnGraphMap<N, E>) -> Self {
        let mut affiliation_holders = Vec::with_capacity(graph.node_count() + graph.edge_count());
        let nodes = graph.nodes().collect_vec();
        let num_affiliations = match nodes.iter().filter_map(|node| node.is_terminus().and_then(Some)).max() {
            None => 0,
            Some(max) => max.get(),
        };
        affiliation_holders.extend(nodes.into_iter().map(HasAffiliation::from_node));
        affiliation_holders.extend(graph.all_edges().map(HasAffiliation::from_edge));

        Self {
            graph,
            affiliation_holders,
            max_affiliation: num_affiliations,
        }
    }
}

impl<N, E> GraphSolver<'_, N, E>
where
    N: Terminus,
    E: PartialEq + Eq + Hash + Copy,
{
    #[inline]
    fn valid_affiliations(&self) -> RangeInclusive<AffiliationID> {
        0..=self.max_affiliation
    }

    #[inline]
    fn valid_non_null_affiliations(&self) -> RangeInclusive<AffiliationID> {
        1..=self.max_affiliation
    }

    #[inline]
    fn num_affiliations(&self) -> usize {
        self.valid_affiliations().try_len().unwrap()
    }

    #[inline]
    fn affiliation_var(&self, subject: HasAffiliation<N, E>, affiliation: AffiliationID) -> Var {
        Var::from_index(self.affiliation_holders.iter().find_position(|elem| **elem == subject).unwrap().0
            * self.num_affiliations() + affiliation)
    }

    #[inline]
    fn solved_affiliation_of(&self, model: &Vec<Lit>, subject: HasAffiliation<N, E>, nonzero: bool) -> Option<AffiliationID> {
        (if nonzero { self.valid_affiliations() } else { self.valid_non_null_affiliations() })
            .find(|aff| model.get(self.affiliation_var(subject, *aff).index()).unwrap().is_positive())
    }

    /// Solve a Numberlink graph, returning [`Ok`] with a [`HashMap`] of solved affiliations for each edge and vertex or [`Err`] with a [`SolverFailure`] reason.
    ///
    /// # Logical setup
    /// Suppose this board is undirected graph G.
    ///
    /// ## Vertices
    /// Every vertex V on G must have exactly one nonzero affiliation.
    /// If V is a Terminus, its affiliation is known and all other affiliations are incorrect.
    /// Exactly one incident edge has the same affiliation (the edge by which the path exits this Terminus).
    /// Every other incident edge has no affiliation (i.e. affiliation 0).
    ///
    /// If V is not a Terminus, it must have exactly one (not yet known) affiliation A.
    /// Then V is on the path between the two termini with affiliation A and has two incident edges with affiliation A.
    /// Every other incident edge has no affiliation.
    ///
    /// ## Edges
    /// Every edge E on G has exactly one affiliation, which may be 0.
    ///
    /// The two endpoints of E have the same affiliation if and only if E has the same nonzero affiliation.
    /// So, by complement, the two endpoints of E have different affiliation if and only if E has no affiliation.
    /// We encode the former of these two biconditionals.
    pub fn solve(&self) -> Result<HashMap<HasAffiliation<N, E>, AffiliationID>, SolverFailure> {
        let mut assumptions: Vec<Lit> = Vec::new();
        let mut formulae: Vec<CnfFormula> = Vec::new();

        for vertex in self.graph.nodes() {
            // let this vertex be V
            if let Some(aff) = vertex.is_terminus() {
                // the affiliation of V is the one already assigned, and no other; we tell the solver to assume this is so
                assumptions.extend(self.valid_affiliations()
                    .map(|maybe_aff| self.affiliation_var(HasAffiliation::from_node(vertex), maybe_aff).lit(maybe_aff == aff.get())));

                // exactly one incident edge E has the same affiliation
                formulae.push(CnfFormula::from(exactly_one(
                    self.graph.edges(vertex)
                        .map(|e_triple| self.affiliation_var(HasAffiliation::from_edge(e_triple), aff.get()).positive())
                        .collect_vec()
                )));

                // V has deg(V) - 1 incident edges with affiliation 0 (unaffiliated)
                // or, equivalently, exactly 1 incident edge does *not* have affiliation 0
                formulae.push(CnfFormula::from(exactly_one(
                    self.graph.edges(vertex)
                        .map(|e_triple| self.affiliation_var(HasAffiliation::from_edge(e_triple), 0).negative())
                        .collect_vec()
                )));
            } else {
                // V must have nonzero affiliation
                assumptions.push(self.affiliation_var(HasAffiliation::from_node(vertex), 0).negative());

                // V has only one affiliation
                formulae.push(CnfFormula::from(exactly_one(
                    self.valid_non_null_affiliations()
                        .map(|aff| self.affiliation_var(HasAffiliation::from_node(vertex), aff).positive())
                        .collect_vec()
                )));

                let all_incident = self.graph.edges(vertex).collect_vec();

                for aff in self.valid_non_null_affiliations() {
                    {
                        let mut terms = Vec::with_capacity(1 + all_incident.len());
                        // V having affiliation A...
                        terms.push(self.affiliation_var(HasAffiliation::from_node(vertex), aff).negative());

                        // implies at least one incident edge E_1 has the same affiliation
                        terms.extend(all_incident.iter()
                            .map(|e_triple| self.affiliation_var(HasAffiliation::from_edge(*e_triple), aff).positive())
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
                                    .map(|e_triple| self.affiliation_var(HasAffiliation::from_edge(*e_triple), aff).lit(e1_triple != e_triple))
                                    .collect_vec()
                            })));
                    }

                    // however, no three such E exist; i.e. for any choice of 3 incident E (E_1, E_2, E_3), at least one does not have affiliation A
                    let no_three_clauses = all_incident.iter()
                        .combinations(3)
                        // one choice for (E_1, E_2, E_3) as mentioned above
                        .map(|selection| selection.iter()
                            // for each of these three, generate the literal stating its affiliation is not A
                            .map(|e_triple| self.affiliation_var(HasAffiliation::from_edge(**e_triple), aff).negative())
                            .collect_vec()
                        );

                    formulae.push(CnfFormula::from(no_three_clauses));
                }
            }
        }

        for edge_triple in self.graph.all_edges() {
            // this edge E has exactly one affiliation, which may be 0
            formulae.push(CnfFormula::from(exactly_one(
                self.valid_affiliations()
                    .map(|aff| self.affiliation_var(HasAffiliation::from_edge(edge_triple), aff).positive())
                    .collect_vec()
            )));

            for aff in self.valid_non_null_affiliations() {
                // E having a non-null affiliation <=> its vertices have the same affiliation
                // let this be A <=> BC
                // A => BC = !A + BC = (!A + B)(!A + C)
                // BC => A = !(BC) + A = !B + !C + A
                // together, A <=> BC = (!A + B)(!A + C)(A + !B + !C)
                let a = self.affiliation_var(HasAffiliation::from_edge(edge_triple), aff);
                let b = self.affiliation_var(HasAffiliation::from_node(edge_triple.0), aff);
                let c = self.affiliation_var(HasAffiliation::from_node(edge_triple.1), aff);

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
            return Err(SolverFailure::Inconsistent);
        };
        let model = solver.model().unwrap();

        let mut solved_affiliations = HashMap::new();

        for node in self.graph.nodes() {
            solved_affiliations.insert(
                HasAffiliation::from_node(node),
                match self.solved_affiliation_of(&model, HasAffiliation::from_node(node), false) {
                    None => return Err(SolverFailure::NoAffFound),
                    Some(aff) => aff
                });
        }

        for edge_triple in self.graph.all_edges() {
            solved_affiliations.insert(
                HasAffiliation::from_edge(edge_triple),
                match self.solved_affiliation_of(&model, HasAffiliation::from_edge(edge_triple), true) {
                    None => return Err(SolverFailure::NoAffFound),
                    Some(aff) => aff
                });
        }

        Ok(solved_affiliations)
    }
}
