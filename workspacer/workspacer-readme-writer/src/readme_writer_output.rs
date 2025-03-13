// ---------------- [ File: workspacer-readme-writer/src/readme_writer_output.rs ]
//NOTE: these doc comments are ultimately placed in our schema, which typically goes to an AI.
//
crate::ix!();

/// This structure is a schema and a directive indicating what we need you to provide for us in
/// your response to our query.
///
#[derive(AiJsonTemplate,SaveLoad,Builder,Getters,Debug,Clone,Serialize,Deserialize)]
#[getset(get="pub")]
#[builder(setter(into))]
pub struct AiReadmeWriterDesiredOutput {

    /// This string should be valid markdown representing the readme for this crate.
    /// Please make it terse, useful, and designed for the apex consumer of clear intent and
    /// industrious nature. Use advanced vocabulary. 
    ///
    /// If this crate involves any concepts from mathematics and/or physics, please document them
    /// here.
    ///
    full_readme_markdown: String,

    /// We will place this package_description in our Cargo.toml file so the users of this crate
    /// know what it does.
    ///
    /// Please make the discription technical, memorable, funny, and useful.
    ///
    package_description: String,

    /// These keywords will be used in our Cargo.toml file so the users of this crate can easily
    /// find it in the package system. Keywords should be useful, varied, meaningful, and reflect
    /// the contents of our crate. They should make it easy to find. Please provide exactly five.
    ///
    package_keywords:    Vec<String>,

    /// These categories will be used in our Cargo.toml file so the users of this crate can find it
    /// when they sort through other packages of the same category on crates.io.
    ///
    /// Choose max 5 and make sure they are actually meaningful for our crate.
    ///
    /// These categories should only be chosen from among the legal crates.io categories. These
    /// are:
    ///
    /// accessibility
    /// aerospace
    /// algorithms
    /// api-bindings
    /// asynchronous
    /// authentication
    /// caching
    /// command-line-interface
    /// command-line-utilities
    /// compilers
    /// compression
    /// computer-vision
    /// concurrency
    /// config
    /// cryptography
    /// data-structures
    /// database
    /// database-implementations
    /// date-and-time
    /// development-tools
    /// email
    /// embedded
    /// emulators
    /// encoding
    /// external-ffi-bindings
    /// filesystem
    /// finance
    /// game-development
    /// game-engines
    /// games
    /// graphics
    /// gui
    /// hardware-support
    /// internationalization
    /// localization
    /// mathematics
    /// memory-management
    /// multimedia
    /// network-programming
    /// no-std
    /// os
    /// parser-implementations
    /// parsing
    /// rendering
    /// rust-patterns
    /// science
    /// simulation
    /// template-engine
    /// text-editors
    /// text-processing
    /// value-formatting
    /// virtualization
    /// visualization
    /// wasm
    /// web-programming
    package_categories:  Vec<String>,
}
