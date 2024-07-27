pub type AffiliationID = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CellAffiliation {
    pub(crate) ident: AffiliationID,
    pub(crate) display: char,
}
