# DNS Smart contract

The DNS smart contract is our showcase for decentralizing the Domain naming system. 

   

> This contract compiles with ink! 4.0-beta version

Domain name service contract inspired by
    [this blog post on medium](https://medium.com/@chainx_org/secure-and-decentralized-polkadot-domain-name-system-e06c35c2a48d).

### Description
The registration of a new Domain works by the function `register` defined in the contract which takes 2 parameters: name and hash. Client sends a request to the DNS system such as "polka.dot" which returns the resolver address as shown below. The "polka.dot" is resolved by mapping the it to a `hash value` from the function named `Get_address` in the `lib.rs` file. 

<br>
<br>

![Image](images/dns_diagram.png)

## Functionalities provided: 

- Registration of a new Domain Name
- Transfer the (already exsiting) owner to new Address
- Change the Domain Name



 