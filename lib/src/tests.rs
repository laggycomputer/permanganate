#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use varisat::Var;

    use crate::{BoardTraverseDirection, NumberlinkBoard};

    #[test]
    fn construct_basic_board() {
        let mut board = NumberlinkBoard::with_dims((3, 3));
        board.add_termini_with_display('A', ((0, 0), (2, 2)));
        board.add_termini(((0, 1), (2, 1)));
        assert_eq!(board.to_string(), "A..\nB.B\n..A\n")
    }

    #[test]
    fn step_invalid() {
        let board = NumberlinkBoard::with_dims((3, 3));
        assert_eq!(board.step((0, 0), BoardTraverseDirection::UP), None);
    }

    #[test]
    fn step_valid() {
        let board = NumberlinkBoard::with_dims((5, 5));
        assert_eq!(board.step((4, 4), BoardTraverseDirection::LEFT), Some((3, 4)))
    }

    #[test]
    fn num_affiliations() {
        let mut board = NumberlinkBoard::with_dims((3, 5));
        board.add_termini_with_display('A', ((0, 0), (2, 4)));
        board.add_termini_with_display('B', ((0, 1), (2, 3)));
        assert_eq!(board.num_affiliations(), 2)
    }

    #[test]
    fn neighbors_of_corner() {
        let board = NumberlinkBoard::with_dims((3, 3));
        let (neighbor_locs, possible_directions) = board.neighbors_of((0, 0));
        assert_eq!(neighbor_locs.len(), 2);
        assert_eq!(possible_directions, HashSet::from([BoardTraverseDirection::DOWN, BoardTraverseDirection::RIGHT]));
    }

    #[test]
    fn neighbors_of_edge() {
        let board = NumberlinkBoard::with_dims((3, 3));
        assert_eq!(board.neighbors_of((1, 0)).0.len(), 3)
    }

    #[test]
    fn neighbors_of_surrounded() {
        let board = NumberlinkBoard::with_dims((3, 3));
        assert_eq!(board.neighbors_of((1, 1)).0.len(), 4)
    }

    #[test]
    fn affiliation_var() {
        let mut board = NumberlinkBoard::with_dims((3, 5));
        board.add_termini_with_display('A', ((0, 0), (2, 2)));
        board.add_termini_with_display('B', ((0, 1), (0, 2)));
        assert_eq!(board.affiliation_var((2, 4), 1), Var::from_index(29));
    }
}