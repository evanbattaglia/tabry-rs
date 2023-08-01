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
  "$_rabry_executable" commands | while read cmd; do
  echo complete -F _rabry_completions "$cmd"
  complete -F _rabry_completions "$cmd"
  done
}

_rabry_completions() {
  _rabry_completions "$_rabry_path"/target/debug/rabry
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

  if [[ $result == *$'\n'$'\n'* ]]; then
    # double newline signals use of specials (file, directory)
    # Warning: fragile code ahead.
    # Split on double-newline to get regular options and specials.
    specials="$(echo "$result"|sed '1,/^$/d')"
    result="$(echo "$result)"|sed '/^$/q')"

    # First, add anything before the double newline in (regular options)
    COMPREPLY=($result)

    # File special
    if [[ $'\n'$specials$'\n' == *$'\n'file$'\n'* ]]; then
      # doesn't seem to be a "plusfiles" like there is for "plusdirs"
      COMPREPLY+=($(compgen -A file "${COMP_WORDS[$COMP_CWORD]}"))
    fi

    # Directory special
    if [[ $'\n'$specials$'\n' == *$'\n'directory$'\n'* ]]; then
      # If there are only directory results, use nospace to not add a space after it,
      # like "cd" tab completion does.
      [[ ${#COMPREPLY[@]} -eq 0 ]] && compopt -o nospace
      compopt -o plusdirs
    fi

    # "description_if_optionless" special: Options are are meant to be description or examples
    # and not actual options. Add an empty option so we won't tab complete.
    if [[ $'\n'$specials$'\n' == *$'\n'description_if_optionless$'\n'* ]]; then
      compopt -o nosort
      COMPREPLY+=('')
    fi
  else
    COMPREPLY=($result)
  fi

  IFS="$saveifs"
  [[ -n "$TABRY_DEBUG" ]] && echo -n tabry end bash: && date +%s.%N >&2
}

