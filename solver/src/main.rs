use std::num::NonZero;

use permanganate::common::location::Location;
use permanganate::graph::SquareNumberlinkBoardBuilder;

fn main() {
    let mut board = SquareNumberlinkBoardBuilder::with_dims((NonZero::new(3).unwrap(), NonZero::new(3).unwrap()))
        .add_termini('A', (Location(0, 0), Location(0, 2)))
        .add_termini('B', (Location(1, 0), Location(1, 2)))
        .add_termini('C', (Location(2, 0), Location(2, 2)))
        .build()
        .unwrap();
}
