#![warn(missing_docs)]

//! # `permanganate`
//!
//! A solver for [Numberlink](https://en.wikipedia.org/wiki/Numberlink) and variants as posited in the mobile game Flow Free and its expansions.
//! Begin by building a board object using a builder such as [`SquareBoardBuilder`](builder::SquareBoardBuilder) or others in the [`builder`] module.
//! Convert it to a board object, then call [`solve()`](crate::Board::solve), consuming the board and yielding a solved version of the board.
//!
//! `permanganate` can operate on generic board shapes, as encoded by the `Sh` type parameter.
//! These shapes must implement [`Shape`](crate::shape::Shape) and will automatically have [`FullShape`](crate::shape::FullShape) `impl`'d as well.
//!
//! # Internals
//! This crate is driven by expressing the problem as a Boolean satisfiability problem (a "SAT"), extracting information from that solver, and re-expressing the board accordingly.
//! Earlier work into this area such as [Matt Zucker's solution](https://mzucker.github.io/2016/09/02/eating-sat-flavored-crow.html) relies on a notion of "path shape" which is only sufficient for rectangular boards free of bridges and warps.
//! Here, we generalize along lines similar to [Ben Torvaney's project](https://torvaney.github.io/projects/flow-solver.html) and [Sam Goldman's work](https://github.com/samgoldman/FlowSolver).
//!
//! A high level overview is as follows:
//!
//! Given input, express the board as an undirected graph G. A vertex corresponds to a cell as seen in-game and edges, naturally, encode connections between vertices.
//! By using the most general terms possible, we can express map features such as bridges and warps and cell shapes beyond a rectangular grid.
//!
//! We make the following assertions in SAT form:
//! 1. Every vertex is either a "terminus" (the origin of a flow) or a "path" (part of the path from one Terminus to another).
//! All cells must be colored, so this vertex has some "affiliation" not equal to the null affiliation, 0.
//! If V is a Terminus, exactly one incident edge has the same affiliation as V.
//! Otherwise, exactly two incident edges have the same affiliation.
//! 2. Every edge either has affiliation 0, meaning its endpoints have different affiliations, or has a nonzero affiliation, meaning it shares an affiliation with its endpoints and is on the path from one identically affiliated Terminus to the other.
//!
//! We then solve and assign data to the graph accordingly.
//! This is more performant than backtracking or graph algorithm based solutions.

pub use board::Board;
pub use builder::Builder;
pub use location::Location;

pub(crate) mod board;
mod tests;
pub(crate) mod affiliation;
pub(crate) mod location;
pub(crate) mod logic;
pub mod shape;
pub(crate) mod cell;
pub mod builder;
pub(crate) mod solver;
