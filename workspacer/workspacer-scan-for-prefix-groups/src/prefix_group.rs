// ---------------- [ File: src/prefix_group.rs ]
crate::ix!();

// -----------------------------------------------------------------------------
// 1) Data structure describing one prefix group
// -----------------------------------------------------------------------------
#[derive(Builder,Getters,Debug,Clone)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct PrefixGroup<P,H> 
where 
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
{
    /// The “common prefix” name (e.g. "batch-mode")
    prefix: String,

    /// The facade crate, if any (named exactly the prefix)
    prefix_crate: Option<H>,

    /// The *-3p crate, if any
    three_p_crate: Option<H>,

    /// All crates that belong in this group
    member_crates: Vec<H>,

    #[builder(default="None")]
    _0: Option<std::marker::PhantomData<P>>,
}
