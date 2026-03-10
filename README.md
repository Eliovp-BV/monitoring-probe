# monitoring-probe

Minimal Rust HTTP service for Kubernetes monitoring.

It exposes one endpoint:

- `GET /health`

Sample configuration lives in `config.sample.yaml`.

For local runs, create `config.yaml` from the sample. Configuration is loaded from YAML via `CONFIG_PATH` (default: `config.yaml`):

```yaml
services:
  alias1:
    service: "service:80"
    checkforstatus: 200
    shouldcontain: "somestring"
  alias2:
    service: "anotherservice:80"
    checkforstatus: 200
```

If `service` has no scheme, the probe requests `http://<service>/`.

Example response:

```json
{
  "alias1": "ok",
  "alias2": "error"
}
```

Run locally:

```bash
cargo run
```

Build container:

```bash
docker build -t monitoring-probe .
```
