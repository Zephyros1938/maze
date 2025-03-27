pub struct Moveset {
    pub moves: Vec<u8>,
}

impl Moveset {
    pub fn insert_front(&mut self, i: u8) {
        self.moves.rotate_right(1);
        self.moves[0] = i;
    }

    pub fn insert_back(&mut self, i: u8) {
        let len = self.moves.len() - 1;
        self.moves.rotate_left(1);
        self.moves[len] = i;
    }
}

impl std::fmt::Display for Moveset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match String::from_utf8(self.moves.to_vec()) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            }
        )
    }
}
