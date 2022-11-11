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
        loop {
            if self.pos & self.bits != 0 {
                return Some(self.pos);
            } else if self.pos > self.bits {
                return None;
            }
            self.pos <<= 1;
        }
    }
}
