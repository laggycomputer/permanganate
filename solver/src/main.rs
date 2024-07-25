use unordered_pair::UnorderedPair;

use permanganate::NumberlinkBoard;

fn main() {
    let mut board = NumberlinkBoard::with_dims((3, 3));
    board.add_termini(UnorderedPair::from(((0, 0), (0, 2))));
    board.add_termini(UnorderedPair::from(((1, 0), (1, 2))));
    board.add_termini(UnorderedPair::from(((2, 0), (2, 2))));

    println!("{board}");
    let solved = board.solve_bsat();
}
