// ---------------- [ File: ai-descriptor-trait/src/lib.rs ]
use std::borrow::Cow;

#[allow(unused_imports)]
use str_shorthand::lowercase_first_letter;

pub trait ItemFeature {
    fn text(&self) -> Cow<'_,str>;
}

pub trait ItemWithFeatures {
    fn header(&self) -> Cow<'_,str>;
    fn features(&self) -> Vec<Cow<'_, str>>;
}

impl<T> ItemFeature for T where T: ItemWithFeatures {

    fn text(&self) -> Cow<'_,str> {

        let mut lines: Vec<String> = vec![];

        //lines.push("It is".to_string());
        //lines.push(lowercase_first_letter(&self.header()));
        lines.push(self.header().to_string());

        let unique = unique_items(&self.features());

        for feature in unique {
            lines.push(feature.to_string());
        }

        Cow::Owned(lines.join(" "))
    }
}

impl<T: ItemWithFeatures> AIDescriptor for T {

    fn ai(&self) -> Cow<'_,str> {

        let mut lines: Vec<String> = vec![];

        lines.push(self.header().into());

        let unique = unique_items(&self.features());

        if unique.len() > 0 {
            lines.push("It has the following features:".into());
        }

        for feature in unique {
            lines.push(format!("- {}", feature));
        }

        Cow::Owned(lines.join("\n"))
    }

    fn ai_alt(&self) -> Cow<'_,str> {

        let mut lines: Vec<String> = vec![];

        let unique = unique_items(&self.features());

        for feature in unique {
            lines.push(feature.into());
        }

        Cow::Owned(lines.join(" "))
    }
}

pub trait AIDescriptor {

    fn ai(&self) -> Cow<'_,str>;

    fn ai_alt(&self) -> Cow<'_,str> {
        unimplemented!("can implement this function for ai_alt() function")
    }
}

/// Extract unique items from a vector, maintaining their original order.
/// Items after the first occurrence are discarded.
pub fn unique_items<T>(items: &[T]) -> Vec<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    let mut seen = std::collections::HashSet::new();
    let mut unique = Vec::with_capacity(items.len());

    for item in items.iter() {
        if seen.insert(item) {
            unique.push(item.clone());
        }
    }

    unique
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestItem {
        header:   String,
        features: Vec<Cow<'static, str>>,
    }

    impl ItemWithFeatures for TestItem {
        fn header(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.header)
        }

        fn features(&self) -> Vec<Cow<'_, str>> {
            self.features.clone()
        }
    }

    #[test]
    fn test_ai_descriptor() {
        let item = TestItem {
            header: "An Item.".to_string(),
            features: vec![
                Cow::Borrowed("Feature 1"),
                Cow::Borrowed("Feature 2"),
                Cow::Borrowed("Feature 3"),
            ],
        };

        let expected_output = "\
An Item.
It has the following features:
- Feature 1
- Feature 2
- Feature 3";

        assert_eq!(item.ai(), expected_output);
    }
}
