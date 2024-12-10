crate::ix!();

/// Crate Activity Analyzer
#[derive(Getters,StructOpt, Debug)]
#[structopt(name = "crate-activity")]
pub struct CrateActivityCli {
    /// Toggle to enable or disable correlation analysis
    #[structopt(long, short = "c", help = "Display correlation analysis")]
    #[getset(get = "pub")]
    show_correlations: bool,

    /// Toggle to enable or disable PCA analysis
    #[structopt(long, short = "p", help = "Perform PCA analysis")]
    #[getset(get = "pub")]
    perform_pca: bool,

    /// Toggle to enable hierarchical clustering
    #[structopt(long, short = "h", help = "Perform hierarchical clustering")]
    #[getset(get = "pub")]
    perform_hierarchical_clustering: bool,

    /// Toggle to enable correlation network analysis
    #[structopt(long, short = "n", help = "Perform correlation network analysis")]
    #[getset(get = "pub")]
    correlation_network: bool,

    /// Threshold for including edges in the correlation network graph
    #[structopt(long, default_value = "0.7", help = "Correlation threshold for network edges")]
    #[getset(get = "pub")]
    network_threshold: f64,

    /// Use Girvan–Newman algorithm to find a specified number of communities
    #[structopt(long, short = "g", help = "Target number of communities for Girvan–Newman")]
    #[getset(get = "pub")]
    girvan_newman: Option<usize>,

    /// Compute betweenness centrality for nodes (and edges) and display top nodes
    #[structopt(long, short = "b", help = "Compute betweenness centrality and display top nodes")]
    #[getset(get = "pub")]
    compute_betweenness: bool,

    /// Print a summary of the network graph
    #[structopt(long, short = "s", help = "Print a summary of the network graph")]
    #[getset(get = "pub")]
    print_summary: bool,

    /// Toggle to enable time-lagged correlation analysis
    #[structopt(long, short = "t", help = "Compute time-lagged correlations")]
    #[getset(get = "pub")]
    time_lag_correlations: bool,

    /// Maximum lag in days for time-lagged correlations
    #[structopt(long, default_value = "7", help = "Maximum lag (in days) to consider for time-lag correlations")]
    #[getset(get = "pub")]
    max_lag: i32,

    /// Z-score threshold for outlier detection (MAD-based)
    #[structopt(long, default_value = "3.0", help = "Z-score threshold for outlier detection")]
    #[getset(get = "pub")]
    outlier_z_threshold: f64,

    /// Downweight outliers instead of removing them
    #[structopt(long, help = "Downweight outliers instead of removing them")]
    #[getset(get = "pub")]
    downweight_outliers: bool,

    /// Factor by which to downweight outliers if --downweight-outliers is used
    #[structopt(long, default_value = "0.1", help = "Downweight factor for outliers")]
    #[getset(get = "pub")]
    outlier_weight: f64,

    /// Disable outlier handling altogether
    #[structopt(long, help = "Disable outlier detection and handling")]
    #[getset(get = "pub")]
    disable_outlier_handling: bool,
}
