# rush-booking

The Booking API using Rust

## Installation

------------

### Pre-requisites

You'll need to install:

- [Rust](https://www.rust-lang.org/tools/install)

### Init local db

```bash
# To init db and run migration
./scripts/init-db.sh

# or only run migration
SKIP_DOCKER=true ./scripts/init_db.sh
```

### Build the project

To build the project, run:

```bash
cargo build
```

To build the release, check `justfile`

```bash
cargo build --release
```

### Running tests

You can run tests using the following command:

```bash
cargo test
```

### Generate code reports

You can run code reports using the following command:

```bash
./scripts/init_code_report_cov.sh
```

Send reports to sonarqube local
- [Check this](./sonarqube-local/README.md)


### F.A.Qs
1. Fix `too many open files` error on MacOS
```bash
ulimit -n X (X is the number of open files)
```
