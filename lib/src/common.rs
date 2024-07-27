use std::fmt::{Display, Formatter};

type Coord = usize;
// x, y
pub type Location = (Coord, Coord);
pub type AffiliationID = usize;

#[derive(Clone, Copy, Debug)]
pub struct CellAffiliation {
    pub(crate) ident: AffiliationID,
    pub(crate) display: char,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum NumberlinkCell {
    TERMINUS { affiliation: CellAffiliation },
    PATH { affiliation: CellAffiliation },
    #[default]
    EMPTY,
}

impl Display for NumberlinkCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            NumberlinkCell::TERMINUS { affiliation } => affiliation.display.to_ascii_uppercase(),
            NumberlinkCell::PATH { affiliation } => affiliation.display.to_ascii_lowercase(),
            NumberlinkCell::EMPTY => '.'
        })
    }
}