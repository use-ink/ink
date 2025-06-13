import uuid
import json
import glob
import sys
import os

# This script does the following:
# 1. combines all of the jsons in `outputs/*.json`
# 2. orders all the outputs based on the parameter `daysStale`
# 3. reverses the list
# 4. counts the amount of elements and generates an output
# 5. limits the list to the given number (15 in this PR)
# 6. formats the strings with the given template (the for loop above)
# 7. pushes the result as a summary and an output for another github action to use


def set_output(name, value):
    ''''Set an output variable for this action'''
    with open(os.environ['GITHUB_OUTPUT'], 'a') as fh:
        print(f'{name}={value}', file=fh)


def set_multiline_output(name, value):
    ''''Set a multiline output variable for this action'''
    with open(os.environ['GITHUB_OUTPUT'], 'a') as fh:
        delimiter = uuid.uuid1()
        print(f'{name}<<{delimiter}', file=fh)
        print(value, file=fh)
        print(delimiter, file=fh)


# we capture the array limit from the command line
if len(sys.argv) > 1:
    limit = int(sys.argv[1])
else:
    limit = 999

# we combine all the jsons in the outputs folder
result = []
for f in glob.glob("outputs/*.json"):
    with open(f, "r") as infile:
        result += json.load(infile)

# we sort the list based on the daysStale key and then reverse it
sorted_data = sorted(result, key=lambda x: x['daysStale'])
sorted_data.reverse()

# we count the amount of elements and generate an output
count = len(sorted_data)

set_output("COUNT", count)

# if there are no elements, we exit
if count < 1:
    set_output("MESSAGE", "")
    exit()

# we limit the list to the given number
sliced_list = sorted_data[:limit]

# we format the strings with the given template
formated_list = []
for i, data in enumerate(sliced_list):
    message = str(
        # here is the template. If you want to modify it you can do so here
        f' {i + 1}. [{data["title"]}]({data["url"]}) | {data["daysStale"]} days with no reviews')
    formated_list.append(message)

# we push the result as a summary
with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as fh:
    print(f'## There are {count} stale PRs', file=fh)
    for message in formated_list:
        print(f'{message}', file=fh)

# we push the result as an output for another github action to use
message = "\n".join(formated_list)

set_multiline_output("MESSAGE", message)

set_output("data", json.dumps(formated_list))
