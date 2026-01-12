# fxdump

A command-line tool for dumping FX (effects) information from Reaper via OSC.

## Overview

`fxdump` queries Reaper for information about all installed FX and writes the data to a YAML file. This tool is useful for documenting available effects and their parameters.

## Building

From the repository root:

```bash
cargo build --package fxdump --release
```

The binary will be located at `target/release/fxdump`.

## Usage

1. Ensure Reaper is running with the reaper-arpad extension loaded
2. Verify that OSC communication is configured with:
   - Reaper listening on `0.0.0.0:9091`
   - Tool listening on `0.0.0.0:9090`
3. Run the tool:

```bash
cargo run --package fxdump
```

Or if you've built the release binary:

```bash
./target/release/fxdump
```

The tool will:
1. Send an OSC query to `/fxinfo/?`
2. Listen for OSC responses on addresses prefixed with `/fxinfo/...`
3. Collect and parse FX information
4. Write the results to `fx_dump.yaml`

## Output Format

The output YAML file contains an array of FX, each with:

```yaml
- fx_name: "Effect Name"
  params:
    - name: "Parameter Name"
      index: 0
      min: 0.0
      max: 1.0
```

## Configuration

The tool uses the following OSC addresses (configured in the source code):

- `HOST_ADDR`: `0.0.0.0:9090` (where the tool listens)
- `DEVICE_ADDR`: `0.0.0.0:9091` (where Reaper listens)
- `TIMEOUT_SECS`: 5 seconds (how long to wait for responses)

## Error Handling

The tool will panic with descriptive error messages if:
- Socket binding fails
- OSC message encoding fails
- OSC message sending fails
- File writing fails

This ensures that issues are immediately visible and can be debugged.

## Notes

- The tool is designed for one-time execution and exits after producing its output
- Response collection uses a timeout mechanism to determine when all data has been received
- FX are sorted alphabetically in the output for consistency
