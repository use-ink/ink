#!/bin/bash

# Accepts a diff of contract sizes and posts them to the PR as a comment.
#
# Usage:
#   ./contract_sizes_submit.sh \
#     <github_url_to_comments_of_pr>
#     <github_url_to_workflow>
#     <cargo_contract_version>
#     <head_in_branch>
#     <contract-sizes-diffs-csv-file>
#     <abi-contract-sizes-diffs-csv-file>

set -eu
set -o pipefail

pr_comments_url=$1
workflow_url=$2
cc_version=$3
head_in_branch=$3
diffs_markdown_table=$(cat $4)
abi_diffs_markdown_table=$(cat $5)

# If there is already a comment by the user `github-actions[bot]` in the ink! PR which triggered
# this run, then we can just edit this comment (using `PATCH` instead of `POST`).
possibly_comment_url=$(curl --silent $pr_comments_url | \
  jq -r ".[] | select(.user.login == \"github-actions[bot]\") | .url" | \
  head -n1
)
echo $possibly_comment_url

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
These are the results when building the \`integration-tests/*\` contracts from this branch with \`${cc_version}\` and comparing them to ink! \`master\`:

<details><summary>Show Results</summary>

${diffs_markdown_table}

</details>

These are the results when building the \`integration-tests/*\` contracts from this branch with \`${cc_version}\` and comparing them to the same contract build with the same toolchain, but with enforced Solidity ABI:

<details><summary>Show contract size for ABI ink! vs Solidity</summary>

${abi_diffs_markdown_table}

</details>


[Link to the run](${workflow_url}) | Last update: ${updated}
EOF
)
json_body=$(jq -n --arg body "${body}" '{ "body": $body}')

curl -X ${verb} ${pr_comments_url} \
    -H "Cookie: logged_in=no" \
    -H "Authorization: token ${GITHUB_PR_TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d "${json_body}"
