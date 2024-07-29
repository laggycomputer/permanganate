use std::collections::HashMap;
use std::num::NonZero;
use crate::common::affiliation::AffiliationID;
use crate::common::shape::BoardShape;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NumberlinkCell<Sh: BoardShape> {
    TERMINUS { affiliation: AffiliationID },
    PATH { affiliation: AffiliationID },
    BRIDGE { affiliation: Option<AffiliationID>, direction: Sh },
    #[default]
    EMPTY,
}

enum SolvedCellType<Sh: BoardShape> {
    TERMINUS { affiliation: NonZero<AffiliationID> },
    PATH { affiliation: NonZero<AffiliationID> },
    BRIDGE { affiliations: HashMap<Sh, NonZero<AffiliationID>> },
}

pub struct SolvedNumberlinkCell<Sh: BoardShape> {
    exits: HashMap<Sh, bool>,
    cell_type: SolvedCellType<Sh>,
}