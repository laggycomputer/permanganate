use std::collections::{HashMap, HashSet};
use std::num::NonZero;

use crate::affiliation::AffiliationID;
use crate::shape::FullShape;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Cell<Sh: FullShape> {
    Terminus { affiliation: AffiliationID },
    Path { affiliation: AffiliationID },
    Bridge { affiliation: Option<AffiliationID>, direction: Sh },
    #[default]
    Empty,
}

#[derive(Clone, Default)]
pub(crate) enum FrozenCellType<Sh: FullShape> {
    Terminus { affiliation: NonZero<AffiliationID> },
    Path { affiliation: NonZero<AffiliationID> },
    Bridge { affiliations: HashMap<Sh, Option<NonZero<AffiliationID>>> },
    #[default]
    Empty,
}

/// Cells, frozen for output or printing.
#[derive(Clone)]
pub(crate) struct FrozenCell<Sh: FullShape> {
    pub(crate) exits: HashSet<Sh>,
    pub(crate) cell_type: FrozenCellType<Sh>,
}

impl<Sh: FullShape> Default for FrozenCell<Sh> {
    fn default() -> Self {
        Self {
            exits: Default::default(),
            cell_type: FrozenCellType::Empty,
        }
    }
}