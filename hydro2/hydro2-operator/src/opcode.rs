// ---------------- [ File: hydro2-operator/src/opcode.rs ]
crate::ix!();

unsafe impl Send for OpCode {}
unsafe impl Sync for OpCode {}

/// High-level codes for each stage/operator in the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpCode {
    Foo,
    Nothing,
    Sink,
    TestOp,
    MultiThing,
    NoOp,
    FailingOp,
    StreamTestOp,
    SingleValueTestOp,
    AddOp,
    MultiplyOp,
    ToStringOp,
    ConstantOp,
    IncrementOperator,
    SingleChannelPassthrough,
    DoubleToTriOp,
    DoubleOp,
    DoubleToTriTwoGenericsOp,
    Merge2Op,
    DoubleOutOp,
    QuadToQuadOp,
    SingleToTriOp,
    TriToQuadOp,
    TriToSingleOp,
    SplitAndDoubleOp,

    //---------------------------------[aure-books]
    AureDesignCover,
    AureGenerateAppendixMaterial,
    AureArchiveAndCatalogue,
    AureDesignAuthor,
    AureStructureChapters,
    AureCiteAndFootnote,
    AureAssembleColophon,
    AureOrchestrateDistribution,
    AureEditorialReview,
    AureFinalProofread,
    AureEmbedIllustrations,
    AureProvideIllustrations,
    AureRegisterInventory,
    AureAllocateISBN,
    AureLexicalRefinement,
    AureManuscriptInception,
    AureComputePricing,
    AureScheduleForPrintQueue,
    AureComplyWithRegulations,
    AureAnalyzeSales,
    AurePaginateText,

    //---------------------------------[aiwa-books]
    ManuscriptInception,
    AuthorDesign,
    ChapterStructuring,
    AppendixContent,
    LexicalRefinement,
    EditorialReview,
    CitationFootnote,
    ColophonAssembly,
    ISBNAllocation,
    IllustrationProvider,
    IllustrationEmbed,
    TextPagination,
    RegulatoryCompliance,
    PricingComputation,
    FinalProofread,
    PrintQueueScheduling,
    InventoryRegistration,
    DistributionOrchestration,
    ArchivalAndCatalog,
    SalesAnalytics,

    //---------------------------------[hydro2-tokens]
    /// (In each position), this vectorized operator takes a token at the input and produces
    /// a token cluster at the output
    ///
    TokenCluster,

    /// (In each position), this vectorized operator takes a token at the input and produces
    /// a described token at the output
    TokenDescribe,

    /// (In each position), this vectorized operator takes an object-descriptor at the input and
    /// produces an object-design-specification at the output
    ///
    TokenDesignObject,

    /// (In each position), this vectorized operator takes an object-descriptor at the input and
    /// produces a vector of object effects at the output
    TokenEnumerateObjectEffects,

    /// (In each position), this vectorized operator takes a sequence of causation (represented by
    /// a vector of tokens) at the input and produces a vector of predictions for downstream events
    /// at the output
    ///
    TokenEventSequencePredict,

    /// (In each position), this vectorized operator takes a token at the input and produces
    /// a token-expansion at the output
    TokenExpand,

    /// (In each position), this vectorized operator takes a token-expansion at the input and
    /// produces a parametrized structured-language text body at the output
    ///
    TokenExpansionToStructuredLanguage,

    /// (In each position), this vectorized operator takes a set-of-tokens at the input and
    /// produces a filtered-set-of-tokens at the output
    ///
    TokenFilter,

    /// (In each position), this vectorized operator takes a set-of-tokens at the input and
    /// produces a single-token at the output
    ///
    TokenFusion,

    /// (In each position), this vectorized operator takes a token at the input and produces
    /// a linguistic-relationship-DAG (specified by a set of graph database update operations) at
    /// the output
    ///
    TokenGenerateLanguageGraph,

    /// (In each position), this vectorized operator takes a token at the input and produces the
    /// token *transformed into an imperative instruction* as well as a vector of subinstructions
    /// at the output
    ///
    TokenInstructionize,

    /// (In each position), this vectorized operator takes a N-way polygonal set-of-tokens and
    /// a normalized-coordinate at the input and produces an interpolated token at the output
    ///
    TokenInterpolate,

    /// (In each position), this vectorized operator takes a set of tokens at the input and
    /// transforms them into an action loop (this happened, then that happened, then that happened,
    /// then back to this) at the output
    ///
    TokenLoop,

    /// (In each position), this vectorized operator takes an action-loop at the input and produces
    /// a set of tokens which (when followed by an agent) breaks it out of the loop at the
    /// output
    ///
    TokenLoopBreak,

    /// (In each position), this vectorized operator takes a block of text and a set of criteria at
    /// the input and produces a mask selection at the output
    ///
    TokenMaskSelect,

    /// (In each position), this vectorized operator takes an object-descriptor at the input and
    /// produces an evolved form of the same object at the output
    ///
    TokenObjectEvolve,

    /// (In each position), this vectorized operator takes a quad of tokens and a criteria at the
    /// input and produces a quad-of-quads of tokens at the output where each quad has been
    /// expanded based on the criteria
    ///
    TokenQuadExpand,

    /// (In each position), this vectorized operator takes a block of text at the input and
    /// produces a vector of extracted token quads at the output
    ///
    TokenQuadExtract,

    /// (In each position), this vectorized operator takes a quad of tokens at the input and
    /// produces a single fused token at the output
    ///
    TokenQuadFuse,

    /// (In each position), this vectorized operator takes a set of quads at the input and produces
    /// an ordered set of these quads (along with descriptions indicating why the particular order
    /// was chosen) at the output
    ///
    TokenQuadOrder,

    /// (In each position), this vectorized operator takes a set of tokens and a criteria at the
    /// input and produces a rank of these tokens according to these criteria at the output
    ///
    TokenRank,

    /// (In each position), this vectorized operator takes an object-descriptor at the input and
    /// produces a set-of-ingredients required to construct or otherwise obtain the object at the
    /// output
    ///
    TokenSelectIngredients,

    /// (In each position), this vectorized operator takes a vector of tokens at the input and
    /// produces a shuffled vector at the output
    ///
    TokenShuffle,

    /// (In each position), this vectorized operator takes an object-descriptor, a time interval
    /// width, and a number of intervals at the input and produces a simulation of this object
    /// through the intervals at the output. 
    ///
    /// The simulation is represented as a sequence of simulation frames, each frame containing
    /// information to be simulated at each step.
    ///
    TokenSimulateObject,

    /// (In each position), this vectorized operator takes a token at the input and produces
    /// the vector of tokens (which result from linguistically *splitting* the input token into
    /// subcomponents) at the output
    ///
    TokenSplit,

    /// (In each position), this vectorized operator takes a stream of tokens at the input and
    /// produces a collapsed (compressed) representation of the stream at the output.
    ///
    /// The compressed stream can be comprised of different tokens than the input, but should
    /// maintain a proper conceptual hierarchical relationship to the input. 
    ///
    /// For example, for a compression ratio of *four* we have the input stream (represented in
    /// X's) and the output stream represented as dashes:
    ///
    /// XXXX XXXX XXXX XXXX XXXX XXXX XXXX XXXX
    /// -    -    -    -    -    -    -    -   (each dash uncollapses to the four X's above it)
    ///
    /// We want to compress the input stream conceptually by choosing tokens which can optimally
    /// capture the essence of several others.
    ///
    TokenStreamCollapse,

    /// (In each position), this vectorized operator takes a sequence-of-tokens and a criteria at
    /// the input and produces a resequenced-sequence-of-tokens at the output, resequenced by the
    /// provided criteria.
    ///
    TokenStreamResequence,
}
