use twitter_twitter::*;

#[tokio::main]
async fn main() -> Result<(), MessengerError> {
    // Parse CLI arguments
    let args = Cli::from_args();

    // Initialize configuration
    let config = ConfigBuilder::from_files("/Users/kleb")?; // Adjust the path as necessary

    // Initialize Twitter API handler
    let api = TwitterAPI::new(config);

    // Handle commands
    match args {
        Cli::ReadTweets { count } => {
            let tweets = api.get_user_tweets(count).await?;
            for tweet in tweets {
                println!("{}: {}", tweet.user.screen_name, tweet.text);
            }
        }
        Cli::GetMentions { hours } => {
            let interval = Duration::hours(hours);
            let mentions = api.get_mentions_within(interval).await?;
            for mention in mentions {
                println!(
                    "[{}] {}: {}",
                    mention.created_at, mention.user.screen_name, mention.text
                );
            }
        }
    }

    Ok(())
}

