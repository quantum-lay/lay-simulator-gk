//! `fakerng` is a fake `[RngCore]` for testing or debugging.
//! This crate provides [RepeatSeqFakeRng], it repeats single sequence.
use rand_core::RngCore;

/// Repeat single sequence.
///
/// # Example
/// ```
/// # use lay_simulator_gk::RepeatSeqFakeRng;
/// # use rand_core::RngCore;
/// let mut rng = RepeatSeqFakeRng::new(vec![1, 2, 3]);
/// assert_eq!(rng.next_u64(), 1);
/// assert_eq!(rng.next_u64(), 2);
/// assert_eq!(rng.next_u64(), 3);
/// assert_eq!(rng.next_u64(), 1);
/// ```
#[derive(Debug)]
pub struct RepeatSeqFakeRng {
    cnt: usize,
    seq: Vec<u64>,
}

impl RepeatSeqFakeRng {
    pub fn new(seq: Vec<u64>) -> Self {
        assert_ne!(seq.len(), 0, "seq.len() > 0 is required.");
        Self { cnt: 0, seq }
    }
}

impl RngCore for RepeatSeqFakeRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        let i = self.cnt;
        self.cnt += 1;
        match self.seq.get(i) {
            Some(x) => *x,
            None => {
                self.cnt = 1;
                self.seq[0]
            }
        }
    }

    fn fill_bytes(&mut self, dest: &mut [u8]){
        let mut ofs = 0;
        while ofs + 8 <= dest.len() {
            let v = self.next_u64();
            dest[ofs] = (v & 0xff) as u8;
            dest[ofs + 1] = ((v & 0xff00) >> 8) as u8;
            dest[ofs + 2] = ((v & 0xff0000) >> 16) as u8;
            dest[ofs + 3] = ((v & 0xff000000) >> 24) as u8;
            dest[ofs + 4] = ((v & 0xff00000000) >> 32) as u8;
            dest[ofs + 5] = ((v & 0xff0000000000) >> 40) as u8;
            dest[ofs + 6] = ((v & 0xff000000000000) >> 48) as u8;
            dest[ofs + 7] = ((v & 0xff00000000000000) >> 56) as u8;
            ofs += 8;
        }
        let mut v = self.next_u64();
        while ofs < dest.len() {
            dest[ofs] = (v & 0xff) as u8;
            v = v >> 8;
            ofs += 1;
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        Ok(self.fill_bytes(dest))
    }
}

#[cfg(test)]
mod tests {
    use crate::RepeatSeqFakeRng;
    use rand_core::RngCore;

    #[test]
    fn test_u64() {
        let mut rng = RepeatSeqFakeRng::new(vec![100, 200, 300]);

        assert_eq!(rng.next_u64(), 100);
        assert_eq!(rng.next_u64(), 200);
        assert_eq!(rng.next_u64(), 300);
        assert_eq!(rng.next_u64(), 100);
        assert_eq!(rng.next_u64(), 200);
        assert_eq!(rng.next_u64(), 300);
        assert_eq!(rng.next_u64(), 100);
    }

    #[test]
    fn test_u32() {
        let mut rng = RepeatSeqFakeRng::new(vec![(100 << 32) | 5, (200 << 32) | 6, (300 << 32) | 7]);

        assert_eq!(rng.next_u32(), 5);
        assert_eq!(rng.next_u32(), 6);
        assert_eq!(rng.next_u32(), 7);
    }

    #[test]
    fn test_len1() {
        let mut rng = RepeatSeqFakeRng::new(vec![1]);
        assert_eq!(rng.next_u64(), 1);
        assert_eq!(rng.next_u32(), 1);
        assert_eq!(rng.next_u64(), 1);
    }

    #[test]
    fn fill_bytes_justsize() {
        let mut rng = RepeatSeqFakeRng::new(vec![0x4142434445464748, 0x494a4b4c4d4e4f50]);
        let mut vec = vec![0; 16];

        rng.fill_bytes(vec.as_mut());
        assert_eq!(vec.as_slice(), b"HGFEDCBAPONMLKJI");
    }

    #[test]
    fn fill_bytes_repeated() {
        let mut rng = RepeatSeqFakeRng::new(vec![0x4142434445464748, 0x494a4b4c4d4e4f50]);
        let mut vec = vec![0; 16];

        let _ = rng.next_u64();
        rng.fill_bytes(vec.as_mut());
        assert_eq!(vec.as_slice(), b"PONMLKJIHGFEDCBA");
    }

    #[test]
    fn fill_bytes_smallsize() {
        let mut rng = RepeatSeqFakeRng::new(vec![0x4142434445464748, 0x494a4b4c4d4e4f50, 0xdeadbeef]);
        let mut vec = vec![0; 3];

        let _ = rng.next_u64();
        rng.fill_bytes(vec.as_mut());
        assert_eq!(vec.as_slice(), b"PON");
        assert_eq!(rng.next_u64(), 0xdeadbeef);
    }
}
