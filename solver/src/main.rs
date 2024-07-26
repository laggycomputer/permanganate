use permanganate::NumberlinkBoard;

fn main() {
    let mut board = NumberlinkBoard::with_dims((5, 5));
    board.add_termini_with_display('A', ((0, 0), (1, 4)));
    board.add_termini_with_display('B', ((2, 0), (1, 3)));
    board.add_termini_with_display('C', ((2, 1), (2, 4)));
    board.add_termini_with_display('D', ((4, 0), (3, 3)));
    board.add_termini_with_display('E', ((4, 1), (3, 4)));

    println!("{board}");
    let solved = board.solve_bsat().unwrap();
    println!("{solved}");
}
