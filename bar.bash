_tabry_executable='/Users/paul.gray/dev/tabry-rs/result/bin/tabry'
_tabry_executable=${_tabry_executable:-$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )"/.. &> /dev/null && pwd)/target/debug/tabry}

_tabry_complete_all() {
  if [[ -z "$TABRY_IMPORT_PATH" ]]; then
    if [[ -n "$_tabry_imports_path" ]]; then
      export TABRY_IMPORT_PATH="$_tabry_imports_path"
    else
      TABRY_IMPORT_PATH=~/.local/share/tabry
    fi
  fi

# TODO could also have a 'wizard' which asks you to put it in your .bash_profile or .bashrc or whatever, if we get it in nixpkgs it could literally be: `, tabry` which installs it in your bash_profile
# (home-manager plugin would be nice too.)

# USAGE:
# 1. Put the following you your .bash_profile:
#      source <(tabry bash)
#    OR
#      source <(tabry bash ~/.tabry:~/my-tabry-files/)
#
#    (You can use multiple colon-separated strings if you want instead of
#    ~/.tabry)
# 2. Put *.tabry and/or compiled *.json configs in the ~/.tabry directory. If you
#   are using *.tabry files, tabry will compile and cache the results!
# 3. Enjoy your completions!

_tabry_executable=${_tabry_executable:-$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )"/.. &> /dev/null && pwd)/target/debug/tabry}

_tabry_complete_allBAR() {
  if [[ -z "$TABRY_IMPORT_PATH" ]]; then
    if [[ -n "$_tabry_imports_path" ]]; then
      export TABRY_IMPORT_PATH="$_tabry_imports_path"
    else
      TABRY_IMPORT_PATH=~/.local/share/tabry
    fi
  fi

  [[ -x "$_tabry_executable" ]] || { echo "tabry_bash.sh: error: can't find tabry executable at $_tabry_executable -- if you are using the script from source rather than using via 'tabry bash', perhaps you need to run 'cargo build'?"; return 1; }
  local oldifs="$IFS"
  IFS=$'\n'
  for cmd in $("$_tabry_executable" commands); do
      complete -F _tabry_completionsBAR $cmd
  done
  IFS="$oldifs"
}

_tabry_complete_one_commandBAR() {
  complete -F _tabry_completionsBAR $1
}

_tabry_completionsBAR() {
  _tabry_completions_internalBAR "$_tabry_executable"
}

_tabry_set_compreply_from_linesBAR() {
  # Feed in lines from a variable, quoting each line.
  # Using readarray is much faster than using += many times to build the array.
  local lines="$1"
  local saveifs="$IFS"
  COMPREPLY=()
  readarray -t COMPREPLY < <(
    IFS=$'\n'
    while IFS= read -r line; do
      if [[ -n "$line" ]]; then
        printf '%q\n' "$line"
      fi
    done <<< "$lines"
    IFS="$saveifs"
  )
}

# This is unchanged from ruby tabry, except to remove the second arg
_tabry_completions_internalBAR()
{
  local tabry_bash_executable="$1"

  [[ -n "$TABRY_DEBUG" ]] && echo && echo -n tabry start bash: && date +%s.%N >&2
  local saveifs="$IFS"
  IFS=$'\n'

  [[ -n "$TABRY_DEBUG" ]] && printf "%q %q %q %q\n" "$tabry_bash_executable" complete "$COMP_LINE" "$COMP_POINT"
  local result=$("$tabry_bash_executable" complete "$COMP_LINE" "$COMP_POINT")
  local specials
  local specials_line

  if [[ $result == *$'\n'$'\n'* ]]; then
    # double newline signals use of specials (file, directory)
    # Warning: fragile code ahead.
    # Split on double-newline to get regular options and specials.
    specials="$(echo "$result"|sed '1,/^$/d')"
    result="$(echo "$result)"|sed '/^$/q')"

    # First, add anything before the double newline in (regular options)
    _tabry_set_compreply_from_linesBAR "$result"

    while IFS= read -r specials_line; do
      if [[ "$specials_line" == "file" ]]; then
        # File special
        # doesn't seem to be a "plusfiles" like there is for "plusdirs"
        COMPREPLY+=($(printf "%q" "$(compgen -A file "${COMP_WORDS[$COMP_CWORD]}")"))
      elif [[ "$specials_line" == "dir" ]]; then
        # Directory special
        # If there are only directory results, use nospace to not add a space after it,
        # like "cd" tab completion does.
        [[ ${#COMPREPLY[@]} -eq 0 ]] && compopt -o nospace
        compopt -o plusdirs
      elif [[ "$specials" == "description_if_optionless" ]]; then
        # "description_if_optionless" special: Options are are meant to be description or examples
        # and not actual options. Add an empty option so we won't tab complete.
        compopt -o nosort
        COMPREPLY+=('')
      elif [[ "$specials_line" == "delegate "* ]]; then
        local delegate_cmd="${specials_line#delegate }"

        # split delegate_cmd to get the actual command (first word):
        # this is not reliable but will work for now...
        local delegate_cmd_arg0="${delegate_cmd%% *}"
        local complete_fn=$(complete -p "$delegate_cmd_arg0" 2>/dev/null | sed 's/.*-F \([^ ]*\) .*/\1/')
        if [[ -z "$complete_fn" ]]; then
          _completion_loader "$delegate_cmd_arg0"
        fi
        complete_fn=$(complete -p "$delegate_cmd_arg0" 2>/dev/null | sed 's/.*-F \([^ ]*\) .*/\1/')
        if [[ -z "$complete_fn" ]]; then
          echo "Error: Could not find completion function for $delegate_cmd_arg0" >&2
          return 1
        fi

        # Backup
        local comp_cword="$COMP_CWORD"
        local comp_point="$COMP_POINT"
        local comp_line="$COMP_LINE"
        local comp_words=("${COMP_WORDS[@]}")
        local comp_key="$COMP_KEY"
        local comp_type="$COMP_TYPE"
        local comp_reply=("${COMPREPLY[@]}")

        # get completions -> COMPREPLY
        IFS="$saveifs"
        COMP_LINE="$delegate_cmd"
        COMP_POINT="${#delegate_cmd}"
        #__compal__split_cmd_line "$COMP_LINE"
        COMP_WORDS=("${__compal__retval[@]}" "")
        COMP_WORDS=($delegate_cmd "")
        COMP_CWORD=$((${#COMP_WORDS[@]} - 1))
        COMPREPLY=()
        "$complete_fn"

        # Reset the completion variables
        IFS=$'\n'
        COMP_CWORD="$comp_cword"
        COMP_POINT="$comp_point"
        COMP_LINE="$comp_line"
        COMP_WORDS=("${comp_words[@]}")
        COMP_KEY="$comp_key"
        COMP_TYPE="$comp_type"

        # Concatenate the completions
        COMPREPLY=("${comp_reply[@]}" "${COMPREPLY[@]}")
      fi
    done <<< "$specials"
  else
    _tabry_set_compreply_from_linesBAR "$result"
    COMPREPLY=()
    while IFS= read -r line; do
      COMPREPLY+=($(printf "%q" "$line"))
    done <<< "$result"
  fi

  IFS="$saveifs"
  [[ -n "$TABRY_DEBUG" ]] && echo -n tabry end bash: && date +%s.%N >&2
}

_tabry_complete_allBAR
