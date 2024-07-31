#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use crate::builder::{Builder, SquareBoardBuilder};
    use crate::location::Location;

    #[test]
    fn remove_termini() {
        let board = SquareBoardBuilder::with_dims((NonZero::new(5).unwrap(), NonZero::new(5).unwrap()))
            .add_termini('A', (Location(0, 0), Location(1, 4)))
            .pop_termini()
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
        let board = SquareBoardBuilder::with_dims((NonZero::new(5).unwrap(), NonZero::new(5).unwrap()))
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

        let solved = board.solve().unwrap();
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
        let board = SquareBoardBuilder::with_dims((NonZero::new(12).unwrap(), NonZero::new(12).unwrap()))
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

        let solved = board.solve().unwrap();
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

    #[test]
    fn simple_with_bridge() {
        // flow free bridges starter pack 5x5 level 2
        let board = SquareBoardBuilder::with_dims((NonZero::new(5).unwrap(), NonZero::new(5).unwrap()))
            .add_termini('A', (Location(1, 3), Location(3, 0)))
            .add_termini('B', (Location(1, 4), Location(4, 3)))
            .add_termini('C', (Location(0, 0), Location(0, 4)))
            .add_termini('D', (Location(1, 0), Location(2, 2)))
            .add_termini('E', (Location(4, 0), Location(2, 3)))
            .add_bridge(Location(2, 1))
            .build()
            .unwrap();

        assert_eq!(format!("{}", board), "CD.AE
..+..
..D..
.AE.B
CB...
");

        let solved = board.solve().unwrap();
        assert_eq!(format!("{}", solved), "CDdAE
ca+ae
caDee
cAEeB
CBbbb
");
    }

    #[test]
    fn adjacent_bridges() {
        // flow free bridges hashed pack 7x7 level 1
        let board = SquareBoardBuilder::with_dims((NonZero::new(7).unwrap(), NonZero::new(7).unwrap()))
            .add_termini('A', (Location(0, 5), Location(5, 6)))
            .add_termini('B', (Location(0, 0), Location(6, 6)))
            .add_termini('C', (Location(1, 1), Location(6, 1)))
            .add_termini('D', (Location(1, 2), Location(6, 4)))
            .add_termini('E', (Location(1, 5), Location(6, 2)))
            .add_termini('F', (Location(4, 2), Location(4, 5)))
            .add_bridge(Location(2, 3))
            .add_bridge(Location(2, 4))
            .add_bridge(Location(3, 3))
            .add_bridge(Location(3, 4))
            .build()
            .unwrap();

        assert_eq!(format!("{}", board), "B......
.C....C
.D..F.E
..++...
..++..D
AE..F..
.....AB
");

        let solved = board.solve().unwrap();
        assert_eq!(format!("{}", solved), "Bcccccc
bCeeeeC
bDefFeE
bd++ddd
bb++bbD
AEefFbb
aaaaaAB
");
    }

    #[test]
    fn simple_with_warp() {
        // flow free warps starter pack level 2
        let board = SquareBoardBuilder::with_dims((NonZero::new(5).unwrap(), NonZero::new(3).unwrap()))
            .add_termini('A', (Location(0, 0), Location(4, 0)))
            .add_termini('B', (Location(3, 1), Location(4, 2)))
            .add_termini('C', (Location(0, 2), Location(2, 1)))
            .add_termini('D', (Location(1, 1), Location(4, 1)))
            .add_warp(Location(0, 1), None)
            .build()
            .unwrap();

        assert_eq!(format!("{}", board), "A...A
.DCBD
C...B
");

        let solved = board.solve().unwrap();
        assert_eq!(format!("{}", solved), "AaaaA
dDCBD
CccbB
");
    }

    #[test]
    fn warp_with_holes() {
        // flow free warps starter pack level 1
        let board = SquareBoardBuilder::with_dims((NonZero::new(6).unwrap(), NonZero::new(3).unwrap()))
            .add_termini('A', (Location(0, 1), Location(4, 1)))
            .add_termini('B', (Location(1, 0), Location(3, 0)))
            .add_termini('C', (Location(1, 1), Location(3, 1)))
            .add_termini('D', (Location(1, 2), Location(3, 2)))
            .add_warp(Location(0, 1), None)
            .drop_location(Location(0, 0))
            .drop_location(Location(0, 2))
            .drop_location(Location(4, 0))
            .drop_location(Location(5, 0))
            .drop_location(Location(4, 2))
            .drop_location(Location(5, 2))
            .build()
            .unwrap();

        assert_eq!(format!("{}", board), ".B.B..
AC.CA.
.D.D..
");

        let solved = board.solve().unwrap();
        assert_eq!(format!("{}", solved), ".BbB..
ACcCAa
.DdD..
");
    }

}