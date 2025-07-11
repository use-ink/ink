#!/bin/bash

csv=$1

echo '```mermaid'
echo 'gantt'
echo '   dateFormat x'
echo '   axisFormat x'

cat $csv | while read line;
do
    section=$(echo "$line" | cut -d ";" -f1)
    master_default_abi=$(echo "$line" | cut -d ";" -f2)
    pr_default_abi=$(echo "$line" | cut -d ";" -f3)
    pr_sol_abi=$(echo "$line" | cut -d ";" -f4)
    pr_all_abi=$(echo "$line" | cut -d ";" -f5)
    v5_default_abi=$(echo "$line" | cut -d ";" -f6)

    echo "section  "
    echo "$section : a5, 0, 0"
    echo "v5 default abi - $v5_default_abi kb       : a1, 0, $v5_default_abi"
    echo "master default abi - $master_default_abi kb       : a1, 0, $master_default_abi"
    echo "pr default abi - $pr_default_abi kb      : a1, 0, $pr_default_abi"
    echo "pr sol abi - $pr_sol_abi kb      : a2, 0, $pr_sol_abi"
    echo "pr all abi - $pr_all_abi kb     : a3, 0, $pr_all_abi"
    echo "#nbsp; : a4, 0, 0"
    echo ""
done

echo '```'
