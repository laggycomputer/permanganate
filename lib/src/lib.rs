// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }

use std::fmt::{Display, Formatter};
use std::ops::IndexMut;

use ndarray::{Array2, AssignElem};
use unordered_pair::UnorderedPair;

type Coord = usize;
// x, y
type Location = (Coord, Coord);
type CellAffiliation = usize;

enum BoardTraverseDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    // switch it up like nintendo
}

#[derive(Copy, Clone)]
pub struct NumberlinkCell {
    affiliation: Option<CellAffiliation>,
    is_terminus: bool,
}

impl Default for NumberlinkCell {
    fn default() -> Self {
        Self {
            affiliation: None,
            is_terminus: false,
        }
    }
}

const PATH_CHARS: [char; 26] = [
    'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j',
    'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't',
    'u', 'v', 'w', 'x', 'y',
    'z',
];

impl Display for NumberlinkCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self.is_terminus {
            true => PATH_CHARS[self.affiliation.unwrap()].to_ascii_uppercase(),
            false => match self.affiliation {
                Some(aff) => PATH_CHARS[aff],
                None => '.'
            }
        })
    }
}

pub struct NumberlinkBoard {
    dims: Location,
    cells: Array2<NumberlinkCell>,
    last_used_affiliation: Option<CellAffiliation>,
}

impl Default for NumberlinkBoard {
    fn default() -> Self {
        Self::new((5, 5))
    }
}

impl NumberlinkBoard {
    pub fn new(dims: Location) -> NumberlinkBoard {
        NumberlinkBoard {
            dims,
            // row major
            cells: Array2::from_shape_simple_fn((dims.1, dims.0), NumberlinkCell::default),
            last_used_affiliation: None,
        }
    }

    pub fn add_endpoints(&mut self, locations: UnorderedPair<Location>) {
        let first_avail_affiliation = Some(match self.last_used_affiliation {
            None => 0,
            Some(aff) => aff + 1
        });
        for endpoint in [locations.0, locations.1] {
            self.cells.index_mut(endpoint).assign_elem(NumberlinkCell {
                affiliation: first_avail_affiliation,
                is_terminus: true,
                ..Default::default()
            })
        }
        self.last_used_affiliation = first_avail_affiliation;
    }

    pub fn step(&self, loc: Location, direction: BoardTraverseDirection) -> Option<Location> {
        let new_loc = match direction {
            BoardTraverseDirection::UP => (loc.0 + 0, loc.1 - 1),
            BoardTraverseDirection::DOWN => (loc.0 + 0, loc.1 + 1),
            BoardTraverseDirection::LEFT => (loc.0 - 1, loc.1 + 0),
            BoardTraverseDirection::RIGHT => (loc.0 + 1, loc.1 + 0),
        };

        match (0..self.dims.0).contains(&new_loc.0) && (0..self.dims.1).contains(&new_loc.1) {
            true => Some(new_loc),
            false => None
        }
    }
}

impl Display for NumberlinkBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut ret = String::new();

        for row in self.cells.rows() {
            ret.push_str(&*row.mapv(|cell| cell.to_string()).to_vec().join(""));
            ret.push('\n');
        }
        write!(f, "{}", ret)
    }
}