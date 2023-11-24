#!/bin/bash

# Accepts a diff of contract sizes and posts the to the PR as a comment.
#
# Usage:
#   ./contract_sizes_submit.sh \
#     <github_url_to_comments_of_pr>
#     <github_url_to_workflow>
#     <cargo_contract_version>
#     < <diffs-csv-file>

set -eu
set -o pipefail

pr_comments_url=$1
workflow_url=$2
cc_version=$3
diffs_markdown_table=$(</dev/stdin)

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

ink_master_head=$(curl -s "https://api.github.com/repos/paritytech/ink/commits/master" | jq -r .sha)
head_in_branch=$(git log | grep -q $ink_master_head; echo $?)

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

${diffs_markdown_table}

[Link to the run](${workflow_url}) | Last update: ${updated}
EOF
)
json_body=$(jq -n --arg body "${body}" '{ "body": $body}')

curl -X ${verb} ${pr_comments_url} \
    -H "Cookie: logged_in=no" \
    -H "Authorization: token ${GITHUB_PR_TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d "${json_body}"
