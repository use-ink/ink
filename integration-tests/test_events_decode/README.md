# Testing Events Reference Implementation

Capturing events for ink! end-to-end tests in particular is not clearly documented, nor are there clear reference implementations. This contract contains a minimalist flipflop contract that emits two different events when called. It contains both unit and e2e test modules illustrating one way to check for multiple events in both testing environments.

The end-to-end event test/check is awkward and could certainly be improved on. This reference may not be appropriate for it's own example project in the ink-examples repo, so it should be up to this repo's maintainers/seasoned-folk to figure out where to put it, if at all.

Notes:

- e2e: depending on the field number and content of an event struct, the byte range in the `&event` that is decoded may be either `&event[34..]` or `&event[35..]`. This is a point to improve on, dealing with the prepending bytes that don't pertain to the field values.

- There is probably a way to filter for event types simultaneously, instead of one at a time (that is, keeping track of which events were caught by using a boolean and assert).
