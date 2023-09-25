#!/bin/bash

upstream=$1
pr_branch=$2

awk '
NR==FNR {
  a[$1]=$2; next
}
$1 in a {
  name=$1
  up_kb=a[$1]/1000
  pr_kb=$2/1000
  diff=pr_kb - up_kb
  diff_pc=(diff / up_kb) * 100

  print "|" name "|" up_kb "|" pr_kb "|" diff_pc "|"
}' "$upstream" "$pr_branch"
