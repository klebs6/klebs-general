#[inline] pub fn transform<T, U, F>(slice: &[T], f: F) -> Vec<U>
where F: FnMut(&T) -> U
{
    slice.iter().map(f).collect()
}

#[inline] pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}


