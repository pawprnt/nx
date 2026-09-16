# nx

A nix helper CLI for NixOS.

## Features

- **`nx os rebuild`** — Build and activate system configuration
- **`nx os rollback`** — Roll back to a previous generation
- **`nx os info`** — Show information about current and past generations
- **`nx os delete`** — Delete specific generations by ID
- **`nx clean`** — Garbage collect old generations and optimise the store
- **`nx search`** — Search for packages in nixpkgs

## Installation

```bash
nix build .#default
# or
nix profile install .#default
```

## Usage

```bash
# Rebuild system
nx os rebuild

# Rebuild with flake update
nx os rebuild --update

# Show generations
nx os info

# Delete specific generations
nx os delete 80 81 82

# Interactive deletion
nx os delete

# Garbage collect
nx clean

# Search for packages
nx search firefox
```

## Configuration

Set `NX_FLAKE` to your flake directory (defaults to `~/.dotfiles`):

```bash
export NX_FLAKE="~/.dotfiles"
```

## License

MIT
