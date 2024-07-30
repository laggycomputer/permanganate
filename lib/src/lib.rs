pub use builder::{BuilderInvalidReason, SquareNumberlinkBoardBuilder};
pub use location::Location;

pub(crate) mod graph;
mod tests;
pub(crate) mod affiliation;
pub(crate) mod location;
pub(crate) mod logic;
pub(crate) mod shape;
pub(crate) mod cell;
pub(crate) mod builder;
