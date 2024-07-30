use std::collections::{HashMap, HashSet};
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

#[derive(Clone, Default)]
pub(crate) enum FrozenCellType<Sh: BoardShape> {
    TERMINUS { affiliation: NonZero<AffiliationID> },
    PATH { affiliation: NonZero<AffiliationID> },
    BRIDGE { affiliations: HashMap<Sh, NonZero<AffiliationID>> },
    #[default]
    EMPTY,
}

#[derive(Clone)]
pub struct FrozenNumberLinkCell<Sh: BoardShape> {
    pub(crate) exits: HashSet<Sh>,
    pub(crate) cell_type: FrozenCellType<Sh>,
}

impl<Sh: BoardShape> Default for FrozenNumberLinkCell<Sh> {
    fn default() -> Self {
        Self {
            exits: Default::default(),
            cell_type: FrozenCellType::EMPTY,
        }
    }
}