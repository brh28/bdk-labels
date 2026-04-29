# bdk-labels

A CLI tool for labeling Bitcoin data. 

## Features

- Store annotative data in a local SQLite database (`~/.bdk-bitcoin/bdk-labels.db`), including:
    - Descriptions (`bdk-label describe --tx <txid> "from Satoshi, for Pizza"`) 
    - Tags (`bdk-label tag --output <txid:0> 'kyc' 'cost_basis:20260428', 'credit:income:pizzashop'`) 
    - Spendable
- Query and filter by type, wallet, origin, or tag (`bdk-labels list --output --tag 'credit:income'`)
- Supports multiple wallets & public types
- Import & Export BIP329 JSONL from a file or stdin

## Installation

```sh
cargo install --path .
```

## Usage

```
bdk-label [OPTIONS] <COMMAND>

Options:
    --db <PATH>    Path to the labels database [env: BDK_LABELS_DB]

Commands:
    import    Import BIP329 JSONL from stdin or a file
    add       Create a new annotation
    describe  Set the description (and optionally origin) on an annotation
    tag       Add tags to an annotation
    rm        Remove tags from an annotation, or delete it entirely with --force
    export    Export annotations as BIP329 JSONL
    list      Query and list stored annotations
```

### Import

```sh
# Import from a file (wallet name defaults to filename stem)
bdk-label import --file wallet.jsonl

# Import with an explicit wallet name
bdk-label import --file wallet.jsonl --wallet savings

# Import from stdin (--wallet required)
cat wallet.jsonl | bdk-label import --wallet savings
```

### Add

Manually create an annotation for any Bitcoin reference. `--wallet` is required.

```sh
bdk-label add --addr bc1q... --wallet savings --description "cold storage"
bdk-label add --tx <txid> --wallet savings --description "exchange withdrawal" --tag exchange:kraken
bdk-label add --output <txid:vout> --wallet savings --spendable
```

### Describe

Update the description on an existing annotation.

```sh
bdk-label describe --tx <txid> "payment to vendor"
bdk-label describe --addr bc1q... "donation address" --origin "wpkh([deadbeef/84h/0h/0h])"
```

### List

```sh
# List all annotations
bdk-label list

# Filter by type
bdk-label list --tx
bdk-label list --addr

# Filter by wallet or origin
bdk-label list --wallet savings
bdk-label list --origin "wpkh([deadbeef/84h/0h/0h])"

# Filter by tag (repeatable; all must match)
bdk-label list --tag kyc: --tag exchange:kraken
```

## Tags

Tags use a colon-separated hierarchical format:

| Token | Meaning |
|---|---|
| `kyc:` | Boolean tag (single segment) |
| `exchange:kraken` | Key-value tag |
| `debit:expenses:food` | Hierarchical tag |

Tags are serialized into the BIP329 `label` field alongside the description, separated by ` ; `:

```
payment to vendor ; exchange:kraken debit:expenses:food
```

## Storage

The default database is `~/.bdk-bitcoin/bdk-labels.db`. Override with `--db` or `BDK_LABELS_DB`:

```sh
BDK_LABELS_DB=/path/to/labels.db bdk-label list
```
