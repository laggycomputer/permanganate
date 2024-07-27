#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use varisat::Var;

    use crate::basic::{BoardTraverseDirection, SimpleNumberlinkBoard};
    use crate::common::Location;

    #[test]
    fn construct_basic_board() {
        let mut board = SimpleNumberlinkBoard::with_dims((3, 3)).unwrap();
        board.add_termini_with_display('A', (Location(0, 0), Location(2, 2)));
        board.add_termini((Location(0, 1), Location(2, 1)));
        assert_eq!(board.to_string(), "A..\nB.B\n..A\n")
    }

    #[test]
    fn step_invalid() {
        let board = SimpleNumberlinkBoard::with_dims((3, 3)).unwrap();
        assert_eq!(board.step(Location(0, 0), BoardTraverseDirection::UP), None);
    }

    #[test]
    fn step_valid() {
        let board = SimpleNumberlinkBoard::with_dims((5, 5)).unwrap();
        assert_eq!(board.step(Location(4, 4), BoardTraverseDirection::LEFT), Some(Location(3, 4)))
    }

    #[test]
    fn num_affiliations() {
        let mut board = SimpleNumberlinkBoard::with_dims((3, 5)).unwrap();
        board.add_termini_with_display('A', (Location(0, 0), Location(2, 4)));
        board.add_termini_with_display('B', (Location(0, 1), Location(2, 3)));
        assert_eq!(board.num_affiliations(), 2)
    }

    #[test]
    fn neighbors_of_corner() {
        let board = SimpleNumberlinkBoard::with_dims((3, 3)).unwrap();
        let (neighbor_locs, possible_directions) = board.neighbors_of(Location(0, 0));
        assert_eq!(neighbor_locs.len(), 2);
        assert_eq!(possible_directions, HashSet::from([BoardTraverseDirection::DOWN, BoardTraverseDirection::RIGHT]));
    }

    #[test]
    fn neighbors_of_edge() {
        let board = SimpleNumberlinkBoard::with_dims((3, 3)).unwrap();
        assert_eq!(board.neighbors_of(Location(1, 0)).0.len(), 3)
    }

    #[test]
    fn neighbors_of_surrounded() {
        let board = SimpleNumberlinkBoard::with_dims((3, 3)).unwrap();
        assert_eq!(board.neighbors_of(Location(1, 1)).0.len(), 4)
    }

    #[test]
    fn affiliation_var() {
        let mut board = SimpleNumberlinkBoard::with_dims((3, 5)).unwrap();
        board.add_termini_with_display('A', (Location(0, 0), Location(2, 2)));
        board.add_termini_with_display('B', (Location(0, 1), Location(0, 2)));
        assert_eq!(board.affiliation_var(Location(2, 4), 1), Var::from_index(29));
    }

    #[test]
    fn solve_board() {
        {
            // flow free classic pack level 1
            let mut board = SimpleNumberlinkBoard::with_dims((5, 5)).unwrap();
            board.add_termini_with_display('A', (Location(0, 0), Location(1, 4)));
            board.add_termini_with_display('B', (Location(2, 0), Location(1, 3)));
            board.add_termini_with_display('C', (Location(2, 1), Location(2, 4)));
            board.add_termini_with_display('D', (Location(4, 0), Location(3, 3)));
            board.add_termini_with_display('E', (Location(4, 1), Location(3, 4)));

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
            let mut board = SimpleNumberlinkBoard::with_dims((12, 12)).unwrap();
            board.add_termini_with_display('A', (Location(7, 4), Location(4, 11)));
            board.add_termini_with_display('B', (Location(6, 4), Location(5, 11)));
            board.add_termini_with_display('C', (Location(6, 6), Location(0, 11)));
            board.add_termini_with_display('D', (Location(2, 2), Location(7, 3)));
            board.add_termini_with_display('E', (Location(5, 4), Location(7, 11)));
            board.add_termini_with_display('F', (Location(7, 2), Location(3, 8)));
            board.add_termini_with_display('G', (Location(2, 8), Location(5, 10)));
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