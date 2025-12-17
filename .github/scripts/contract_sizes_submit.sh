#!/bin/bash

# Accepts a diff of contract sizes and posts them to the PR as a comment.
#
# Usage:
#   ./contract_sizes_submit.sh \
#     <github_url_to_comments_of_pr>
#     <github_url_to_workflow>
#     <head_in_branch>
#     <contract-sizes-diffs-csv-file>
#     <abi-contract-sizes-diffs-csv-file>

set -eu
set -o pipefail

pr_comments_url=$1
workflow_url=$2
head_in_branch=$3
master_v_pr_diffs_markdown_table=$(cat $4)
master_v_pr_abi_sol_diffs_markdown_table=$(cat $5)
master_v_pr_abi_all_diffs_markdown_table=$(cat $6)
pr_abi_ink_v_sol_diffs_markdown_table=$(cat $7)
pr_abi_ink_v_all_diffs_markdown_table=$(cat $8)
mermaid_diagram=$(cat $9)

# If there is already a comment by the user `github-actions[bot]` in the ink! PR which triggered
# this run, then we can just edit this comment (using `PATCH` instead of `POST`).
echo "pr_comments_url: " $pr_comments_url
possibly_comment_url=$(curl --silent $pr_comments_url | \
  jq -r ".[] | select(.user.login == \"github-actions[bot]\") | .url" | \
  head -n1
)
echo "possibly_comment_url: " $possibly_comment_url

verb="POST"
if [ ! -z "$possibly_comment_url" ]; then
   verb="PATCH";
   pr_comments_url="$possibly_comment_url"
fi

echo $verb
echo $pr_comments_url

master_ahead=""
if [ "$head_in_branch" == "1" ]; then
  echo "ink! master is ahead"
  master_ahead="⚠️ **The ink! \`master\` is ahead of your branch, this might skew the comparison data below.** ⚠️"
fi

updated=$(TZ='Europe/Berlin' date)
body=$(cat << EOF
## 🦑 📈 ink! Example Contracts ‒ Changes Report 📉 🦑
${master_ahead}
These are the results when building the \`integration-tests/*\` contracts from this branch and comparing them to ink! \`master\`:

<details><summary>Show contract sizes: PR vs master</summary>

Using the ABI denoted in the contract manifest.

${master_v_pr_diffs_markdown_table}

</details>

<details><summary>Show diagram</summary>

${mermaid_diagram}

</details>

<details><summary>Show contract sizes: PR vs master (with enforced Solidity ABI)</summary>

The following table shows how the contract sizes _on this branch_ change compared to the \`master\` branch when using the _Solidity ABI_.

${master_v_pr_abi_sol_diffs_markdown_table}

</details>

<details><summary>Show contract sizes: PR vs master (with enforced all ABI mode)</summary>

The following table shows how the contract sizes _on this branch_ change compared to the \`master\` branch when using the _all ABI_ mode.

${master_v_pr_abi_all_diffs_markdown_table}

</details>

<details><summary>Show contract sizes: ink! vs Solidity ABI</summary>

The following table shows how the contract sizes _on this branch_ change when choosing the _Solidity ABI_ instead of the ink! ABI.

${pr_abi_ink_v_sol_diffs_markdown_table}

</details>

<details><summary>Show contract sizes: ink! vs all ABI mode</summary>

The following table shows how the contract sizes _on this branch_ change when choosing the _all ABI_ mode instead of the ink! ABI.

${pr_abi_ink_v_all_diffs_markdown_table}

</details>

[Link to the run](${workflow_url}) | Last update: ${updated}
EOF
)
echo "body: " $body
json_body=$(jq -n --arg body "${body}" '{ "body": $body}')

curl -X ${verb} ${pr_comments_url} \
    -H "Cookie: logged_in=no" \
    -H "Authorization: token ${GITHUB_PR_TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d "${json_body}"
