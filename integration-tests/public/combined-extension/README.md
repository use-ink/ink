# Combined Chain Extension Example

## What is this example about?

It demonstrates how to combine several chain extensions and call them from ink!.

See [this chapter](https://use.ink/macros-attributes/chain-extension)
in our ink! documentation for more details about chain extensions.


This example uses two chain extensions, `FetchRandom`(from [rand-extension](../rand-extension)) 
and `Psp22Extension`(from [psp22-extension](../psp22-extension)) defined in other examples. 
The example shows how to combine two chain extensions together 
and use them in the contract along with each other. 
Also example shows how to mock each chain extension for testing.

This example doesn't show how to define a chain extension and how 
to implement in on the runtime side. For that purpose, you can 
check the two examples mentioned above.
