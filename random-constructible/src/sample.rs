crate::ix!();

pub fn sample_variants_with_probabilities<K: Clone + Eq + Hash + Sized, RNG: Rng + ?Sized>(rng: &mut RNG, probs: &HashMap<K,f64>) 
-> K 
{
    let variants: Vec<_> = probs.keys().cloned().collect();
    let weights:  Vec<_> = variants.iter().map(|v| probs[v]).collect();
    let dist = rand::distributions::WeightedIndex::new(&weights).unwrap();
    variants[dist.sample(rng)].clone()
}
