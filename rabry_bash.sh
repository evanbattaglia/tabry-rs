# TODO remove these notes --
# # Hmm, using some of this could be useful...
# COMP_TYPE
# Set to an integer value corresponding to the type of completion attempted that caused a completion function to be called: TAB, for normal completion, ‘?’, for listing completions after successive tabs, ‘!’, for listing alternatives on partial word completion, ‘@’, to list completions if the word is not unmodified, or ‘%’, for menu completion. This variable is available only in shell functions and external commands invoked by the programmable completion facilities (see Programmable Completion).

####

# USAGE:
# 1. Put the following you your .bash_profile:
#      source /my/path/to/rabry_bash.sh && _rabry_complete_all ~/.tabry
#    (You can use multiple colon-separated strings if you want)
# 2. Put *.tabry and/or compiled *.json configs in the ~/.tabry directory. If you
#   are using *.tabry files, currently you'll also need to have the tabry
#   compiler (tabryc) installed and in the path too (rabry will compile and
#   cache the results)
# 3. Enjoy your completions!

_rabry_path=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
_rabry_executable="$_rabry_path/target/debug/rabry"

_rabry_complete_all() {
  [[ -n "$1" ]] && export RABRY_IMPORT_PATH="$1"
  [[ -x "$_rabry_executable" ]] || return
  local oldifs="$IFS"
  IFS=$'\n'
  for cmd in $("$_rabry_executable" commands); do
      complete -F _rabry_completions $cmd
  done
  IFS="$oldifs"
}

_rabry_completions() {
  _rabry_completions_internal "$_rabry_path"/target/debug/rabry
}

# This is unchanged from tabry, except to remove the second arg
_rabry_completions_internal()
{
  local tabry_bash_executable="$1"

  [[ -n "$TABRY_DEBUG" ]] && echo && echo -n tabry start bash: && date +%s.%N >&2
  local saveifs="$IFS"
  IFS=$'\n'

  [[ -n "$TABRY_DEBUG" ]] && printf "%q %q %q %q\n" "$tabry_bash_executable" "$COMP_LINE" "$COMP_POINT"
  local result=`"$tabry_bash_executable" "$COMP_LINE" "$COMP_POINT"`
  local specials
  local specials_line

  if [[ $result == *$'\n'$'\n'* ]]; then
    # double newline signals use of specials (file, directory)
    # Warning: fragile code ahead.
    # Split on double-newline to get regular options and specials.
    specials="$(echo "$result"|sed '1,/^$/d')"
    result="$(echo "$result)"|sed '/^$/q')"

    # First, add anything before the double newline in (regular options)
    COMPREPLY=($result)

    while IFS= read -r specials_line; do
      if [[ "$specials_line" == "file" ]]; then
        # File special
        # doesn't seem to be a "plusfiles" like there is for "plusdirs"
        COMPREPLY+=($(compgen -A file "${COMP_WORDS[$COMP_CWORD]}"))
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
    COMPREPLY=($result)
  fi

  IFS="$saveifs"
  [[ -n "$TABRY_DEBUG" ]] && echo -n tabry end bash: && date +%s.%N >&2
}

