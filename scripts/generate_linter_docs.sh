#!/usr/bin/env bash

set -eu

usage() {
  echo "Usage: $0 PATH"
  echo "Creates or updates the documentation for ink_linting hosted at use.ink."
  echo
  echo "Arguments:"
  echo "  PATH Path to the local clone of https://github.com/paritytech/ink-docs"
  echo
  echo "Examples:"
  echo "  $0 ~/ink-docs"
}

if [ ! $# -eq 1 ]; then
    usage
    exit 1
fi

ink_docs_dir="$1"
tmp_file=$(mktemp /tmp/linting_docs.XXXXXX.md)
had_update=0
for lint_src in linting/{mandatory,extra}/src/*.rs; do
    lint_name="$(basename "${lint_src%.*}")"
    lint_doc_file="$ink_docs_dir/docs/linter/rules/$lint_name.md"
    lint_doc_filestring="$(sed -n '/^declare_lint! {$/,/^}$/p' "$lint_src")"
    [[ -z "$lint_doc_filestring" ]] && continue

    # Save the extracted documentation to the temporary file
    truncate -s 0 "$tmp_file"
    {
        echo "---"
        echo "title: $lint_name"
        echo "hide_title: true"
        echo "description: $lint_name lint documentation"
        echo "---"
        echo "# $lint_name"
        printf "%s" "$lint_doc_filestring" | sed -n 's/^\s*\/\/\/\s\?\(.*\)$/\1/;T;p'
    } >> "$tmp_file"

    if [[ ! -f "$lint_doc_file" ]]; then
        echo "$lint_name: created documentation"
        had_update=1
        mv "$tmp_file" "$lint_doc_file"
        continue
    fi

    if cmp -s "$tmp_file" "$lint_doc_file"; then
        echo "$lint_name: no changes"
    else
        echo "$lint_name: updated"
        mv "$tmp_file" "$lint_doc_file"
    fi
done

[[ $had_update -eq 1 ]] && echo -e "\nPlease add created documentation sections to $ink_docs_dir/sidebars.js"
