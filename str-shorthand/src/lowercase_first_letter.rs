crate::ix!();

pub fn lowercase_first_letter(input: &str) -> String {
    let mut graphemes = input.graphemes(true); // Split the string into graphemes
    if let Some(first) = graphemes.next() {
        format!("{}{}", first.to_lowercase(), graphemes.collect::<String>())
    } else {
        String::new()
    }
}
