#![allow(dead_code)]

use std::fmt::{self, Debug, Formatter};

type Block = u32;
const BLOCK_SIZE: usize = 32;
const BLOCK_MASK: usize = (!(0 as Block)) as usize;

pub struct BitArray {
    // TODO: not pub, they're private fields.
    pub inner: Vec<Block>,
    pub len: usize,
}

impl BitArray {
    fn _cap_from_len(len: usize) -> usize {
        if len > 0 {
            (len - 1) / BLOCK_SIZE + 1
        } else {
            0
        }
    }

    #[inline]
    fn _access(index: usize) -> (usize, Block) {
        (index / BLOCK_SIZE, 1 << ((index % BLOCK_SIZE) as Block))
    }

    pub fn zeros(len: usize) -> Self {
        Self { inner: vec![0; Self::_cap_from_len(len)], len }
    }

    pub fn ones(len: usize) -> Self {
        let mut ones = Self { inner: vec![!0; Self::_cap_from_len(len)], len };
        let rem = len % BLOCK_SIZE;
        if rem > 0 {
            *ones.inner.last_mut().unwrap() = (1 << rem) - 1;
        }
        ones
    }

    pub fn reset(&mut self) {
        self.inner.iter_mut().for_each(|x| *x = 0);
    }

    #[inline]
    pub fn negate(&mut self, index: usize) {
        let (block, mask) = Self::_access(index);
        self.inner[block] ^= mask;
    }

    #[inline]
    pub fn set_bool(&mut self, index: usize, val: bool) {
        let (block, mask) = Self::_access(index);
        if val {
            self.inner[block] |= mask;
        } else {
            self.inner[block] &= !mask;
        }
    }

    #[inline]
    pub fn get_masked(&self, index: usize) -> Block {
        let (block, mask) = Self::_access(index);
        self.inner[block] & mask
    }

    #[inline]
    pub fn get_bool(&self, index: usize) -> bool {
        self.get_masked(index) != 0
    }

    #[inline]
    pub fn xor_all(&mut self, other: &Self) {
        assert_eq!(self.len, other.len);
        for (dest, src) in self.inner.iter_mut().zip(other.inner.iter()) {
            *dest ^= *src;
        }
    }

    pub fn true_indices(&self) -> TIndices {
        TIndices::new(&self)
    }
}

impl Debug for BitArray {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.write_str("Bitarray { inner: [")?;
        if !self.inner.is_empty() {
            fmt.write_fmt(format_args!("{:b}", self.inner[0]))?;
        }
        for bin in self.inner[1..].iter() {
            fmt.write_fmt(format_args!(" {:b}", *bin))?;
        }
        fmt.write_fmt(format_args!("], len: {} }}", self.len))
    }
}

pub struct TIndices<'a> {
    barray: &'a BitArray,
    current_blk: usize,
    current_bit: usize,
    buf: Block,
}

impl<'a> TIndices<'a> {
    fn new(barray: &'a BitArray) -> Self {
        if barray.inner.len() > 0 {
            TIndices { barray, current_blk: 0, current_bit: 0, buf: barray.inner[0] }
        } else {
            TIndices { barray, current_blk: 0, current_bit: 0, buf: 0 }
        }
    }
}

impl Iterator for TIndices<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.buf == 0 {
            if self.current_blk < self.barray.inner.len() - 1 {
                self.current_blk += 1;
                self.current_bit = 0;
                self.buf = self.barray.inner[self.current_blk];
                return self.next();
            }
            return None;
        }
        while (self.buf & 1) == 0 {
            self.current_bit += 1;
            self.buf >>= 1;
        }
        self.buf ^= 1;
        Some(self.current_blk * BLOCK_SIZE + self.current_bit)
    }
}

#[cfg(test)]
mod tests {
    use crate::BitArray;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn set_get1() {
        let mut ba = BitArray::zeros(6);

        ba.set_bool(1, true);
        ba.set_bool(2, false);
        ba.negate(3);

        let ans = [false, true, false, true, false, false];
        for i in 0..6 {
            assert_eq!(ans[i], ba.get_bool(i));
        }
    }

    #[test]
    fn set_get2() {
        let mut ba = BitArray::ones(6);

        ba.negate(3);
        ba.set_bool(1, true);
        ba.set_bool(2, false);

        let ans = [true, true, false, false, true, true];
        for i in 0..6 {
            assert_eq!(ans[i], ba.get_bool(i));
        }
    }

    #[test]
    fn set_get3() {
        let mut ba = BitArray::zeros(34);
        ba.negate(31);
        for i in 0..34 {
            assert_eq!(i == 31, ba.get_bool(i));
        }
    }

    #[test]
    fn set_get4() {
        let mut ba = BitArray::zeros(34);
        ba.negate(32);
        for i in 0..34 {
            assert_eq!(i == 32, ba.get_bool(i));
        }
    }

    #[test]
    fn set_get5() {
        let mut ba = BitArray::zeros(34);
        ba.negate(33);
        for i in 0..34 {
            assert_eq!(i == 33, ba.get_bool(i));
        }
    }

    #[test]
    fn set_get6() {
        let mut ba = BitArray::zeros(6);

        ba.set_bool(1, true);
        ba.set_bool(1, false);
        for i in 0..6 {
            assert_eq!(false, ba.get_bool(i));
        }
    }

    #[test]
    fn set_get7() {
        let mut ba = BitArray::zeros(7);

        ba.set_bool(2, false);
        ba.set_bool(2, true);
        for i in 0..7 {
            assert_eq!(i == 2, ba.get_bool(i));
        }
    }

    #[test]
    fn indices1() {
        let mut ba = BitArray::zeros(41);
        ba.negate(0);
        ba.negate(3);
        ba.negate(21);
        ba.negate(31);
        ba.negate(32);
        ba.negate(33);
        let v: Vec<_> = ba.true_indices().collect();
        assert_eq!(v, vec![0, 3, 21, 31, 32, 33]);
    }

    #[test]
    fn indices2() {
        let ba = BitArray::ones(3);
        let v: Vec<_> = ba.true_indices().collect();
        assert_eq!(v, vec![0, 1, 2]);
    }
}
