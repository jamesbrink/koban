#!/usr/bin/env sh
set -eu

max_source_lines=900
max_test_lines=1100
failed=0

check_file() {
  file="$1"
  max="$2"
  lines="$(wc -l < "$file" | tr -d ' ')"
  if [ "$lines" -gt "$max" ]; then
    printf '%s has %s lines, above the %s line budget\n' "$file" "$lines" "$max" >&2
    failed=1
  fi
}

for file in $(find src tests -name '*.rs' -type f); do
  case "$file" in
    src/tests/*|tests/*)
      check_file "$file" "$max_test_lines"
      ;;
    *)
      check_file "$file" "$max_source_lines"
      ;;
  esac
done

exit "$failed"
