#![warn(missing_docs)]

pub use location::Location;
pub use graph::GeneralNumberlinkBoard;

pub(crate) mod graph;
mod tests;
pub(crate) mod affiliation;
pub(crate) mod location;
pub(crate) mod logic;
pub(crate) mod shape;
pub(crate) mod cell;
pub mod builder;
