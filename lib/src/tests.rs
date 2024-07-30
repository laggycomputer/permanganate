#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use crate::builder::SquareNumberlinkBoardBuilder;
    use crate::location::Location;

    #[test]
    fn remove_termini() {
        let board = SquareNumberlinkBoardBuilder::with_dims((NonZero::new(5).unwrap(), NonZero::new(5).unwrap()))
            .add_termini('A', (Location(0, 0), Location(1, 4)))
            .remove_termini('A')
            .build()
            .unwrap();

        assert_eq!(format!("{}", board), ".....
.....
.....
.....
.....
");
    }

    #[test]
    fn solve_most_basic() {
        // flow free classic pack level 1
        let board = SquareNumberlinkBoardBuilder::with_dims((NonZero::new(5).unwrap(), NonZero::new(5).unwrap()))
            .add_termini('A', (Location(0, 0), Location(1, 4)))
            .add_termini('B', (Location(2, 0), Location(1, 3)))
            .add_termini('C', (Location(2, 1), Location(2, 4)))
            .add_termini('D', (Location(4, 0), Location(3, 3)))
            .add_termini('E', (Location(4, 1), Location(3, 4)))
            .build()
            .unwrap();

        assert_eq!(format!("{}", board), "A.B.D
..C.E
.....
.B.D.
.ACE.
");

        let solved = board.solve();
        assert_eq!(format!("{}", solved), "AbBdD
abCdE
abcde
aBcDe
aACEe
")
    }

    #[test]
    fn solve_large_simple_square() {
        // flow free extreme pack 2 12x12 level 13
        let board = SquareNumberlinkBoardBuilder::with_dims((NonZero::new(12).unwrap(), NonZero::new(12).unwrap()))
            .add_termini('A', (Location(7, 4), Location(4, 11)))
            .add_termini('B', (Location(6, 4), Location(5, 11)))
            .add_termini('C', (Location(6, 6), Location(0, 11)))
            .add_termini('D', (Location(2, 2), Location(7, 3)))
            .add_termini('E', (Location(5, 4), Location(7, 11)))
            .add_termini('F', (Location(7, 2), Location(3, 8)))
            .add_termini('G', (Location(2, 8), Location(5, 10)))
            .build()
            .unwrap();

        assert_eq!(format!("{}", board), "............
............
..D....F....
.......D....
.....EBA....
............
......C.....
............
..GF........
............
.....G......
C...AB.E....
");

        let solved = board.solve();
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