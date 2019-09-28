use serde::{Serialize, Deserialize};

use super::bit_vector::BitVector;

#[derive(Serialize, Deserialize, Debug)]
pub struct WaveletMatrix {
    layers: Vec<BitVector>,
}

impl WaveletMatrix {
    pub fn new(vals: &Vec<u64>) -> Self {
        let dim = get_dim(&vals);
        let bit_len = get_bit_len(dim);
        let mut zeros: Vec<u64> = vals.clone();
        let mut ones: Vec<u64> = Vec::new();
        let mut layers: Vec<BitVector> = Vec::new();

        for depth in 0..bit_len {
            let mut next_zeros: Vec<u64> = Vec::new();
            let mut next_ones: Vec<u64> = Vec::new();
            let mut bits = BitVector::new(vals.len() as usize);

            let mut i = 0;
            for val in &zeros {
                let bit = get_bit_lsb(*val, bit_len - depth - 1);
                bits.set(i, bit);
                i += 1;
                if bit {
                    next_ones.push(*val);
                } else {
                    next_zeros.push(*val);
                }
            }
            for val in &ones {
                let bit = get_bit_lsb(*val, bit_len - depth - 1);
                bits.set(i, bit);
                i += 1;
                if bit {
                    next_ones.push(*val);
                } else {
                    next_zeros.push(*val);
                }
            }
            bits.build();

            zeros = next_zeros;
            ones = next_ones;
            layers.push(bits);
        }

        Self { layers }
    }

    pub fn len(&self) -> usize {
        self.layers[0].len()
    }

    pub fn rank(&self, pos: usize, c: u64) -> usize {
        self.prefix_rank_op(pos, c, Operator::Equal)
    }

    pub fn rank_less_than(&self, pos: usize, c: u64) -> usize {
        self.prefix_rank_op(pos, c, Operator::LessThan)
    }

    pub fn access(&self, pos: usize) -> u64 {
        let mut c = 0;
        let mut pos = pos;
        for layer in &self.layers {
            let bit = layer.access(pos);
            pos = layer.rank(pos, bit);
            c <<= 1;
            if bit {
                pos += layer.zeros();
                c |= 1;
            }
        }
        c
    }

    #[inline]
    fn prefix_rank_op(&self, pos: usize, val: u64, operator: Operator) -> usize {
        let mut bpos = 0;
        let mut epos = pos;
        let mut rank = 0;
        let bit_len = self.layers.len() as u8;

        for depth in 0..bit_len {
            let rsd = &self.layers[depth as usize];
            let bit = get_bit_msb(val, depth, bit_len);
            if bit {
                if let Operator::LessThan = operator {
                    rank += rsd.rank(epos, false) - rsd.rank(bpos, false);
                }
                bpos = rsd.rank(bpos, bit) + rsd.zeros();
                epos = rsd.rank(epos, bit) + rsd.zeros();
            } else {
                bpos = rsd.rank(bpos, bit);
                epos = rsd.rank(epos, bit);
            }
        }
        match operator {
            Operator::Equal => epos - bpos,
            _ => rank,
        }
    }
}

pub enum Operator {
    Equal,
    LessThan,
}

fn get_dim(vals: &[u64]) -> u64 {
    let mut dim: u64 = 0;
    for val in vals.iter() {
        if *val >= dim {
            dim = *val + 1;
        }
    }
    dim
}

fn get_bit_len(val: u64) -> u8 {
    let mut blen: u8 = 0;
    let mut val = val;
    while val > 0 {
        val >>= 1;
        blen += 1;
    }
    blen
}

fn get_bit_msb(x: u64, pos: u8, blen: u8) -> bool {
    ((x >> (blen - pos - 1)) & 1) == 1
}

fn get_bit_lsb(x: u64, pos: u8) -> bool {
    ((x >> pos) & 1) == 1
}

#[test]
fn test_wavelet_matrix() {
    let vec: Vec<u64> = vec![1, 2, 4, 5, 1, 0, 4, 6, 2, 9, 2, 0];
    //                       0  1  2  3  4  5  6  7  8  9 10 11 (length = 12)
    let wm = WaveletMatrix::new(&vec);

    assert_eq!(wm.len(), 12);
    for i in 0..vec.len() {
        assert_eq!(wm.access(i), vec[i]);
    }

    assert_eq!(wm.rank(wm.len(), 2), 3);
    assert_eq!(wm.rank(wm.len(), 4), 2);
    assert_eq!(wm.rank(wm.len(), 5), 1);
    assert_eq!(wm.rank(wm.len(), 7), 0);
    assert_eq!(wm.rank(wm.len(), 39), 0);

    assert_eq!(wm.rank_less_than(wm.len(), 2), 4);
    assert_eq!(wm.rank_less_than(wm.len(), 7), 11);
}
