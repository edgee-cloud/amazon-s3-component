<div align="center">
<p align="center">
  <a href="https://www.edgee.cloud">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://cdn.edgee.cloud/img/component-dark.svg">
      <img src="https://cdn.edgee.cloud/img/component.svg" height="100" alt="Edgee">
    </picture>
  </a>
</p>
</div>

<h1 align="center">Amazon S3 component for Edgee</h1>

This component enables seamless integration between [Edgee](https://www.edgee.cloud) and [Amazon S3](https://aws.amazon.com/s3/), allowing you to collect and forward analytics events to your data lake on S3.


## Quick Start

1. Download the latest component version from our [releases page](../../releases)
2. Place the `s3.wasm` file in your server (e.g., `/var/edgee/components`)
3. Add the following configuration to your `edgee.toml`:

```toml
[[destinations.data_collection]]
id = "amazon-s3"
file = "/var/edgee/components/s3.wasm"
settings.aws_access_key = "YOUR_AWS_ACCESS_KEY"
settings.aws_secret_key = "YOUR_AWS_SECRET_KEY"
settings.aws_region = "YOUR_AWS_REGION"
settings.s3_bucket = "YOUR_S3_BUCKET_NAME"
```


## Event Handling

### Event Mapping
The component maps Edgee events to S3 objects as follows:

| Edgee Event | S3 object | Description |
|-------------|----------------|-------------|
| Page        | `{bucket}/{prefix}{random-key}.json` | Full JSON dump of the Page event |
| Track       | `{bucket}/{prefix}{random-key}.json` | Full JSON dump of the Track event |
| User        | `{bucket}/{prefix}{random-key}.json` | Full JSON dump of the User event |


## Configuration Options

### Basic Configuration
```toml
[[destinations.data_collection]]
id = "amazon-s3"
file = "/var/edgee/components/s3.wasm"
settings.aws_access_key = "YOUR_AWS_ACCESS_KEY"
settings.aws_secret_key = "YOUR_AWS_SECRET_KEY"
settings.aws_region = "YOUR_AWS_REGION"
settings.s3_bucket = "YOUR_S3_BUCKET_NAME"

# Optional configurations
settings.aws_session_token = "YOUR_AWS_SESSION_TOKEN" # Useful for tests, not recommended in prod since it's short-lived
settings.s3_key_prefix = "sub-folder/" # Optional prefix for all S3 objects
```


### Event Controls
Control which events are forwarded to S3:
```toml
settings.edgee_page_event_enabled = true   # Enable/disable page view tracking
settings.edgee_track_event_enabled = true  # Enable/disable custom event tracking
settings.edgee_user_event_enabled = true   # Enable/disable user identification
```


## Development

### Building from Source
Prerequisites:
- [Rust](https://www.rust-lang.org/tools/install)
- wit-deps: `cargo install wit-deps`

Build command:
```bash
edgee component build
```

Test command:
```bash
make test
```

Test coverage command:
```bash
make test.coverage[.html]
```

### Contributing
Interested in contributing? Read our [contribution guidelines](./CONTRIBUTING.md)

### Security
Report security vulnerabilities to [security@edgee.cloud](mailto:security@edgee.cloud)
