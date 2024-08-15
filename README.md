# Getting started
Run cargo run from your terminal. This will open a listening UDP socket on port 2053.
In a separate terminal, send a dig request to the server:
```sh
 dig @127.0.0.1 -p 2053 +noedns google.com
```