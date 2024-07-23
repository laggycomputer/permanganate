#[cfg(test)]
mod tests {
    use unordered_pair::UnorderedPair;

    use crate::{BoardTraverseDirection, NumberlinkBoard};

    #[test]
    fn construct_basic_board() {
        let mut board = NumberlinkBoard::with_dims((3, 3));
        board.add_endpoints(UnorderedPair::from(((0, 0), (2, 2)))).unwrap();
        board.add_endpoints_with_affiliation(1, UnorderedPair::from(((0, 1), (0, 3)))).unwrap();
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
}