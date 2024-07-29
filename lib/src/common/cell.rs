use crate::common::affiliation::AffiliationID;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NumberlinkCell {
    TERMINUS { affiliation: AffiliationID },
    PATH { affiliation: AffiliationID },
    #[default]
    EMPTY,
}