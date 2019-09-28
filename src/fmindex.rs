use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

use suffix::SuffixTable;
use super::wavelet_matrix::*;

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
pub struct FMIndex {
    wm: WaveletMatrix,
    sampled_sa: Vec<usize>,
    sa_sampling_rate: f64,
}

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

#[wasm_bindgen]
impl FMIndex {
    #[wasm_bindgen(constructor)]
    pub fn new(text: &str) -> Self {
        let sa_sampling_rate = 0.25;

        // SA
        let st = SuffixTable::new(text);
        let bytes = st.text().as_bytes();

        // Sampled SA
        let len = bytes.len() + 1; // +1 for $
        let div = len / ((len as f64 * sa_sampling_rate) as usize);
        let mut sampled_sa = vec![0];
        for i in 1..len {
            if i % div == 0 {
                sampled_sa.push(st.table()[i - 1] as usize);
            }
        }

        // BWT
        let bytes = st.text().as_bytes();
        let mut bwt = vec![bytes[bytes.len() - 1] as u64];
        for i in st.table() {
            let i = *i as usize;
            bwt.push(if i == 0 { 0 } else { bytes[i - 1] as u64 })
        }

        // Wavelet Matrix
        let wm = WaveletMatrix::new(&bwt);

        Self {
            wm,
            sampled_sa,
            sa_sampling_rate,
        }
    }

    pub fn from(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }

    pub fn contains(&self, query: &str) -> bool {
        let range = self.search(query);
        range.start < range.end
    }

    pub fn counts(&self, query: &str) -> usize {
        let range = self.search(query);
        range.end - range.start
    }

    pub fn search(&self, query: &str) -> Range {
        let mut start = 0;
        let mut end = self.wm.len();

        for b in query.bytes().rev() {
            let b = b as u64;
            start = self.wm.rank(start, b) + self.wm.rank_less_than(self.wm.len(), b);
            end = self.wm.rank(end, b) + self.wm.rank_less_than(self.wm.len(), b);
            if start >= end {
                return Range { start: 0, end: 0 };
            }
        }
        Range { start, end }
    }

    pub fn locate(&self, i: usize) -> usize {
        let len = self.wm.len();
        let div = len / ((len as f64 * self.sa_sampling_rate) as usize);
        let mut j = i;
        let mut t = 0;

        while j % div != 0 {
            let c = self.wm.access(j);
            j = self.wm.rank(j, c) + self.wm.rank_less_than(self.wm.len(), c);
            t += 1;
        }

        if self.sampled_sa[j / div] + t >= len {
            self.sampled_sa[j / div] + t - len
        } else {
            self.sampled_sa[j / div] + t
        }
    }

    pub fn previous_string(&self, i: usize, len: usize) -> String {
        let unicode_max_bytes: usize = 6;

        let mut i = i;
        let mut bytes = vec![0u8; len * unicode_max_bytes];
        let mut j = bytes.len() - 1;

        // TODO (seikichi): refactor
        loop {
            let c = self.wm.access(i);
            bytes[j] = c as u8;
            i = self.wm.rank(i, c) + self.wm.rank_less_than(self.wm.len(), c);
            if i == 0 {
                break;
            }
            j -= 1;

            match std::str::from_utf8(&bytes[j + 1..]) {
                Ok(s) if s.chars().count() >= len => { return s.to_string(); }
                _ => {}
            }
        }

        match std::str::from_utf8(&bytes[j + 1..]) {
            Ok(s) => s.to_string(),
            _ => String::new()
        }
    }
}

#[test]
fn test_fm_index() {
    let fmi = FMIndex::new("The quick brown fox was very quick.");

    assert_eq!(fmi.contains("quick"), true);
    assert_eq!(fmi.contains("vary"), false);
}
