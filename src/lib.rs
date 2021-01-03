use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use lay::Operations;
use lay::gates::CliffordGate;

mod bitarray;
use bitarray::BitArray;

mod fakerng;
pub use fakerng::RepeatSeqFakeRng;

pub type Qubit = u32;
pub type DefaultRng = XorShiftRng;

pub struct GottesmanKnillSimulator<Rng> {
    xs: Vec<BitArray>,
    zs: Vec<BitArray>,
    sgns: BitArray,
    // TODO: Not pub.
    pub measured: BitArray,
    rng: Rng,
}

impl GottesmanKnillSimulator<DefaultRng> {
    pub fn from_seed(n: Qubit, seed: u64) -> Self {
        Self::from_rng(n, DefaultRng::seed_from_u64(seed))
    }
}
impl<Rng: RngCore> GottesmanKnillSimulator<Rng> {
    pub fn from_rng(n: Qubit, rng: Rng) -> Self {
        let xs = (0..n).map(|_| BitArray::zeros(n as usize)).collect();
        let zs = (0..n).map(|i| {
            let mut arr = BitArray::zeros(n as usize);
            arr.negate(i as usize);
            arr
        }).collect();
        let sgns = BitArray::zeros(n as usize);
        let measured = BitArray::zeros(n as usize);
        Self { xs, zs, sgns, measured, rng }
    }
}

impl<Rng> GottesmanKnillSimulator<Rng> {
    pub fn dump_print(&self) {
        println!("xs:   {:?}", self.xs);
        println!("zs:   {:?}", self.zs);
        println!("sgns: {:?}", self.sgns);
        println!("measured: {:?}", self.measured);
    }
    pub fn n_qubits(&self) -> Qubit {
        self.xs.len() as _
    }
}

impl<Rng: RngCore> Operations for GottesmanKnillSimulator<Rng> {
    type Qubit = Qubit;
    type Slot = Qubit;
    fn initialize(&mut self) {
        self.xs.iter_mut().for_each(|a| a.reset());
        self.zs.iter_mut().for_each(|a| a.reset());
        self.zs.iter_mut().enumerate().for_each(|(i, a)| a.negate(i as usize));
        self.sgns.reset();
        self.measured.reset();
    }
    fn measure(&mut self, q: Qubit, ch: Self::Slot) {
        let bit = measure(self, q);
        self.measured.set_bool(ch as usize, bit);
    }
}

impl<Rng: RngCore> CliffordGate for GottesmanKnillSimulator<Rng> {
    #[inline]
    fn x(&mut self, q: Qubit) {
        for (i, _) in self.zs.iter().enumerate()
                                    .filter(|(_, zs)| zs.get_bool(q as usize)) {
            self.sgns.negate(i as usize);
        }
    }

    #[inline]
    fn y(&mut self, q: Qubit) {
        for (i, _) in  self.xs.iter().zip(self.zs.iter())
                           .enumerate()
                           .filter(|(_, (xs, zs))| (xs.get_masked(q as usize) ^ zs.get_masked(q as usize)) != 0) {
            self.sgns.negate(i as usize);
         }
    }

    #[inline]
    fn z(&mut self, q: Qubit) {
        for (i, _) in self.xs.iter().enumerate()
                                    .filter(|(_, xs)| xs.get_bool(q as usize)) {
            self.sgns.negate(i as usize);
        }
    }

    #[inline]
    fn h(&mut self, q: Qubit) {
        for (i, (xs, zs)) in self.xs.iter_mut().zip(self.zs.iter_mut()).enumerate() {
            let x = xs.get_bool(q as usize);
            let z = zs.get_bool(q as usize);
            if x && z {
                self.sgns.negate(i);
            } else if x || z {
                xs.negate(q as usize);
                zs.negate(q as usize);
            }
         }
    }

    #[inline]
    fn s(&mut self, q: Qubit) {
        for (i, (xs, zs)) in self.xs.iter().zip(self.zs.iter_mut())
                                           .enumerate() {
            if xs.get_bool(q as usize) {
                if zs.get_bool(q as usize) {
                    self.sgns.negate(i as usize);
                }
                zs.negate(q as usize);
            }
         }
    }

    #[inline]
    fn sdg(&mut self, q: Qubit) {
        for (i, (xs, zs)) in self.xs.iter().zip(self.zs.iter_mut())
                                           .enumerate() {
            if xs.get_bool(q as usize) {
                if !zs.get_bool(q as usize) {
                    self.sgns.negate(i as usize);
                }
                zs.negate(q as usize);
            }
         }
    }

