// ---------------- [ File: src/camel_case_token_with_comment.rs ]
crate::ix!();

pub type TokenQuad = [CamelCaseTokenWithComment; 4];

#[derive(Hash,Debug,Clone,PartialEq,Eq)]
pub struct CamelCaseTokenWithComment {
    data:    String,
    comment: Option<String>,
}

impl CamelCaseTokenWithComment {

    pub fn data(&self) -> &str {
        &self.data
    }
}

impl Display for CamelCaseTokenWithComment {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.comment {
            Some(comment) => write!(f, "{} -- {}", self.data, comment),
            None => write!(f, "{}", self.data),
        }
    }
}

impl CamelCaseTokenWithComment {

    pub fn target_path_for_ai_json_expansion(&self, target_dir: impl AsRef<Path>) -> PathBuf {

        // Convert 'token_name' to snake_case
        let snake_token_name = to_snake_case(&self.data);

        // Determine the output filename based on custom_id
        // You can customize this as needed, e.g., using token names
        let filename = format!("{}.json", snake_token_name);

        target_dir.as_ref().to_path_buf().join(filename)
    }
}

impl Into<String> for CamelCaseTokenWithComment {

    fn into(self) -> String {
        match self.comment {
            Some(ref comment) => format!("{} -- {}", self.data, comment),
            None              => format!("{}", self.data),
        }
    }
}

impl std::str::FromStr for CamelCaseTokenWithComment {
    type Err = TokenParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let line = s.trim();
        if line.is_empty() {
            return Err(TokenParseError::InvalidTokenLine(line.to_string()));
        }

        let parts: Vec<&str> = line.splitn(2, "--").collect();
        let data = parts[0].trim();
        if data.is_empty() {
            return Err(TokenParseError::InvalidTokenLine(line.to_string()));
        }

        let comment = parts.get(1).map(|&s| s.trim().to_string()).filter(|s| !s.is_empty());

        Ok(CamelCaseTokenWithComment {
            data: data.to_string(),
            comment,
        })
    }
}

pub async fn parse_token_file(filename: &str) 
    -> Result<Vec<CamelCaseTokenWithComment>, TokenParseError> 
{
    info!("parsing token file {}", filename);

    let file = File::open(filename).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut tokens = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match line.parse::<CamelCaseTokenWithComment>() {
            Ok(token) => tokens.push(token),
            Err(e) => {
                warn!("{:?}", e);
            }
        }
    }

    Ok(tokens)
}
