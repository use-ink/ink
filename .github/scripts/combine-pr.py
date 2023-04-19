import uuid
import json
import glob
import sys
import os


def set_output(name, value):
    with open(os.environ['GITHUB_OUTPUT'], 'a') as fh:
        print(f'{name}={value}', file=fh)


def set_multiline_output(name, value):
    with open(os.environ['GITHUB_OUTPUT'], 'a') as fh:
        delimiter = uuid.uuid1()
        print(f'{name}<<{delimiter}', file=fh)
        print(value, file=fh)
        print(delimiter, file=fh)


if len(sys.argv) > 1:
    limit = int(sys.argv[1])
else:
    limit = 999

result = []
for f in glob.glob("outputs/*.json"):
    with open(f, "r") as infile:
        result += json.load(infile)

sorted_data = sorted(result, key=lambda x: x['daysStale'])
sorted_data.reverse()

count = len(sorted_data)

set_output("COUNT", count)

if count < 1:
    set_output("MESSAGE", "")
    exit()

sliced_list = sorted_data[:limit]

formated_list = []
for i, data in enumerate(sliced_list):
    message = str(
        f' {i + 1}. [{data["title"]}]({data["url"]}) | {data["daysStale"]} days with no reviews')
    formated_list.append(message)

with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as fh:
    print(f'## There are {count} stale PRs', file=fh)
    for message in formated_list:
        print(f'{message}', file=fh)

message = "\n".join(formated_list)

set_multiline_output("MESSAGE", message)

set_output("data", json.dumps(formated_list))
