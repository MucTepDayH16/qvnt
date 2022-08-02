pub struct BitsIter {
    bits: usize,
    pos: usize,
}

impl From<usize> for BitsIter {
    fn from(bits: usize) -> Self {
        Self { bits, pos: 1 }
    }
}

impl Iterator for BitsIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pos {
            pos if pos > self.bits => None,
            pos if pos & self.bits != 0 => {
                self.pos <<= 1;
                Some(pos)
            }
            _ => {
                self.pos <<= 1;
                self.next()
            }
        }
    }
}
