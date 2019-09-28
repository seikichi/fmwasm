use serde::{Serialize, Deserialize};

const BLOCK_BITS: usize = 16;
const LEVEL_L: usize = 512;
const LEVEL_S: usize = 16;

#[derive(Serialize, Deserialize, Debug)]
pub struct BitVector {
    size: usize,
    large: Vec<u64>,
    small: Vec<u16>,
    bits: Vec<u16>,
}

impl BitVector {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            large: vec![0; size / LEVEL_L + 1],
            small: vec![0; size / LEVEL_S + 1],
            bits: vec![0; (size + BLOCK_BITS - 1) / BLOCK_BITS + 1],
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn zeros(&self) -> usize {
        self.rank(self.len(), false)
    }

    pub fn set(&mut self, pos: usize, bit: bool) {
        let block_pos = pos / BLOCK_BITS;
        let offset   = pos % BLOCK_BITS;
        if bit {
            self.bits[block_pos] |= 1u16 << offset;
        } else {
            self.bits[block_pos] &= !(1u16 << offset);
        }
    }

    pub fn access(&self, pos: usize) -> bool {
        let block_pos = pos / BLOCK_BITS;
        let offset   = pos % BLOCK_BITS;
        ((self.bits[block_pos] >> offset) & 1) == 1
    }

    // # of ${bit} in [0, pos)
    pub fn rank(&self, pos: usize, bit: bool) -> usize {
        if bit {
            let l = self.large[pos / LEVEL_L] as usize;
            let s = self.small[pos / LEVEL_S] as usize;
            let b = (self.bits[pos / BLOCK_BITS] & ((1 << (pos % BLOCK_BITS)) - 1)).count_ones() as usize;
            l + s + b
        } else {
            pos - self.rank(pos, true)
        }
    }

    pub fn build(&mut self) {
        let mut ones = 0;
        for i in 0..=self.size {
            if i % LEVEL_L == 0 {
                self.large[i / LEVEL_L] = ones as u64;
            }
            if i % LEVEL_S == 0 {
                self.small[i / LEVEL_S] = ((ones as u64) - self.large[i / LEVEL_L]) as u16;
            }
            if i != self.size && i % BLOCK_BITS == 0 {
                ones += self.bits[i / BLOCK_BITS].count_ones() as usize;
            }
        }
    }
}

#[test]
fn test_bit_vector_small() {
    let mut bits = BitVector::new(8);
    bits.set(0, true);
    bits.set(1, true);
    bits.set(4, true);
    bits.set(5, true);
    bits.set(6, true);
    bits.build();

    // 1, 1, 0, 0, 1, 1, 1, 0
    assert_eq!(bits.access(0), true);
    assert_eq!(bits.rank(5, true), 3);
    assert_eq!(bits.rank(5, false), 2);
    assert_eq!(bits.rank(8, true), 5);
}

#[test]
fn test_bit_vector_large() {
    let mut bits = BitVector::new(1_000_000);
    for i in 0..1_000_000 {
        bits.set(i, true);
    }
    bits.build();

    assert_eq!(bits.access(0), true);
    assert_eq!(bits.access(999_999), true);

    assert_eq!(bits.rank(0, true), 0);
    assert_eq!(bits.rank(999_999, true), 999_999);

    assert_eq!(bits.rank(0, false), 0);
    assert_eq!(bits.rank(999_999, false), 0);

    assert_eq!(bits.zeros(), 0);
}
