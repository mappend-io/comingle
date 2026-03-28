# Usage

```
Usage: comingle [OPTIONS] --layer-config-uri <LAYER_CONFIG_URI>

Options:
      --log-level <LOG_LEVEL>
          Log level [env: RUST_LOG=] [default: comingle=info]
      --pretty-log
          Use pretty logging instead of JSON [env: PRETTY_LOG=true]
      --listen-addr <LISTEN_ADDR>
          Listen address [env: LISTEN_ADDR=] [default: 0.0.0.0:3200]
      --cors-origin <CORS_ORIGIN>
          Allow CORS from a specific origin, or "*" for any [env: CORS_ORIGIN=*]
      --layer-config-uri <LAYER_CONFIG_URI>
          Location of layer configuration JSON documents [env: LAYER_CONFIG_URI=]
      --layer-definition-ttl <LAYER_DEFINITION_TTL>
          [env: LAYER_DEFINITION_TTL=] [default: 5m]
  -h, --help
          Print help
```

# Configuration

The options named in the usage section above may be specified on the command
line, the environment or a .env file.
