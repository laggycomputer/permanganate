pub type AffiliationID = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Affiliation {
    pub(crate) ident: AffiliationID,
    pub(crate) display: char,
}
