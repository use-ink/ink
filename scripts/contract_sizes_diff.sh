#!/bin/bash

upstream=$1
pr_branch=$2

awk '
NR==FNR {
  a[$1]=$2; next
}
$1 in a {
  print "|" $1 "|" a[$1] "|" $2 "|" (($2 - a[$1]) / a[$1]) * 100 "|"
}' "$upstream" "$pr_branch"
