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


<h1 align="center">Segment Component for Edgee</h1>

This component implements the data collection protocol between [Edgee](https://www.edgee.cloud) and [Segment](https://segment.com).

## Quick Start

1. Download the latest component version from our [releases page](../../releases)
2. Place the `segment.wasm` file in your server (e.g., `/var/edgee/components`)
3. Add the following configuration to your `edgee.toml`:

```toml
[[destinations.data_collection]]
name = "segment"
component = "/var/edgee/components/segment.wasm"
credentials.segment_api_key = "..."
```

## Event Handling

### Event Mapping
The component maps Edgee events to Segment events as follows:

| Edgee Event | Segment Event  | Description |
|-------------|--------------|-------------|
| Page        | `page`  | Triggered when a user views a page |
| Track       | `track` | Uses the provided event name directly |
| User        | `identify` | Used for user identification only |

### User Event Handling
Each time you make a `user` call, Edgee will send an `identify` event to Segment.

But when you make a `user` call using Edgee's JS library or Data Layer, the `user_id`, `anonymous_id` and `properties` are stored in the user's device.
This allows the user's data to be added to any subsequent page or follow-up calls for the user, so that you can correctly attribute these actions.

## Configuration Options

### Basic Configuration
```toml
[[destinations.data_collection]]
name = "segment"
component = "/var/edgee/components/segment.wasm"
credentials.segment_api_key = "..."

# Optional configurations
config.anonymization = true        # Enable/disable data anonymization in case of pending or denied consent
config.default_consent = "pending" # Set default consent status if not specified by the user
```

### Event Controls
Control which events are forwarded to Segment:
```toml
config.page_event_enabled = true   # Enable/disable page event
config.track_event_enabled = true  # Enable/disable track event
config.user_event_enabled = true   # Enable/disable user event
```

### Consent Management
Before sending events to Segment, you can set the user consent using the Edgee SDK: 
```javascript
edgee.consent("granted");
```

Or using the Data Layer:
```html
<script id="__EDGEE_DATA_LAYER__" type="application/json">
  {
    "data_collection": {
      "consent": "granted"
    }
  }
</script>
```

If the consent is not set, the component will use the default consent status.

| Consent | Anonymization | 
|---------|---------------|
| pending | true          |
| denied  | true          |
| granted | false         |

## Development

### Building from Source
Prerequisites:
- [Rust](https://www.rust-lang.org/tools/install)
- WASM target: `rustup target add wasm32-wasip2`
- wit-deps: `cargo install wit-deps`

Build command:
```bash
make wit-deps
make build
```

### Contributing
Interested in contributing? Read our [contribution guidelines](./CONTRIBUTING.md)

### Security
Report security vulnerabilities to [security@edgee.cloud](mailto:security@edgee.cloud)