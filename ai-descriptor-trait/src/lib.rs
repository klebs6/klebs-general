// ---------------- [ File: ai-descriptor-trait/src/lib.rs ]
use std::borrow::Cow;
use language_enum::Language;

#[allow(unused_imports)]
use str_shorthand::lowercase_first_letter;

pub trait ItemFeature {
    fn text(&self) -> Cow<'_,str>;
}

pub trait ItemWithFeatures {

    /// Localised header.
    fn header_in(&self, lang: &Language) -> Cow<'_, str>;

    /// English header (legacy helper).
    fn header(&self) -> Cow<'_, str> {
        self.header_in(&Language::English)
    }

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

    /// Localised description.
    fn ai_in(&self, lang: &Language) -> Cow<'_, str> {
        tracing::trace!("Generating AI descriptor in {:?}", lang);

        let mut lines: Vec<String> = Vec::new();
        lines.push(self.header_in(&lang).to_string());

        let unique = crate::unique_items(&self.features());
        if !unique.is_empty() {
            lines.push(has_features_phrase(&lang).into());
        }

        for feat in unique {
            lines.push(format!("- {}", feat));
        }

        let out = Cow::Owned(lines.join("\n"));
        tracing::info!(%out, "Generated AI descriptor");
        out
    }
}

/// High‑level description facilities for any value that implements
/// [`ItemWithFeatures`].
///
/// * `ai` generates an English description (back‑compat).  
/// * `ai_in` generates a description in the requested language.  
/// * `ai_alt` remains a terse, single‑line variant.
pub trait AIDescriptor {

    /// English description (legacy default).
    fn ai(&self) -> Cow<'_,str>;

    /// Compact, single‑line English description.
    fn ai_alt(&self) -> Cow<'_,str> {
        unimplemented!("can implement this function for ai_alt() function")
    }

    /// Localised description.
    fn ai_in(&self, lang: &Language) -> Cow<'_, str>;
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

        fn header_in(&self, l: &Language) -> Cow<'_, str> {
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

/// Return a localized translation of the sentence  
/// “It has the following features:”.  
///  
/// If the provided language is not explicitly recognised, the function
/// gracefully falls back to English.
///
/// Supported languages:
/// * English  
/// * Latin  
/// * Ancient Greek  
/// * Russian  
/// * French  
/// * Italian  
/// * Swedish  
/// * Finnish  
/// * Icelandic
pub fn has_features_phrase(lang: &Language) -> &'static str {
    match lang {
        Language::English       => "It has the following features:",
        Language::Latin         => "Habet has proprietates sequentes:",
        Language::AncientGreek  => "Ἔχει τὰ ἑξῆς γνωρίσματα:",
        Language::Russian       => "Он обладает следующими характеристиками:",
        Language::French        => "Il possède les caractéristiques suivantes :",
        Language::Italian       => "Ha le seguenti caratteristiche:",
        Language::Swedish       => "Den har följande egenskaper:",
        Language::Finnish       => "Sillä on seuraavat ominaisuudet:",
        Language::Icelandic     => "Það hefur eftirfarandi einkenni:",
        Language::Arabic        => "له الميزات التالية:",
        Language::Swahili       => "Ina sifa zifuatazo:",
        Language::IrishGaeilge  => "Tá na tréithe seo a leanas aige:",
        _                       => "It has the following features:",
    }
}

