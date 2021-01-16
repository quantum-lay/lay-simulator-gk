use rand_core::RngCore;

#[derive(Debug)]
pub struct RepeatSeqFakeRng {
    cnt: usize,
    seq: Vec<u64>,
}

impl RepeatSeqFakeRng {
    pub fn new(seq: Vec<u64>) -> Self {
        Self { cnt: 0, seq }
    }
}

impl RngCore for RepeatSeqFakeRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        let i = self.cnt % self.seq.len();
        self.cnt += 1;
        self.seq[i]
    }

    fn fill_bytes(&mut self, _: &mut [u8]){
        unimplemented!();
    }

    fn try_fill_bytes(&mut self, _: &mut [u8]) -> Result<(), rand_core::Error> {
        unimplemented!();
    }
}
