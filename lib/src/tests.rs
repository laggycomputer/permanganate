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

    #[test]
    fn solve_board() {
        {
            // flow free classic pack level 1
            let mut board = NumberlinkBoard::with_dims((5, 5));
            board.add_termini_with_display('A', ((0, 0), (1, 4)));
            board.add_termini_with_display('B', ((2, 0), (1, 3)));
            board.add_termini_with_display('C', ((2, 1), (2, 4)));
            board.add_termini_with_display('D', ((4, 0), (3, 3)));
            board.add_termini_with_display('E', ((4, 1), (3, 4)));

            let solved = board.solve_bsat().unwrap();
            assert_eq!(format!("{}", solved), "AbBdD
abCdE
abcde
aBcDe
aACEe
");
        }
        {
            // flow free extreme pack 2 12x12 level 13
            let mut board = NumberlinkBoard::with_dims((12, 12));
            board.add_termini_with_display('A', ((7, 4), (4, 11)));
            board.add_termini_with_display('B', ((6, 4), (5, 11)));
            board.add_termini_with_display('C', ((6, 6), (0, 11)));
            board.add_termini_with_display('D', ((2, 2), (7, 3)));
            board.add_termini_with_display('E', ((5, 4), (7, 11)));
            board.add_termini_with_display('F', ((7, 2), (3, 8)));
            board.add_termini_with_display('G', ((2, 8), (5, 10)));
            let solved = board.solve_bsat().unwrap();
            assert_eq!(format!("{}", solved), "ccccceeeeeee
caaacebbbbbe
caDacebFffbe
cadacebDdfbe
cadacEBAdfbe
cadacccadfbe
cadaaaCadfbe
cadddaaadfbe
caGFdddddfbe
cagfffffffbe
cagggGbbbbbe
CaaaABbEeeee
")
        }
    }
}