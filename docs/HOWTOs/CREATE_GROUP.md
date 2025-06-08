# create-group

To create a group chat using Mls protocol, there are three stages:

1. Setup - Register to the server
2. Handshake - Establish group state
3. Exchange - Message

## Setup

------------

### Pre-requisites

You'll need to install:

- [Telnet Client](https://webhostinggeeks.com/howto/how-to-install-telnet-on-windows-macos-linux/) - Client UI
- [Rust](https://www.rust-lang.org/tools/install) - Programming Language

> **_NOTE:_** We use port `3456` to connect to the server
