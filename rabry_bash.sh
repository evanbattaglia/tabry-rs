
_rabry_VEHICLES_completions() {
  _rabry_completions_internal /home/evan/localonly/tdev/rabry/target/debug/rabry /home/evan/localonly/tdev/rabry/fixtures/vehicles.json
}

complete -F _rabry_VEHICLES_completions vehicles

# This is unchanged from tabry:
_rabry_completions_internal()
{
  local tabry_bash_executable="$1"
  local tabry_bash_arg="$2"

  [[ -n "$TABRY_DEBUG" ]] && echo && echo -n tabry start bash: && date +%s.%N >&2
  local saveifs="$IFS"
  IFS=$'\n'

  [[ -n "$TABRY_DEBUG" ]] && printf "%q %q %q %q\n" "$tabry_bash_executable" "$tabry_bash_arg" "$COMP_LINE" "$COMP_POINT"
  local result=`"$tabry_bash_executable" "$tabry_bash_arg" "$COMP_LINE" "$COMP_POINT"`
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

