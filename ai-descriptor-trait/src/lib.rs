use std::borrow::Cow;

pub trait ItemFeature {
    fn text(&self) -> Cow<'_,str>;
}

pub trait ItemWithFeatures {
    fn header(&self) -> Cow<'_,str>;
    fn features(&self) -> Vec<Cow<'_, str>>;
}

pub trait AIDescriptor {
    fn ai(&self) -> Cow<'_,str>;
}

impl<T: ItemWithFeatures> AIDescriptor for T {

    fn ai(&self) -> Cow<'_,str> {
        let mut lines: Vec<String> = vec![];
        lines.push(self.header().into());
        lines.push("It has the following features:".into());

        for feature in self.features() {
            lines.push(format!("- {}", feature));
        }
        Cow::Owned(lines.join("\n"))
    }
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
