use std::{fmt, str::FromStr};

use crate::tag::Tag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub record_type: RecordType,
    pub ref_: String,
    pub description: Option<String>,
    pub origin: Option<String>,
    pub spendable: Option<bool>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordType {
    Tx,
    Addr,
    Pubkey,
    Input,
    Output,
    Xpub,
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordType::Tx => write!(f, "tx"),
            RecordType::Addr => write!(f, "addr"),
            RecordType::Pubkey => write!(f, "pubkey"),
            RecordType::Input => write!(f, "input"),
            RecordType::Output => write!(f, "output"),
            RecordType::Xpub => write!(f, "xpub"),
        }
    }
}

impl FromStr for RecordType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tx" => Ok(RecordType::Tx),
            "addr" => Ok(RecordType::Addr),
            "pubkey" => Ok(RecordType::Pubkey),
            "input" => Ok(RecordType::Input),
            "output" => Ok(RecordType::Output),
            "xpub" => Ok(RecordType::Xpub),
            other => anyhow::bail!("unknown record type: {other}"),
        }
    }
}
