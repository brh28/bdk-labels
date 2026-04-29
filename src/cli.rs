use clap::{Args, Parser, Subcommand};

use crate::annotation::RecordType;

#[derive(Parser)]
#[command(name = "bdk-label", about = "Manage BIP329 wallet labels")]
pub struct Cli {
    /// Path to the labels database (default: ~/.bdk-bitcoin/bdk-labels.db)
    #[arg(long, env = "BDK_LABELS_DB", value_name = "PATH")]
    pub db: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Import BIP329 JSONL from stdin or a file
    Import {
        /// Path to JSONL file; reads stdin if omitted
        #[arg(short, long)]
        file: Option<std::path::PathBuf>,

        /// Wallet name to associate with all imported annotations;
        /// defaults to the filename stem when --file is provided,
        /// required when reading from stdin
        #[arg(short, long)]
        wallet: Option<String>,
    },

    /// Create a new annotation
    Add {
        #[command(flatten)]
        annotation: AnnotationRef,

        /// Wallet to associate this annotation with
        #[arg(short, long)]
        wallet: String,

        /// Human-readable description
        #[arg(short, long)]
        description: Option<String>,

        /// Wallet origin (BIP32 descriptor or fingerprint)
        #[arg(long)]
        origin: Option<String>,

        /// Tags in key: or key:value form (repeatable)
        #[arg(long = "tag", value_name = "TAG")]
        tags: Vec<String>,

        /// Mark output as spendable
        #[arg(long, group = "spendable_flag")]
        spendable: bool,

        /// Mark output as not spendable
        #[arg(long, group = "spendable_flag")]
        not_spendable: bool,
    },

    /// Set the description (and optionally origin) on an annotation
    Describe {
        #[command(flatten)]
        annotation: AnnotationRef,

        /// Human-readable description
        description: String,

        /// Wallet origin (BIP32 descriptor or fingerprint)
        #[arg(long)]
        origin: Option<String>,
    },

    /// Add tags to an annotation
    Tag {
        #[command(flatten)]
        annotation: AnnotationRef,

        /// Tags in key: or key:value form
        #[arg(value_name = "TAG", required = true)]
        tags: Vec<String>,
    },

    /// Remove tags from an annotation, or delete it entirely with --force
    Rm {
        #[command(flatten)]
        annotation: AnnotationRef,

        /// Tags to remove; omit to delete the entire annotation (requires --force)
        #[arg(value_name = "TAG")]
        tags: Vec<String>,

        /// Required when deleting an entire annotation
        #[arg(long, short)]
        force: bool,
    },

    /// Export annotations as BIP329 JSONL
    Export {
        #[command(flatten)]
        filter: AnnotationFilter,
    },

    /// Query and list stored annotations
    List {
        #[command(flatten)]
        filter: AnnotationFilter,
    },
}

/// Identifies a single annotation; exactly one field must be provided.
#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct AnnotationRef {
    /// Transaction ID
    #[arg(long)]
    pub tx: Option<String>,

    /// Address
    #[arg(long)]
    pub addr: Option<String>,

    /// Public key
    #[arg(long)]
    pub pubkey: Option<String>,

    /// Input outpoint (txid:vout)
    #[arg(long)]
    pub input: Option<String>,

    /// Output outpoint (txid:vout)
    #[arg(long)]
    pub output: Option<String>,

    /// Extended public key
    #[arg(long)]
    pub xpub: Option<String>,
}

impl AnnotationRef {
    /// Split into record type and reference string.
    /// Clap guarantees exactly one field is set.
    pub fn split(&self) -> (RecordType, &str) {
        if let Some(v) = &self.tx     { return (RecordType::Tx,     v); }
        if let Some(v) = &self.addr   { return (RecordType::Addr,   v); }
        if let Some(v) = &self.pubkey { return (RecordType::Pubkey, v); }
        if let Some(v) = &self.input  { return (RecordType::Input,  v); }
        if let Some(v) = &self.output { return (RecordType::Output, v); }
        if let Some(v) = &self.xpub  { return (RecordType::Xpub,   v); }
        unreachable!("clap ensures exactly one field is set")
    }
}

/// Optional filters for list/export; zero or more may be provided.
#[derive(Args, Debug)]
pub struct AnnotationFilter {
    /// Filter to transactions
    #[arg(long, group = "type_filter")]
    pub tx: bool,

    /// Filter to addresses
    #[arg(long, group = "type_filter")]
    pub addr: bool,

    /// Filter to public keys
    #[arg(long, group = "type_filter")]
    pub pubkey: bool,

    /// Filter to inputs
    #[arg(long, group = "type_filter")]
    pub input: bool,

    /// Filter to outputs
    #[arg(long, group = "type_filter")]
    pub output: bool,

    /// Filter to xpubs
    #[arg(long, group = "type_filter")]
    pub xpub: bool,

    /// Filter by wallet origin
    #[arg(long)]
    pub origin: Option<String>,

    /// Filter by wallet name
    #[arg(long)]
    pub wallet: Option<String>,

    /// Filter by tag (repeatable; all must match)
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
}
