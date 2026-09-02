// ---------------- [ File: random-constructible/src/sample.rs ]
crate::ix!();

pub fn sample_variants_with_probabilities<K: Clone + Eq + Hash + Sized, RNG: RngCore + ?Sized>(
    rng: &mut RNG,
    probs: &HashMap<K,f64>
) -> K 
{
    let variants: Vec<_> = probs.keys().cloned().collect();
    let weights:  Vec<_> = variants.iter().map(|v| probs[v]).collect();

    // WeightedIndex is still from rand, but it doesn't require R: rand::Rng,
    // it just needs an RNG that can generate uniform u64. We'll do that manually:
    let mut sum = 0.0;
    for w in &weights {
        sum += w;
    }
    // you could call WeightedIndex::new if sum>0
    let wdist = WeightedIndex::new(&weights).unwrap();

    // WeightedIndex::sample() needs a rand::Rng.  We'll implement a small adapter.
    use rand_core::RngCore;
    let mut adapter = RandCoreAdapter(rng);

    variants[wdist.sample(&mut adapter)].clone()
}

struct RandCoreAdapter<'a,RC: RngCore + ?Sized>(&'a mut RC);

impl<'a,RC: RngCore + ?Sized> Rng for RandCoreAdapter<'a,RC> {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.0.try_fill_bytes(dest).map_err(|e| rand::Error::new(e))
    }
}
