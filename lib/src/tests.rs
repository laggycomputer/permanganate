#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::num::NonZero;

    use varisat::Var;

    use crate::basic::SimpleNumberlinkBoard;
    use crate::common::location::Location;
    use crate::common::shape::SquareStep;
    use crate::graph::SquareNumberlinkBoardBuilder;

    #[test]
    fn construct_basic_board() {
        let mut board = SimpleNumberlinkBoard::with_dims((NonZero::new(3).unwrap(), NonZero::new(3).unwrap())).unwrap();
        board.add_termini_with_display('A', (Location(0, 0), Location(2, 2)));
        board.add_termini((Location(0, 1), Location(2, 1)));
        assert_eq!(board.to_string(), "A..\nB.B\n..A\n")
    }

    #[test]
    fn step_invalid() {
        let board = SimpleNumberlinkBoard::with_dims((NonZero::new(3).unwrap(), NonZero::new(3).unwrap())).unwrap();
        assert_eq!(board.step(Location(0, 0), SquareStep::UP), None);
    }

    #[test]
    fn step_valid() {
        let board = SimpleNumberlinkBoard::with_dims((NonZero::new(5).unwrap(), NonZero::new(5).unwrap())).unwrap();
        assert_eq!(board.step(Location(4, 4), SquareStep::LEFT), Some(Location(3, 4)))
    }

    #[test]
    fn num_affiliations() {
        let mut board = SimpleNumberlinkBoard::with_dims((NonZero::new(3).unwrap(), NonZero::new(5).unwrap())).unwrap();
        board.add_termini_with_display('A', (Location(0, 0), Location(2, 4)));
        board.add_termini_with_display('B', (Location(0, 1), Location(2, 3)));
        assert_eq!(board.num_affiliations(), 2)
    }

    #[test]
    fn neighbors_of_corner() {
        let board = SimpleNumberlinkBoard::with_dims((NonZero::new(3).unwrap(), NonZero::new(3).unwrap())).unwrap();
        let (neighbor_locs, possible_directions) = board.neighbors_of(Location(0, 0));
        assert_eq!(neighbor_locs.len(), 2);
        assert_eq!(possible_directions, HashSet::from([SquareStep::DOWN, SquareStep::RIGHT]));
    }

    #[test]
    fn neighbors_of_edge() {
        let board = SimpleNumberlinkBoard::with_dims((NonZero::new(3).unwrap(), NonZero::new(3).unwrap())).unwrap();
        assert_eq!(board.neighbors_of(Location(1, 0)).0.len(), 3)
    }

    #[test]
    fn neighbors_of_surrounded() {
        let board = SimpleNumberlinkBoard::with_dims((NonZero::new(3).unwrap(), NonZero::new(3).unwrap())).unwrap();
        assert_eq!(board.neighbors_of(Location(1, 1)).0.len(), 4)
    }

    #[test]
    fn affiliation_var() {
        let mut board = SimpleNumberlinkBoard::with_dims((NonZero::new(3).unwrap(), NonZero::new(5).unwrap())).unwrap();
        board.add_termini_with_display('A', (Location(0, 0), Location(2, 2)));
        board.add_termini_with_display('B', (Location(0, 1), Location(0, 2)));
        assert_eq!(board.affiliation_var(Location(2, 4), 1), Var::from_index(29));
    }

    #[test]
    fn solve_simple() {
        {
            // flow free classic pack level 1
            let mut board = SimpleNumberlinkBoard::with_dims((NonZero::new(5).unwrap(), NonZero::new(5).unwrap())).unwrap();
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
            let mut board = SimpleNumberlinkBoard::with_dims((NonZero::new(12).unwrap(), NonZero::new(12).unwrap())).unwrap();
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

    #[test]
    fn solve_graph_simple_square() {
        // flow free extreme pack 2 12x12 level 13
        let mut board = SquareNumberlinkBoardBuilder::with_dims((NonZero::new(12).unwrap(), NonZero::new(12).unwrap()))
            .add_termini('A', (Location(7, 4), Location(4, 11)))
            .add_termini('B', (Location(6, 4), Location(5, 11)))
            .add_termini('C', (Location(6, 6), Location(0, 11)))
            .add_termini('D', (Location(2, 2), Location(7, 3)))
            .add_termini('E', (Location(5, 4), Location(7, 11)))
            .add_termini('F', (Location(7, 2), Location(3, 8)))
            .add_termini('G', (Location(2, 8), Location(5, 10)))
            .build()
            .unwrap();

        println!("{}", board);

        let solved = board.solve();
        println!("{}", solved);
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