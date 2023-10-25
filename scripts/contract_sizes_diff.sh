#!/bin/bash

script_name="${BASH_SOURCE[0]}"
scripts_path=$( cd "$(dirname "$script_name")" || exit; pwd -P )

upstream=$1
pr_branch=$2

csv=$(awk '
NR==FNR {
  a[$1]=$2; next
}
BEGIN{print "Contract,Upstream Size (kB),PR Size (kB),Diff (kB),Diff (%),Change"};
$1 in a {
  name=$1
  up_kb=a[$1]/1000
  pr_kb=$2/1000
  diff_kb=pr_kb - up_kb
  diff_pc=(diff_kb / up_kb) * 100

  if (diff_kb > 0) {
    change=":chart_with_upwards_trend:"
  } else if (diff_kb < 0) {
    change=":chart_with_downwards_trend:"
  } else {
    change=":heavy_minus_sign:"
  }

  print name","up_kb","pr_kb","diff_kb","diff_pc","change
}' "$upstream" "$pr_branch")

"${scripts_path}"/markdown-table.sh --csv <<< "${csv}"