    #[inline]
    fn cx(&mut self, c: Qubit, t: Qubit) {
        for (i, (xs, zs)) in self.xs.iter_mut()
                                 .zip(self.zs.iter_mut())
                                 .enumerate() {
            if xs.get_bool(c as usize) {
                xs.negate(t as usize);
                if zs.get_bool(c as usize) {
                    self.sgns.negate(i as usize);
                }
            }
            if zs.get_bool(t as usize) {
                zs.negate(c as usize);
            }
        }
    }
}

fn mult_to<Rng>(gk: &mut GottesmanKnillSimulator<Rng>, dest: usize, src: usize) {
    assert_ne!(dest, src);
    let from = unsafe { &*(&gk.xs[src] as *const _) };
    let into = &mut gk.xs[dest];
    into.xor_all(&*from);
    let from = unsafe { &*(&gk.zs[src] as *const _) };
    let into = &mut gk.zs[dest];
    into.xor_all(&*from);
    gk.sgns.set_bool(dest, gk.sgns.get_bool(src));
}

fn measure<Rng: RngCore>(gk: &mut GottesmanKnillSimulator<Rng>, q: Qubit) -> bool {
    let noncommutatives: Vec<_> = gk.xs.iter().map(|a| a.get_bool(q as usize))
                                              .enumerate()
                                              .filter(|(_, b)| *b)
                                              .map(|(i, _)| i)
                                              .collect();
    if noncommutatives.is_empty() {
        //eprintln!("stabilized pattern");
        let n_qubits = gk.n_qubits() as usize;
        let mut indices: Vec<_> = (0..n_qubits).collect();
        for i in 0..n_qubits as usize {
            let x_inds: Vec<_> = indices.iter().enumerate().filter(|(_, &k)| gk.xs[k].get_bool(i)).map(|(i, _)| i).collect();
            if !x_inds.is_empty() {
                let xs0 = unsafe { &*(&gk.xs[indices[x_inds[0]]] as *const _) };
                let zs0 = unsafe { &*(&gk.zs[indices[x_inds[0]]] as *const _) };
                let sg0 = gk.sgns.get_bool(indices[x_inds[0]]);
                for j in x_inds[1..].iter() {
                    gk.xs[indices[*j]].xor_all(&xs0);
                    gk.zs[indices[*j]].xor_all(&zs0);
                    if sg0 {
                        gk.sgns.negate(indices[*j]);
                    }
                }
                indices.swap_remove(x_inds[0]);
            }
        }
        for i in 0..n_qubits as usize {
            if i == q as usize { continue }
            let z_inds: Vec<_> = indices.iter().enumerate().filter(|(_, &k)| gk.zs[k].get_bool(i)).map(|(i, _)| i).collect();
            if !z_inds.is_empty() {
                let xs0 = unsafe { &*(&gk.xs[indices[z_inds[0]]] as *const _) };
                let zs0 = unsafe { &*(&gk.zs[indices[z_inds[0]]] as *const _) };
                let sg0 = gk.sgns.get_bool(indices[z_inds[0]]);
                for j in z_inds[1..].iter() {
                    gk.xs[indices[*j]].xor_all(&xs0);
                    gk.zs[indices[*j]].xor_all(&zs0);
                    if sg0 {
                        gk.sgns.negate(indices[*j]);
                    }
                }
                indices.swap_remove(z_inds[0]);
            }
        }
        assert_eq!(indices.len(), 1);
        // println!("measured xs: {:?}", gk.xs[indices[0]]);
        // println!("measured zs: {:?}", gk.zs[indices[0]]);
        // println!("measured sg: {:?}", gk.sgns.get_bool(indices[0]));
        gk.sgns.get_bool(indices[0])
    } else {
        //eprintln!("non-stabilized pattern");
        let i = noncommutatives[0];
        for &j in noncommutatives[1..].iter() {
            mult_to(gk, j, i);
        }
        let is_one = (gk.rng.next_u32() & 1) != 0;
        gk.xs[noncommutatives[0]].reset();
        gk.zs[noncommutatives[0]].reset();
        gk.zs[noncommutatives[0]].negate(q as usize);
        gk.sgns.set_bool(noncommutatives[0], is_one);
        is_one
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::{GottesmanKnillSimulator, Qubit, DefaultRng, RepeatSeqFakeRng};
    #[allow(unused_imports)]
    use rand_core::{RngCore, SeedableRng};
    #[allow(unused_imports)]
    use rand_xorshift::XorShiftRng;
    #[allow(unused_imports)]
    use lay::Operations;
    #[allow(unused_imports)]
    use lay::gates::CliffordGate;
    #[allow(unused_imports)]
    use lay_simulator_blueqat::{BlueqatSimulator, BlueqatOperations};
    #[allow(unused_imports)]
    use tokio::{prelude::*, runtime::Runtime};


    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let _ = GottesmanKnillSimulator::from_seed(3, 0);
    }

    fn check(ops: impl Fn(&mut GottesmanKnillSimulator<DefaultRng>), expect: &[u32]) {
        let mut gk = GottesmanKnillSimulator::from_seed(expect.len() as Qubit, 0);
        ops(&mut gk);
        let actual: Vec<_> = (0..expect.len()).map(|i| gk.measured.get_bool(i) as u32).collect();
        assert_eq!(actual.as_slice(), expect);
    }

    fn check_with_randseq(ops: impl Fn(&mut GottesmanKnillSimulator<RepeatSeqFakeRng>), expect: &[u32], seq: Vec<u64>) {
        let mut gk = GottesmanKnillSimulator::from_rng(expect.len() as Qubit, RepeatSeqFakeRng::new(seq));
        ops(&mut gk);
        let actual: Vec<_> = (0..expect.len()).map(|i| gk.measured.get_bool(i) as u32).collect();
        assert_eq!(actual.as_slice(), expect);
    }

    fn check_stabilized(gk: &GottesmanKnillSimulator<DefaultRng>, bq: &BlueqatOperations) {
        let rt = Runtime::new().unwrap();
        let mut bqsim = BlueqatSimulator::new().unwrap();

        // TODO: Implement
    }

    #[test]
    fn test_zgate1() {
        check(|gk| {
            gk.z(0);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[0]);
    }

    #[test]
    fn test_xgate1() {
        check(|gk| {
            gk.x(0);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[1]);
    }

    #[test]
    fn test_xgate2() {
        check(|gk| {
            gk.x(0);
            gk.x(3);
            gk.z(2);
            gk.x(6);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[1, 0, 0, 1, 0, 0, 1]);
    }

    #[test]
    fn test_cx1() {
        check(|gk| {
            gk.cx(0, 1);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[0, 0]);
    }

    #[test]
    fn test_cx2() {
        check(|gk| {
            gk.x(1);
            gk.cx(0, 1);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[0, 1]);
    }

    #[test]
    fn test_cx3() {
        check(|gk| {
            gk.x(0);
            gk.cx(0, 1);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[1, 1]);
    }

    #[test]
    fn test_cx4() {
        check(|gk| {
            gk.x(0);
            gk.cx(0, 1);
            gk.cx(1, 2);
            gk.cx(2, 0);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[0, 1, 1]);
    }

    #[test]
    fn test_h_and_z() {
        check(|gk| {
            gk.h(0);
            gk.z(0);
            gk.h(0);
            gk.x(1);
            gk.h(1);
            gk.h(1);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[1, 1]);
    }

    #[test]
    fn test_h_and_s() {
        check(|gk| {
            gk.h(0);
            gk.s(0);
            gk.s(0);
            gk.s(0);
            gk.s(0);
            gk.h(0);
            gk.h(1);
            gk.sdg(1);
            gk.sdg(1);
            gk.h(1);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[0, 1]);
    }

    #[test]
    fn test_h_and_x() {
        check(|gk| {
            gk.h(0);
            gk.s(0);
            gk.h(0);
            gk.x(0);
            gk.h(0);
            gk.sdg(0);
            gk.h(0);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[1]);
    }

    #[test]
    fn test_hh() {
        check_with_randseq(|gk| {
            gk.h(0);
            gk.cx(0, 1);
            gk.h(2);
            gk.cx(2, 3);
            for i in 0..gk.n_qubits() {
                gk.measure(i, i);
            }
        }, &[1, 1, 0, 0], vec![1, 0, 0, 0]);
    }

    #[test]
    fn test_rand_except_cnot() {
        let mut rng = XorShiftRng::seed_from_u64(123);
        let mut gk = GottesmanKnillSimulator::from_seed(2, 0);
        let mut bq = BlueqatOperations::new();
        for _ in 0..10 {
            match rng.next_u32() % 6 {
                0 => {
                    bq.h(0);
                    gk.h(0);
                },
                1 => {
                    bq.x(0);
                    gk.x(0);
                },
                2 => {
                    bq.y(0);
                    gk.y(0);
                },
                3 => {
                    bq.z(0);
                    gk.z(0);
                },
                4 => {
                    bq.s(0);
                    gk.s(0);
                },
                5 => {
                    bq.sdg(0);
                    gk.sdg(0);
                },
                _ => unreachable!()
            }
        }
        for _ in 0..10 {
            match rng.next_u32() % 6 {
                0 => {
                    bq.h(1);
                    gk.h(1);
                },
                1 => {
                    bq.x(1);
                    gk.x(1);
                },
                2 => {
                    bq.y(1);
                    gk.y(1);
                },
                3 => {
                    bq.z(1);
                    gk.z(1);
                },
                4 => {
                    bq.s(1);
                    gk.s(1);
                },
                5 => {
                    bq.sdg(1);
                    gk.sdg(1);
                },
                _ => unreachable!()
            }
        }
        check_stabilized(&gk, &bq);
    }
}
