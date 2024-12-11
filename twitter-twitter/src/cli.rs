crate::ix!();

/// Twitter Messenger CLI Application
#[derive(StructOpt, Debug)]
#[structopt(name = "twitter_messenger", about = "A CLI Messenger using Twitter as Backend")]
pub enum Cli {
    /// Read your own tweets
    ReadTweets {
        /// Number of recent tweets to retrieve
        #[structopt(short, long, default_value = "10")]
        count: usize,
    },

    /// Get mentions directed at you within a specified time interval
    GetMentions {
        /// Time interval in hours (e.g., 24 for the past day)
        #[structopt(short, long)]
        hours: i64,
    },
}

