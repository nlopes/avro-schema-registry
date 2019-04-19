# Avro Schema Registry

[![Build Status](https://travis-ci.org/nlopes/avro-schema-registry.svg?branch=master)](https://travis-ci.org/nlopes/avro-schema-registry)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/nlopes/avro-schema-registry/blob/master/LICENSE)

**DO NOT USE IN PRODUCTION**

This is an implementation of the Confluent Schema Registry API (mostly) compatible with
the Rails implementation by
[salsify/avro-schema-registry](https://github.com/salsify/avro-schema-registry).

I kept the database schema used by `salsify` and my main goal was to replace the Rails
service with this one. There's nothing wrong with theirs, I just happen to enjoy Rust more
than Ruby. All thanks should go to `salsify` for their initial design and ideas on using
PostgreSQL instead of Kafka as a backend.

Note: the backend used is PostgreSQL (schema compatible with salsify/avro-schema-registry
PostgreSQL schema). The Confluent Schema Registry uses Kafka as a backend therefore we
don't provide exactly the same semantics (but close enough).

## Fingerprint

In [salsify/avro-schema-registry](https://github.com/salsify/avro-schema-registry) two
fingerprints are present (v1 and v2). In this implementation, we only make use of v2 and
do NOT support v1.

If you are still using fingerprints with v1, please make sure you migrate first, before
using this service as your API.

## Endpoints

| Endpoint | Method | Maturity |
|---|---|---|
| `/compatibility/subjects/{subject}/versions/{version}` | POST | Unimplemented |
| `/config` | GET | Ready |
| `/config` | PUT | Ready |
| `/config/{subject}` | GET | Ready |
| `/config/{subject}` | PUT | Ready |
| `/schemas/ids/{id}`| GET | Ready |
| `/subjects` | GET | Ready |
| `/subjects/{subject}` | DELETE | Ready |
| `/subjects/{subject}` | POST | Ready |
| `/subjects/{subject}/versions` | GET | Ready |
| `/subjects/{subject}/versions` | POST | Ready |
| `/subjects/{subject}/versions/latest` | DELETE | Unimplemented |
| `/subjects/{subject}/versions/latest` | GET | Ready |
| `/subjects/{subject}/versions/{version}` | DELETE | Ready |
| `/subjects/{subject}/versions/{version}` | GET | Ready |
| `/subjects/{subject}/versions/latest/schema` | GET | Ready |
| `/subjects/{subject}/versions/{version}/schema` | GET | Ready |

## Extra Endpoints

| Endpoint | Method | Maturity |
|---|---|---|
| `/_/health_check` | GET | Incomplete |
| `/_/metrics` | GET | Unimplemented |


## Build

```
cargo build --release
```

## Run

This assumes you have a running PostgreSQL instance (versions 9.4 and above).

1) Setup env (everything is controlled through environment variables)
```
export SENTRY_URL="http://sentry-url/id" \ # optional
    DEFAULT_HOST=127.0.0.1:8080 \ # optional (default is 127.0.0.1:8080)
    DATABASE_URL=postgres://postgres:@localhost:5432/diesel_testing \
    SCHEMA_REGISTRY_PASSWORD=silly_password
```

2) Run application
```
# If you haven't set PORT, it listens on the default 8080
cargo run # or the binary after running `cargo build`
```
## Tests

### Unit

```
cargo test middleware
```

### Integration

1) Setup testing environment variables
```
export RUST_TEST_THREADS=1 \
    DATABASE_URL=postgres://postgres:@localhost:5432/diesel_testing \
    SCHEMA_REGISTRY_PASSWORD=silly_password
```

2) Run test suite
```
cargo test speculate
```

## Important

We don't ever return `Error code 50003 -- Error while forwarding the request to the
master`. This is because this error is specific to Kafka.

## Contributing

You are more than welcome to contribute to this project. Fork and make a Pull Request, or
create an Issue if you see any problem.
