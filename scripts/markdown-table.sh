#!/usr/bin/env bash

# Script copied from: https://josh.fail/2022/pure-bash-markdown-table-generator/

# Usage: markdown-table -COLUMNS [CELLS]
#        markdown-table -sSEPARATOR < file
#
# NAME
#   markdown-table -- generate markdown tables
#
# SYNOPSIS
#   markdown-table -COLUMNS [CELLS]
#   markdown-table -sSEPARATOR < file
#
# DESCRIPTION
#   markdown-table helps generate markdown tables. Manually supply arguments
#   and a column count to generate a table, or pass in a delimited file to
#   convert to a table.
#
# OPTIONS
#   -COLUMNS
#       Number of columns to include in output.
#
#   -sSEPARATOR
#       String used to separate columns in input files.
#
#   --csv
#       Shortcut for `-s,` to parse CSV files. Note that this is a "dumb" CSV
#       parser -- it won't work if your cells contain commas!
#
#   --tsv
#       Shortcut for `-s$'\t'` to parse TSV files.
#
#   -h, --help
#       Prints help text and exits.
#
# EXAMPLES
#   Build a 4 column markdown table from arguments:
#     markdown-table -4 \
#       "Heading 1"  "Heading 2" "Heading 3" "Heading 4" \
#       "Hi"         "There"     "From"      "Markdown!" \
#       "Everything" "Is"        "So"        "Nicely Aligned!"
#
#   Convert a CSV file into a markdown table:
#     markdown-table -s, < some.csv
#     markdown-table --csv < some.csv
#
#   Convert a TSV file into a markdown table:
#     markdown-table -s$'\t' < test.tsv
#     markdown-table --tsv < test.tsv

# Call this script with DEBUG=1 to add some debugging output
if [[ "$DEBUG" ]]; then
  export PS4='+ [${BASH_SOURCE##*/}:${LINENO}] '
  set -x
fi

set -e

# Echoes given args to STDERR
#
# $@ - args to pass to echo
warn() {
  echo "$@" >&2
}

# Print the help text for this program
#
# $1 - flag used to ask for help ("-h" or "--help")
print_help() {
  sed -ne '/^#/!q;s/^#$/# /;/^# /s/^# //p' < "$0" |
    awk -v f="$1" '
      f == "-h" && ($1 == "Usage:" || u) {
        u=1
        if ($0 == "") {
          exit
        } else {
          print
        }
      }
      f != "-h"
      '
}

# Returns the highest number in the given arguments
#
# $@ - one or more numeric arguments
max() {
  local max=0 arg

  for arg; do
    (( ${arg:-0} > max )) && max="$arg"
  done

  printf "%s" "$max"
}

# Formats a table in markdown format
#
# $1 - field separator string
format_table() {
  local fs="$1" buffer col current_col=0 current_row=0 min=3
  local -a lengths=()

  buffer="$(cat)"

  # First pass to get column lengths
  while read -r line; do
    current_col=0

    while read -r col; do
      lengths["$current_col"]="$(max "${#col}" "${lengths[$current_col]}")"

      current_col=$((current_col + 1))
    done <<< "${line//$fs/$'\n'}"
  done <<< "$buffer"

  # Second pass writes each row
  while read -r line; do
    current_col=0
    current_row=$((current_row + 1))

    while read -r col; do
      printf "| %-$(max "${lengths[$current_col]}" "$min")s " "$col"

      current_col=$((current_col + 1))
    done <<< "${line//$fs/$'\n'}"

    printf "|\n"

    # If this is the first row, print the header dashes
    if [[ "$current_row" -eq 1 ]]; then
      for (( current_col=0; current_col < ${#lengths[@]}; current_col++ )); do
        printf "| "
        printf "%$(max "${lengths[$current_col]}" "$min")s" | tr " " -
        printf " "
      done

      printf "|\n"
    fi
  done <<< "$buffer"
}

# Main program
main() {
  local arg cols i fs="##$$FS##"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      -h | --help) print_help "$1"; return 0 ;;
      -[0-9]*) cols="${1:1}"; shift ;;
      -s*) fs="${1:2}"; shift ;;
      --csv) fs=","; shift ;;
      --tsv) fs=$'\t'; shift ;;
      --) shift; break ;;
      -*) warn "Invalid option '$1'"; return 1 ;;
      *) break ;;
    esac
  done

  if [[ -z "$fs" ]]; then
    warn "Field separator can't be blank!"
    return 1
  elif [[ $# -gt 0 ]] && ! [[ "$cols" =~ ^[0-9]+$ ]]; then
    warn "Missing or Invalid column count!"
    return 1
  fi

  { if [[ $# -gt 0 ]]; then
      while [[ $# -gt 0 ]]; do
        for (( i=0; i < cols; i++ )); do
          if (( i + 1 == cols )); then
            printf "%s" "$1"
          else
            printf "%s%s" "$1" "$fs"
          fi
          shift
        done

        printf "\n"
      done
    else
      cat
    fi
  } | format_table "$fs"
}

main "$@"
