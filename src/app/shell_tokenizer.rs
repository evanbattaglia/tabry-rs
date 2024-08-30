const COMPPOINT_SENTINEL: &str = "\u{FFFF}";

pub struct TokenizedResult {
    pub command_basename: String,
    pub arguments: Vec<String>,
    pub last_argument: String,
}

/// Reproduction of the algorithm used in Ruby tabry. Replace comppoint with a sentinel value,
/// split as a shell would, then find the token with the sentinel. This is slightly better than
/// just chopping off the string after the comppoint because iv the lat arg is correctly quoted but
/// we are in the middle, shell_words split will return an error. TODO: However this still isn't
/// perfect as other shell completion seems to _not_take into account the rest of the last token as
/// we here. This could probably still be improved (maybe we should chop off the rest of the
/// last_token after the sentinel). We should also investigate use of COMP_WORDS and COMP_CWORD
/// which may help with special quoting situations not handled by shell_words.
pub fn split_with_comppoint(
    compline: &str,
    comppoint: usize,
) -> Result<TokenizedResult, shell_words::ParseError> {
    let mut compline = compline.to_owned();

    // compline is characters, not bytes. Find where to insert the sentinel.
    let comppoint_byte: usize = compline
        .chars()
        .take(comppoint)
        .map(|c| c.len_utf16())
        .sum();
    compline.insert_str(comppoint_byte, COMPPOINT_SENTINEL);

    let all_tokens: Vec<String> = shell_words::split(&compline)?;

    let last_arg_index: usize = all_tokens
        .iter()
        .enumerate()
        .find(|(_i, s)| s.contains(COMPPOINT_SENTINEL))
        .map(|(i, _s)| i)
        .unwrap();

    let (command, arguments, last_argument) = match &all_tokens[0..=last_arg_index] {
        [last_arg] => ("", vec![], last_arg.as_str()),
        [cmd, args @ .., last_arg] => (cmd.as_str(), Vec::from(args), last_arg.as_str()),
        [] => unreachable!(), // we'll always have at least the sentinel
    };

    let command_basename = command.split('/').last().unwrap_or("").to_owned();
    let last_argument = last_argument.replace(COMPPOINT_SENTINEL, "");

    Ok(TokenizedResult {
        command_basename,
        arguments,
        last_argument,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_result(
        compline: &str,
        comppoint: usize,
        expected_cmd_basename: &str,
        expected_arguments: Vec<&str>,
        expected_last_argument: &str,
    ) {
        let res = split_with_comppoint(compline, comppoint).unwrap();
        let expected_args_vec = expected_arguments
            .iter()
            .map(|s| String::from(*s))
            .collect::<Vec<String>>();
        assert_eq!(res.command_basename, expected_cmd_basename);
        assert_eq!(res.arguments, expected_args_vec);
        assert_eq!(res.last_argument, expected_last_argument);
    }

    #[test]
    fn test_tokenizer_returns_command_basename_argument_last_argument() {
        assert_result(
            "foo/bar abc def ghi",
            13, // input
            "bar",
            vec!["abc"],
            "def", // expected
        );
    }

    #[test]
    fn test_tokenizer_treats_only_one_argument_as_last_arg() {
        assert_result("foo bar", 5, "foo", vec![], "bar");
    }

    #[test]
    fn test_tokenizer_handles_quotes_and_backslashes_like_a_shell() {
        assert_result(
            r#""/foo bar/waz" a'b 'c\ d "ef g" "hi jk" lmn"#,
            38,
            "waz",
            vec!["ab c d", "ef g"],
            "hi jk",
        );
    }

    #[test]
    fn test_tokenizer_uses_comppoint_to_correctly_figures_out_whether_we_are_still_on_the_last_token(
    ) {
        assert_result("jir add ", 7, "jir", vec![], "add");
        assert_result("jir add ", 8, "jir", vec!["add"], "");
    }

    #[test]
    fn test_tokenizer_treats_only_one_token_as_last_arg() {
        assert_result("abc", 2, "", vec![], "abc");
    }

    #[test]
    fn test_tokenizer_supports_empty_strings() {
        assert_result("", 0, "", vec![], "");
    }
}
