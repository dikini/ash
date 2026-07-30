# Ash

## Overview

Ash is an experimental programming language and runtime. It checks programs before running them.

Ash is released under the MIT or Apache-2.0 license.

## Status

Ash is alpha software. The language, commands, and installation process can change.

## Quick Start

From a checkout, build Ash:

```bash
cargo build --release
```

Install a local Ash toolchain with Ashgrove:

```bash
./target/release/ashgrove install --from source --path . --switch
```

Ashgrove puts `ash` and `ashgrove` in `~/.local/bin`. Add that directory to your `PATH` if needed:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Check an existing example:

```bash
ash check examples/10-testing-helpers/testing_helpers.ash
```

Run a program after saving the example below as `hello.ash`:

```bash
ash run hello.ash
```

You can also use Ash directly from a checkout without installing it:

```bash
cargo run -p ash-cli -- check examples/10-testing-helpers/testing_helpers.ash
cargo run -p ash-cli -- run hello.ash
```

## Examples

This small program exits successfully:

```ash
use result::Result
use runtime::RuntimeError

fn main() -> Result<(), RuntimeError> {
    Ok { value: {} }
}
```

More checked examples are listed in [examples/README.md](examples/README.md).
