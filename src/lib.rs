use bit_vec::BitVec;
use fnv::FnvHasher;
use fxhash::FxHasher;
use std::{marker::PhantomData, collections::hash_map::DefaultHasher, hash::Hasher};

pub struct BloomFilter<T> {
    bits: BitVec,
    _phantom: PhantomData<T>,
}

pub struct BloomFilterArgs {
    bits: usize,
}

impl Default for BloomFilterArgs {
    fn default() -> Self {
        Self { bits: 1024 }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BloomFilterContainsResponse {
    No,
    Maybe
}

impl<T: AsRef<[u8]>> BloomFilter<T> {
    /// Creates a new BloomFilter with the default arguments.
    pub fn new() -> Self {
        BloomFilter::with(BloomFilterArgs::default())
    }

    /// Creates a new BloomFilter with the given arguments.
    pub fn with(args: BloomFilterArgs) -> Self {
        Self {
            bits: BitVec::from_elem(args.bits, false),
            _phantom: PhantomData 
        }
    }

    /// Inserts a new value into the BloomFilter.
    pub fn insert(&mut self, value: &T) {
        for idx in self.calculate_hash_indices(value) {
            self.bits.set(idx, true);
        }
    }

    /// Checks if the BloomFilter contains the given value.
    /// Note that this function returns "no" or "maybe" instead of a boolean.
    /// This is because false positives are possible in a bloom filter.
    pub fn contains(&self, value: &T) -> BloomFilterContainsResponse {
        for idx in self.calculate_hash_indices(value) {
            if !self.bits.get(idx).unwrap_or(false) {
                return BloomFilterContainsResponse::No;
            }
        }
        BloomFilterContainsResponse::Maybe
    }

    /// Calculates the K number of hash values for the given value,
    /// and reduce the hash values modulo the size of the bit vector.
    fn calculate_hash_indices(&self, value: &T) -> Vec<usize> {
        let mut fnv = FnvHasher::default();
        let mut fx = FxHasher::default();
        let mut default = DefaultHasher::default();

        fnv.write(value.as_ref());
        fx.write(value.as_ref());
        default.write(value.as_ref());

        let m = self.bits.len() as u64;
        vec![
            (fnv.finish() % m) as usize,
            (fx.finish() % m) as usize,
            (default.finish() % m) as usize,
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::{BloomFilter, BloomFilterContainsResponse};

    #[test]
    fn bloom_filter_does_not_provide_false_negatives() {
        let mut bloom_filter: BloomFilter<String> = BloomFilter::new();
        let keys = vec!["Test 1", "Other Test", "What about this long one?"];
        keys.iter().for_each(|&s| bloom_filter.insert(&s.into()));
        keys.iter().for_each(
            |&s| assert_eq!(
                bloom_filter.contains(&s.into()),
                BloomFilterContainsResponse::Maybe
            )
        );
    }

    #[test]
    fn bloom_filter_empty_provides_no_response() {
        let bloom_filter: BloomFilter<String> = BloomFilter::new();
        let keys = vec!["This key ain't there", "Testing123", "What about this key right here?"];
        keys.iter().for_each(
            |&s| assert_eq!(
                bloom_filter.contains(&s.into()),
                BloomFilterContainsResponse::No
            )
        );
    }
}
