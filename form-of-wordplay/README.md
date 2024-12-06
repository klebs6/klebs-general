
# FormOfWordplay

**FormOfWordplay** is a Rust crate that encapsulates the richness of wordplay in an enumeration. The `FormOfWordplay` enum represents an exhaustive catalog of linguistic and rhetorical techniques, providing structured definitions and practical usage contexts for developers and enthusiasts who want to explore, generate, or analyze advanced forms of wordplay.

## Features

- **Comprehensive Enum**: Encodes a wide range of wordplay forms, from the familiar (puns, alliteration) to the advanced (antimetabole, hypallage).
- **Rich Descriptions**: Each variant includes a detailed description of an item showcasing the maximal and exhaustive use of the corresponding wordplay technique.
- **Ready for Integration**: Designed with traits such as `Plural`, `Default`, and `RandConstruct`, enabling seamless integration into Rust applications.

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
form_of_wordplay = "0.1.0"
```

Then, bring it into your project:

```rust
use form_of_wordplay::FormOfWordplay;
```

## The `FormOfWordplay` Enum

The `FormOfWordplay` enum provides structured access to various forms of wordplay. Each variant represents a distinct linguistic or rhetorical technique, accompanied by an AI-powered tag describing an advanced item that exemplifies the form.

### Example Variants

Here are a few examples of what you can do with the `FormOfWordplay` enum:

- `FormOfWordplay::Pun`:
  > A densely layered joke or passage exploiting multiple meanings of words or homophones, with intricate wit and interwoven semantic nuances.

- `FormOfWordplay::Alliteration`:
  > A poetic or prose composition saturated with the repetition of initial consonant sounds, creating a rhythmic, almost hypnotic effect.

- `FormOfWordplay::Palindrome`:
  > A poem or sentence crafted entirely as a palindrome, with complete semantic coherence forward and backward, achieving both structural symmetry and layered meaning.

### Full Enum

The full list of variants includes forms like:
- **Assonance**
- **Consonance**
- **Anagram**
- **Oxymoron**
- **Zeugma**
- **Litotes**
- ...and many more!

For the complete list and their descriptions, see the [source code](src/lib.rs).

## Usage

### Example: Generate Wordplay Variants

```rust
use form_of_wordplay::FormOfWordplay;

fn main() {
    let form = FormOfWordplay::Pun;
    println!("{:?}", form);
    // Output: Pun
}
```

### Example: Random Selection

With `RandConstruct` implemented, you can easily generate a random `FormOfWordplay`:

```rust
use form_of_wordplay::FormOfWordplay;
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let random_form = rng.gen::<FormOfWordplay>();
    println!("Random Wordplay Form: {:?}", random_form);
}
```

### Example: Iterate Over Forms

You can loop through all forms of wordplay (requires a crate like `strum` for deriving `EnumIter`):

```rust
use form_of_wordplay::FormOfWordplay;

fn main() {
    for form in FormOfWordplay::iter() {
        println!("{:?}", form);
    }
}
```

## Use Cases

- **Creative Writing Tools**: Generate prompts or examples based on different forms of wordplay.
- **Education**: Teach rhetorical and linguistic devices in an engaging and systematic way.
- **Game Development**: Use wordplay forms to build puzzles or challenges.
- **Linguistic Analysis**: Explore the structures and effects of wordplay in texts.

## Contributing

Contributions are welcome! If you'd like to add new variants, refine descriptions, or suggest improvements, feel free to open a pull request or an issue.

## License

This crate is licensed under the [MIT License](LICENSE). See the `LICENSE` file for details.

---

Enjoy exploring the vast universe of wordplay with **FormOfWordplay**! 🎉
