// ---------------- [ File: camel-case-token-with-comment/src/camel_case_token_with_comment.rs ]
crate::ix!();

pub type TokenQuad = [CamelCaseTokenWithComment; 4];

#[derive(Builder,Getters,Setters,Serialize,Deserialize,Hash,Debug,Clone,PartialEq,Eq)]
#[getset(get = "pub")]
#[builder(setter(into, strip_option))]
pub struct CamelCaseTokenWithComment {
    data:    String,
    comment: Option<String>,
}

impl Named for CamelCaseTokenWithComment {

    fn name(&self) -> Cow<'_,str> {
        Cow::Borrowed(&self.data)
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

#[cfg(test)]
mod test_camel_case_token_with_comment {
    use super::*;

    #[traced_test]
    fn test_from_str_valid() {
        info!("Testing CamelCaseTokenWithComment::from_str with valid input");
        let input = "TokenData -- Some comment";
        let token = CamelCaseTokenWithComment::from_str(input).unwrap();
        pretty_assert_eq!(token.data(), "TokenData");
        pretty_assert_eq!(*token.comment(), Some("Some comment".to_string()));
    }

    #[traced_test]
    fn test_from_str_no_comment() {
        info!("Testing CamelCaseTokenWithComment::from_str with no comment");
        let input = "TokenData";
        let token = CamelCaseTokenWithComment::from_str(input).unwrap();
        pretty_assert_eq!(token.data(), "TokenData");
        pretty_assert_eq!(*token.comment(), None);
    }

    #[traced_test]
    fn test_from_str_empty_data() {
        info!("Testing CamelCaseTokenWithComment::from_str with empty data");
        let input = " -- CommentOnly";
        let err = CamelCaseTokenWithComment::from_str(input).unwrap_err();
        assert!(matches!(err, TokenParseError::InvalidTokenLine(_)));
    }

    #[traced_test]
    fn test_from_str_empty_input() {
        info!("Testing CamelCaseTokenWithComment::from_str with empty input");
        let input = "";
        let err = CamelCaseTokenWithComment::from_str(input).unwrap_err();
        assert!(matches!(err, TokenParseError::InvalidTokenLine(_)));
    }

    #[traced_test]
    fn test_into_string_with_comment() {
        info!("Testing Into<String> for CamelCaseTokenWithComment with comment");
        let token = CamelCaseTokenWithComment::from_str("Data -- Comment").unwrap();
        let as_string: String = token.into();
        pretty_assert_eq!(as_string, "Data -- Comment");
    }

    #[traced_test]
    fn test_into_string_without_comment() {
        info!("Testing Into<String> for CamelCaseTokenWithComment without comment");
        let token = CamelCaseTokenWithComment::from_str("Data").unwrap();
        let as_string: String = token.into();
        pretty_assert_eq!(as_string, "Data");
    }

    #[traced_test]
    fn test_display_with_comment() {
        info!("Testing Display for CamelCaseTokenWithComment with comment");
        let token = CamelCaseTokenWithComment::from_str("Data -- Comment").unwrap();
        pretty_assert_eq!(format!("{}", token), "Data -- Comment");
    }

    #[traced_test]
    fn test_display_without_comment() {
        info!("Testing Display for CamelCaseTokenWithComment without comment");
        let token = CamelCaseTokenWithComment::from_str("Data").unwrap();
        pretty_assert_eq!(format!("{}", token), "Data");
    }

    #[traced_test]
    fn test_name_method() {
        info!("Testing name() from Named trait on CamelCaseTokenWithComment");
        let token = CamelCaseTokenWithComment::from_str("Data -- Comment").unwrap();
        pretty_assert_eq!(token.name(), "Data");
    }

    #[traced_test]
    fn test_token_quad_usage() {
        info!("Testing TokenQuad type alias (array of 4 CamelCaseTokenWithComment)");
        let t1 = CamelCaseTokenWithComment::from_str("Data1 -- Comment1").unwrap();
        let t2 = CamelCaseTokenWithComment::from_str("Data2 -- Comment2").unwrap();
        let t3 = CamelCaseTokenWithComment::from_str("Data3").unwrap();
        let t4 = CamelCaseTokenWithComment::from_str("Data4 -- 4").unwrap();

        let quad: TokenQuad = [t1, t2, t3, t4];
        pretty_assert_eq!(quad.len(), 4);
        pretty_assert_eq!(quad[0].data(), "Data1");
        pretty_assert_eq!(quad[1].data(), "Data2");
        pretty_assert_eq!(quad[2].data(), "Data3");
        pretty_assert_eq!(quad[3].data(), "Data4");
    }
}
