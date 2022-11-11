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
                let pos = self.pos;
                self.pos <<= 1;
                return Some(pos);
            } else if self.pos > self.bits {
                return None;
            }
            self.pos <<= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_iter() {
        let number = 0b10011001101010;
        let mut iter = BitsIter::from(number);

        assert_eq!(iter.next(), Some(1 << 1));
        assert_eq!(iter.next(), Some(1 << 3));
        assert_eq!(iter.next(), Some(1 << 5));
        assert_eq!(iter.next(), Some(1 << 6));
        assert_eq!(iter.next(), Some(1 << 9));
        assert_eq!(iter.next(), Some(1 << 10));
        assert_eq!(iter.next(), Some(1 << 13));
        assert_eq!(iter.next(), None);
    }
}
