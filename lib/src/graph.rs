use std::cmp::min;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::num::NonZero;
use std::ops::{IndexMut, Range};

use itertools::Itertools;
use ndarray::{Array2, AssignElem};
use petgraph::graphmap::UnGraphMap;
use petgraph::prelude::GraphMap;
use petgraph::visit::IntoNodeIdentifiers;
use unordered_pair::UnorderedPair;
use varisat::{CnfFormula, Lit, Solver, Var};

use crate::common::affiliation::AffiliationID;
use crate::common::location::{Dimension, Location, NumberlinkCell};
use crate::common::logic::exactly_one;
use crate::common::shape::{BoardShape, SquareStep, Step};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) struct Node {
    pub(crate) location: Location,
    pub(crate) cell: NumberlinkCell,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct Edge<T>
where
    T: BoardShape,
{
    affiliation: AffiliationID,
    direction: T,
}

#[derive(Eq, PartialEq, Clone, Copy, Hash)]
enum HasAffiliation<T> {
    NODE { location: Location },
    EDGE { nodes: UnorderedPair<Location>, direction: T },
}

impl<T: BoardShape> From<&(Node, Node, &Edge<T>)> for HasAffiliation<T>
{
    fn from(value: &(Node, Node, &Edge<T>)) -> Self {
        Self::EDGE { nodes: UnorderedPair::from((value.0.location, value.1.location)), direction: value.2.direction }
    }
}

impl<T> From<Node> for HasAffiliation<T>
{
    fn from(value: Node) -> Self {
        Self::NODE { location: value.location }
    }
}

pub struct GeneralNumberlinkBoard<T>
where
    T: BoardShape,
{
    graph: UnGraphMap<Node, Edge<T>>,
    dims: (Dimension, Dimension),
    affiliation_displays: Vec<char>,
}

impl<T> GeneralNumberlinkBoard<T>
where
    T: BoardShape,
{
    fn valid_affiliations(&self) -> Range<AffiliationID> {
        0..self.affiliation_displays.len()
    }

    fn valid_non_null_affiliations(&self) -> Range<AffiliationID> {
        1..self.affiliation_displays.len()
    }

    fn affiliation_var(&self, subject: HasAffiliation<T>, affiliation: AffiliationID) -> Var {
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
                    * T::forward_edge_directions().len() + T::forward_edge_directions().iter().find_position(|dir| **dir == direction.ensure_forward()).unwrap().0)
                    // then affiliation
                    * self.valid_affiliations().len() + affiliation
            }
        })
    }

    fn solved_affiliation_of(&self, model: &Vec<Lit>, subject: HasAffiliation<T>, can_be_null: bool) -> AffiliationID {
        (if can_be_null { self.valid_affiliations() } else { self.valid_non_null_affiliations() })
            .find(|aff| model.get(self.affiliation_var(subject, *aff).index()).unwrap().is_positive())
            .unwrap()
    }

    pub fn solve(mut self) -> Self {
        let mut assumptions: Vec<Lit> = Vec::new();
        let mut formulae: Vec<CnfFormula> = Vec::new();

        for vertex in self.graph.nodes() {
            // let this vertex be V
            match vertex.cell {
                NumberlinkCell::TERMINUS { affiliation: aff } => {
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
                NumberlinkCell::EMPTY => {
                    // V must have nonzero affiliation
                    assumptions.push(self.affiliation_var(HasAffiliation::from(vertex), 0).negative());

                    // V has only one affiliation
                    formulae.push(CnfFormula::from(exactly_one(
                        self.valid_non_null_affiliations()
                            .map(|aff| self.affiliation_var(HasAffiliation::from(vertex), aff).positive())
                            .collect_vec()
                    )));

                    let all_incident = self.graph.edges(vertex)
                        .collect::<HashSet<(Node, Node, &Edge<T>)>>();

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
        solver.solve().unwrap();
        let model = solver.model().unwrap();
        println!("{:?}", model);

        let mut solved_graph: UnGraphMap<Node, Edge<T>> = GraphMap::with_capacity(self.graph.node_count(), self.graph.edge_count());
        for existing_node in self.graph.node_identifiers() {
            let solved_aff = self.solved_affiliation_of(&model, HasAffiliation::from(existing_node), false);

            let mut new_node = existing_node.clone();
            if existing_node.cell == NumberlinkCell::EMPTY {
                new_node.cell = NumberlinkCell::PATH { affiliation: solved_aff }
            }
            // existing terminus and path cells can stay as is

            solved_graph.add_node(new_node);
        }

        // for (n1, n2, e) in self.graph.all_edges() {
        //     let solved_aff = self.solved_affiliation_of(&model, HasAffiliation::from(&(n1, n2, e)), false);
        //
        //     let mut new_e = *e;
        //     new_e.affiliation = solved_aff;
        //
        //     solved_graph.add_edge(n1, n2, new_e);
        // }

        T::gph_to_array(self.dims, &solved_graph)

        self.graph = solved_graph;
        self
    }
}

impl<T: BoardShape> Display for GeneralNumberlinkBoard<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", T::print(T::gph_to_array(self.dims, &self.graph).map(|cell| match cell {
            NumberlinkCell::TERMINUS { affiliation } => {
                self.affiliation_displays.get(*affiliation).unwrap().to_ascii_uppercase()
            }
            NumberlinkCell::PATH { affiliation } => {
                self.affiliation_displays.get(*affiliation).unwrap().to_ascii_lowercase()
            }
            NumberlinkCell::EMPTY => '.',
        })))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BuilderInvalidReason {
    FeatureOutOfBounds,
}

pub struct SquareNumberlinkBoardBuilder {
    // width, height
    pub dims: (Dimension, Dimension),
    cells: Array2<NumberlinkCell>,
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
    pub fn with_dims(dims: (Dimension, Dimension)) -> Self {
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

    pub fn add_termini(&mut self, display: char, locations: (Location, Location)) -> &mut SquareNumberlinkBoardBuilder {
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

    pub fn build(&self) -> Result<GeneralNumberlinkBoard<SquareStep>, Vec<BuilderInvalidReason>> {
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