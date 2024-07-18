use itertools::Itertools;
use std::collections::HashMap;
use std::hash::Hash;

/// Converts a vector into a HashMap where the keys are the elements of the vector
/// and the values are the counts of each element.
#[inline]
pub fn into_counts<T>(source: Vec<T>) -> HashMap<T, usize>
where
    T: Eq + Hash,
{
    source.into_iter().counts()
}

/// Inverts a HashMap such that the keys become the values and the values become the keys.
/// The resulting HashMap maps the original values to a vector of keys that had that value.
#[inline]
pub fn invert_map<T>(source: HashMap<T, usize>) -> HashMap<usize, Vec<T>>
where
    T: Eq + Hash + Clone,
{
    let mut m: HashMap<usize, Vec<T>> = HashMap::new();
    for (key, value) in source {
        m.entry(value).or_default().push(key);
    }
    m
}

/// Converts a vector into a HashMap where the keys are the counts of elements
/// and the values are vectors of elements that have that count.
#[inline]
pub fn into_count_map<T>(source: Vec<T>) -> HashMap<usize, Vec<T>>
where
    T: Eq + Hash + Clone,
{
    let counts = into_counts(source);
    invert_map(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_into_counts_empty() {
        let vec: Vec<i32> = Vec::new();
        let expected: HashMap<i32, usize> = HashMap::new();
        assert_eq!(into_counts(vec), expected);
    }

    #[test]
    fn test_into_counts_single_element() {
        let vec = vec![1];
        let mut expected = HashMap::new();
        expected.insert(1, 1);
        assert_eq!(into_counts(vec), expected);
    }

    #[test]
    fn test_into_counts_multiple_identical_elements() {
        let vec = vec![1, 1, 1, 1];
        let mut expected = HashMap::new();
        expected.insert(1, 4);
        assert_eq!(into_counts(vec), expected);
    }

    #[test]
    fn test_into_counts_mixed_elements() {
        let vec = vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4];
        let mut expected = HashMap::new();
        expected.insert(1, 1);
        expected.insert(2, 2);
        expected.insert(3, 3);
        expected.insert(4, 4);
        assert_eq!(into_counts(vec), expected);
    }

    #[test]
    fn test_into_counts_non_integer_elements() {
        let vec = vec!["a", "b", "b", "c", "c", "c"];
        let mut expected = HashMap::new();
        expected.insert("a", 1);
        expected.insert("b", 2);
        expected.insert("c", 3);
        assert_eq!(into_counts(vec), expected);
    }

    #[derive(Hash, Eq, PartialEq, Clone, Debug)]
    struct TestStruct {
        value: i32,
    }

    #[test]
    fn test_into_counts_custom_struct() {
        let vec = vec![
            TestStruct { value: 1 },
            TestStruct { value: 2 },
            TestStruct { value: 2 },
        ];
        let mut expected = HashMap::new();
        expected.insert(TestStruct { value: 1 }, 1);
        expected.insert(TestStruct { value: 2 }, 2);
        assert_eq!(into_counts(vec), expected);
    }

    #[test]
    fn test_invert_map_single_value() {
        let mut map = HashMap::new();
        map.insert(1, 1);
        let mut expected = HashMap::new();
        expected.insert(1, vec![1]);
        assert_eq!(invert_map(map), expected);
    }

    #[test]
    fn test_invert_map_multiple_values() {
        let mut map = HashMap::new();
        map.insert(1, 2);
        map.insert(2, 2);
        map.insert(3, 3);
        let mut expected = HashMap::new();
        expected.insert(2, vec![1, 2]);
        expected.insert(3, vec![3]);

        // Convert vectors to sets for comparison
        let result = invert_map(map)
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect::<HashSet<_>>()))
            .collect::<HashMap<_, _>>();

        let expected = expected
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect::<HashSet<_>>()))
            .collect::<HashMap<_, _>>();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_invert_map_empty() {
        let map: HashMap<i32, usize> = HashMap::new();
        let expected: HashMap<usize, Vec<i32>> = HashMap::new();
        assert_eq!(invert_map(map), expected);
    }

    #[test]
    fn test_into_count_map_empty() {
        let vec: Vec<i32> = Vec::new();
        let expected: HashMap<usize, Vec<i32>> = HashMap::new();
        assert_eq!(into_count_map(vec), expected);
    }

    #[test]
    fn test_into_count_map_single_element() {
        let vec = vec![1];
        let mut expected = HashMap::new();
        expected.insert(1, vec![1]);
        assert_eq!(into_count_map(vec), expected);
    }

    #[test]
    fn test_into_count_map_mixed_elements() {
        let vec = vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4];
        let mut expected = HashMap::new();
        expected.insert(1, vec![1]);
        expected.insert(2, vec![2]);
        expected.insert(3, vec![3]);
        expected.insert(4, vec![4]);
        // Use HashSet to compare vectors irrespective of their order
        for (key, value) in into_count_map(vec) {
            let expected_set: HashSet<_> = expected.remove(&key).unwrap().into_iter().collect();
            let result_set: HashSet<_> = value.into_iter().collect();
            assert_eq!(expected_set, result_set);
        }
    }

    #[test]
    fn test_large_input() {
        let vec: Vec<i32> = (0..1000).collect();
        let counts = into_counts(vec.clone());
        for i in 0..1000 {
            assert_eq!(counts.get(&i), Some(&1));
        }
        let inverted = invert_map(counts);
        for i in 0..1000 {
            assert_eq!(inverted.get(&1).unwrap().contains(&i), true);
        }
        let count_map = into_count_map(vec);
        for i in 0..1000 {
            assert_eq!(count_map.get(&1).unwrap().contains(&i), true);
        }
    }
}

