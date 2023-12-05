# Interest management

A simple example that shows how to use Lightyear to perform interest management.

Interest management is a technique to reduce the amount of data that is sent to each client:
we want to send only the data that is relevant to each client.

In this example, we are going to replicate entities that are within a certain distance of the client.



## Running the example

To start the server, run `cargo run --example interest_management -- server`

Then you can launch multiple clients with the commands:

- `cargo run --example interest_management -- client -c 1`

- `cargo run --example interest_management -- client -c 2 --client-port 2000`