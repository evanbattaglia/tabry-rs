set -x TABRY_IMPORT_PATH "$TABRY_IMPORT_PATH:$HOME/.local/share/tabry"

if not set -q _tabry_executable{{UNIQ_FN_ID}}
  set script_dir (dirname (status filename))
  set -g _tabry_executable{{UNIQ_FN_ID}} "$script_dir/target/debug/tabry"
end

function tabry_completion_init{{UNIQ_FN_ID}}
  set cmd $argv[1]
  complete -c "$cmd" -f -a "(__tabry_offer_completions{{UNIQ_FN_ID}})"
end

# Init completions for all commands in the path
function tabry_completion_init_all{{UNIQ_FN_ID}}
  for cmd in ($_tabry_executable{{UNIQ_FN_ID}} commands)
    tabry_completion_init{{UNIQ_FN_ID}} $cmd
  end
end

# Init completions for all commands in the path
function tabry_completion_init_all
  for cmd in (tabry commands)
    tabry_completion_init{{UNIQ_FN_ID}} $cmd
  end
end

# parse the results from tabry
function __tabry_parse_results{{UNIQ_FN_ID}}
  # take from $result until two newlines
  set -l completions
  set -l done_completions "false"
  set -l offer_dirs "false"
  set -l offer_files "false"

  for line in $argv
    if test "x$line" = "x"
      set done_completions "true"
      continue;
    else if test $done_completions = "false"
      set -a completions $line
    else if test "$line" = "file"
      set offer_files true
    else if test "$line" = "dir"
      set offer_dirs true
    end
  end

  for completion in $completions
    echo $completion
  end

  if test "$offer_dirs" = "true"
    __fish_complete_directories (__tabry_get_token_on_cursor{{UNIQ_FN_ID}} $cmd $cursor_position)
  else if test "$offer_files" = "true"
    __fish_complete_path (__tabry_get_token_on_cursor{{UNIQ_FN_ID}} $cmd $cursor_position)
  end
end

# gets the token that the cursor is on,
# including if the cursor is on a space just after the token
#
# foo  bar baz
#     ^ cursor_position
# returns ""
#
# foo  bar baz
#        ^ cursor_position
# returns "bar"
#
# foo  bar baz
#         ^ cursor_position
# returns "bar"
function __tabry_get_token_on_cursor{{UNIQ_FN_ID}}
  set -l cmd $argv[1]
  set -l cursor_position (math $argv[2] + 1) # fish is 1-indexed

  # foo  c bar baz
  #        ^ cursor_position
  # we want to return "bar"
  set -l curr_index 1
  set -l token ""
  for char in (string split '' $cmd)
    # if char is a space and we're before cursor_position,
    if test "$char" = " " -a $curr_index -lt $cursor_position
      set token ""
    end

    if test "$char" = " " -a $curr_index -ge $cursor_position
      break;
    end

    if test "$char" != " "
      set token (string join '' $token $char)
    end
    set curr_index (math $curr_index + 1)
  end
  echo "$token"
end

function __tabry_offer_completions{{UNIQ_FN_ID}}
  set SCRIPT (status --current-filename)
  set SCRIPT_DIR (dirname $SCRIPT)

  # -C "Get cursor position"
  set cursor_position (commandline -C)
  set cmd (commandline)

  set -l result ($_tabry_executable{{UNIQ_FN_ID}} complete --include-descriptions "$cmd" "$cursor_position")

  # get the last item
  
  # if result ends with "\n\nfile", then we should offer files
  # if result ends with "\n\ndir", then we should offer directories
  __tabry_parse_results{{UNIQ_FN_ID}} $result
end
