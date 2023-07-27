use super::types::*;

/// Code for token matching that depends only on the type.
/// Used by the machine, broken out to help keep machine smaller.

pub trait TokenMatching {
    fn match_token(&self, token: &str) -> bool;
}

fn alias_matches(alias: &str, token: &str) -> bool {
    if alias.len() > 1 {
        token.len() >= 3 && &token[0..=1] == "--" && &token[2..] == alias
    } else {
        token.len() >= 2 && &token[0..=0] == "-" && &token[1..] == alias
    }
}

impl TokenMatching for TabryConcreteFlag {
    fn match_token(&self, token: &str) -> bool {
        alias_matches(&self.name, token) ||
            self.aliases.iter().any(|alias| alias_matches(alias, token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_concrete_flag() -> TabryConcreteFlag {
        make_concrete_flag_with_name("foo")
    }

    fn make_concrete_flag_with_name(name: &str) -> TabryConcreteFlag {
        TabryConcreteFlag {
            name: name.to_owned(),
            aliases: vec!["f".to_owned(), "foobar".to_owned()],
            options: vec![],
            arg: false,
            required: false
        }
    }

    #[test]
    fn test_match_token_concrete_flag_long_name() {
        let flag = make_concrete_flag();
        assert!(flag.match_token("--foo"));
        assert!(!flag.match_token("--foob"));
        assert!(!flag.match_token("--fo"));
        assert!(!flag.match_token("--f"));
        assert!(!flag.match_token("-foo"));
    }

    #[test]
    fn test_match_token_concrete_flag_long_alias() {
        let flag = make_concrete_flag();
        assert!(flag.match_token("--foobar"));
        assert!(!flag.match_token("--foobars"));
        assert!(!flag.match_token("--foob"));
        assert!(!flag.match_token("-foobar"));
    }

    #[test]
    fn test_match_token_concrete_flag_short_alias() {
        let flag = make_concrete_flag();
        assert!(flag.match_token("-f"));
        assert!(!flag.match_token("--f"));
        assert!(!flag.match_token("-"));
        assert!(!flag.match_token("-fo"));
    }

    #[test]
    fn test_match_token_concrete_flag_short_name() {
        let flag = make_concrete_flag_with_name("b");
        assert!(flag.match_token("-b"));
        assert!(!flag.match_token("--b"));
        assert!(!flag.match_token("-"));
        assert!(!flag.match_token("-ba"));
    }

}
