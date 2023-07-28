
const COMPPOINT_SENTINEL: &str = "\u{FFFF}";

struct TokenizedResult {
    command_basename: String,
    arguments: Vec<String>,
    last_argument: String,
}

/// Reproduction of the algorithm used in Ruby tabry. Replace comppoint with a sentinel value,
/// split as a shell would, then find the token with the sentinel.
fn split_with_comppoint(compline: &str, comppoint: usize) -> Result<TokenizedResult, shell_words::ParseError> {
    let mut compline = compline.to_owned();

    // compline is characters, not bytes. Find where to insert the sentinel.
    let comppoint_byte: usize = compline.chars().take(comppoint).map(|c| c.len_utf16()).sum();
    compline.insert_str(comppoint_byte, COMPPOINT_SENTINEL);

    let all_tokens: Vec<String> = shell_words::split(&compline)?;

    let last_arg_index: usize = all_tokens.iter().enumerate().
        find(|(i, s)| s.contains(COMPPOINT_SENTINEL)).
        map(|(i, s)| i).unwrap();

    let (command, arguments, last_argument) = match &all_tokens[0..=last_arg_index] {
        [last_arg] => ("", vec![], last_arg.as_str()),
        [cmd, args@.., last_arg] => (cmd.as_str(), Vec::from(args), last_arg.as_str()),
        [] => unreachable!() // we'll always have at least the sentinel
    };

    let command_basename = command.split("/").last().unwrap_or("").to_owned();
    let last_argument = last_argument.replace(COMPPOINT_SENTINEL, "");

    Ok(TokenizedResult { command_basename, arguments, last_argument })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_returns_command_basename_argument_last_argument() {
          let res = split_with_comppoint("foo/bar abc def ghi", 13).unwrap();

          assert_eq!(res.command_basename, "bar");
          assert_eq!(res.arguments, vec!["abc"]);
          assert_eq!(res.last_argument, "def");
    }
}
