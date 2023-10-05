#!/bin/bash

script_name="${BASH_SOURCE[0]}"
scripts_path=$( cd "$(dirname "$script_name")" || exit; pwd -P )

upstream=$1
pr_branch=$2

csv=$(awk '
NR==FNR {
  a[$1]=$2; next
}
BEGIN{print "Contract,Optimized Size (Upstream),Optimized Size (PR),Diff (kB),Diff (%)"};
$1 in a {
  name=$1
  up_kb=a[$1]/1000
  pr_kb=$2/1000
  diff_kb=pr_kb - up_kb
  diff_pc=(diff_kb / up_kb) * 100

  print name","up_kb","pr_kb","diff_kb","diff_pc
}' "$upstream" "$pr_branch")

"${scripts_path}"/markdown-table.sh --csv <<< "${csv}"
