#!/bin/bash

# This script is mostly copied from here:
# https://github.com/paritytech/polkadot/blob/d822565487bdb41a9e8d73615877a8ea31ae0f6c/scripts/gitlab/trigger_pipeline.sh

set -eu

if [ ! -n "$PIPELINE_TOKEN" ]; then
    echo "PIPELINE_TOKEN is missing!"
    exit 1
fi

# API trigger another project's pipeline
echo "Triggering ink-waterfall pipeline."

echo "https://${CI_SERVER_HOST}/api/v4/projects/${DWNSTRM_ID}/trigger/pipeline"

curl --silent \
    -X POST \
    -F "token=${CI_JOB_TOKEN}" \
    -F "ref=master" \
    -F "variables[TRGR_PROJECT]=${TRGR_PROJECT}" \
    -F "variables[TRGR_REF]=${TRGR_REF}" \
    "https://${CI_SERVER_HOST}/api/v4/projects/${DWNSTRM_ID}/trigger/pipeline" | \
        tee pipeline;

PIPELINE_ID=$(cat pipeline | jq ".id")
PIPELINE_URL=$(cat pipeline | jq ".web_url")
echo
echo "ink-waterfall pipeline ${PIPELINE_URL} was successfully triggered."
echo "Now we're polling it to obtain the distinguished status."

# This is a workaround for a Gitlab bug, waits here until
# https://gitlab.com/gitlab-org/gitlab/-/issues/326137 gets fixed.
# The timeout is 360 curls with 8 sec interval, roughly an hour.

function get_status() {
    curl --silent \
        --header "PRIVATE-TOKEN: ${PIPELINE_TOKEN}" \
        "https://${CI_SERVER_HOST}/api/v4/projects/${DWNSTRM_ID}/pipelines/${PIPELINE_ID}" | \
            jq --raw-output ".status";
}

echo "Waiting on ${PIPELINE_ID} status..."

# How long to sleep in between polling the pipeline status.
POLL_SLEEP=5

# Time until the script exits with 1, if the pipeline isn't finished until then.
TIMEOUT_MINUTES=60

SEQ_END=$(( (TIMEOUT_MINUTES * 60) / POLL_SLEEP ))
for i in $(seq 1 $SEQ_END); do
    STATUS=$(get_status);
    echo "Triggered pipeline status is ${STATUS}";
    if [[ ${STATUS} =~ ^(pending|running|created)$ ]]; then
        echo;
    elif [[ ${STATUS} =~ ^(failed|canceled|skipped|manual)$ ]]; then
        echo "Something's broken in: ${PIPELINE_URL}"; exit 1;
    elif [[ ${STATUS} =~ ^(success)$ ]]; then
        echo "Look how green it is: ${PIPELINE_URL}"; exit 0;
    else
        echo "Something else has happened in ${PIPELINE_URL}"; exit 1;
    fi
sleep 5;
done

exit 1
