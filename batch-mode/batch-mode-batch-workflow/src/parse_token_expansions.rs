crate::ix!();

pub async fn parse_token_expansions() -> Result<(), TokenExpanderError> {

    use rand::seq::SliceRandom;

    let mut rng = rand::thread_rng();

    let mut items = ExpandedBasicToken::load_from_directory("product/token-expansion-json").await?;

    items.shuffle(&mut rng);

    const TRAIN_LEN: usize = 10;

    for item in items.into_iter().take(10) {
        let fmt = StructuredLanguageForm::random();
        info!("Please ingest the following information, synthesize it, understand it, upgrade it, and convey it as a train of {} {} communicating the ideas to an apex consumer of maximal intelligence:", TRAIN_LEN, fmt.plural());
        info!("{:#?}", item);
    }

    let fire_rose = ExpandedBasicToken::load_from_file("product/token-expansion-json/the_fire_rose.json").await?;

    info!("{:#?}", fire_rose);

    Ok(())
}

pub async fn map_input_tokens_into_their_expanded_json_form<Expander:TokenExpander>() -> Result<(), TokenExpanderError> {

    let model      = LanguageModelType::Gpt4o;
    let token_file = "input.tokens";

    let input_tokens: Vec<CamelCaseTokenWithComment> 
        = parse_token_file(token_file).await?;

    info!("input tokens:");
    for token in &input_tokens {
        info!("{}", token);
    }

    Ok(())
}
