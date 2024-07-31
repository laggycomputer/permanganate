use std::collections::{HashMap, HashSet};
use std::num::NonZero;

use crate::affiliation::AffiliationID;
use crate::shape::BoardShape;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Cell<Sh: BoardShape> {
    Terminus { affiliation: AffiliationID },
    Path { affiliation: AffiliationID },
    Bridge { affiliation: Option<AffiliationID>, direction: Sh },
    #[default]
    Empty,
}

#[derive(Clone, Default)]
pub(crate) enum FrozenCellType<Sh: BoardShape> {
    Terminus { affiliation: NonZero<AffiliationID> },
    Path { affiliation: NonZero<AffiliationID> },
    Bridge { affiliations: HashMap<Sh, Option<NonZero<AffiliationID>>> },
    #[default]
    Empty,
}

#[derive(Clone)]
pub(crate) struct FrozenCell<Sh: BoardShape> {
    pub(crate) exits: HashSet<Sh>,
    pub(crate) cell_type: FrozenCellType<Sh>,
}

impl<Sh: BoardShape> Default for FrozenCell<Sh> {
    fn default() -> Self {
        Self {
            exits: Default::default(),
            cell_type: FrozenCellType::Empty,
        }
    }
}