use unordered_pair::UnorderedPair;
use permanganate::NumberlinkBoard;

fn main() {
    let mut b = NumberlinkBoard::new((5, 5));
    b.add_endpoints(UnorderedPair::from(((0, 0), (4, 4))));

    println!("{b}");
}
