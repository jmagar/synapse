# synapse-rmcp

Node launcher for the `synapse` Rust MCP server and CLI binary.

```bash
npx -y synapse-rmcp --help
```

The package downloads the matching GitHub Release binary during `postinstall`.

## MCP stdio

Use the package directly as an MCP command:

```json
{
  "mcpServers": {
    "synapse2": {
      "command": "npx",
      "args": ["-y", "synapse-rmcp"]
    }
  }
}
```

## Environment

- `SYNAPSE_RMCP_BINARY_VERSION`: release tag/version to download, defaulting to this npm package version.
- `SYNAPSE_RMCP_VERSION`: alias for `SYNAPSE_RMCP_BINARY_VERSION`.
- `SYNAPSE_RMCP_REPO`: GitHub `owner/repo`, defaulting to `jmagar/synapse2`.
- `SYNAPSE_RMCP_RELEASE_BASE_URL`: full release download base URL.
- `SYNAPSE_RMCP_SKIP_DOWNLOAD=1`: skip postinstall download.
