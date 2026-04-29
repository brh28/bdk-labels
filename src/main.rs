mod annotation;
mod cli;
mod convert;
mod db;
mod label;
mod tag;

use std::io::Read;

use annotation::{Annotation, RecordType};
use bip329::Labels;
use clap::Parser;
use cli::{AnnotationFilter, Cli, Command};
use convert::annotations_from_bip329;
use tag::Tag;

fn validate_ref(record_type: RecordType, ref_: &str) -> anyhow::Result<()> {
    match record_type {
        RecordType::Tx => {
            ref_.parse::<bitcoin::Txid>()
                .map_err(|e| anyhow::anyhow!("invalid txid '{ref_}': {e:?}"))?;
        }
        RecordType::Addr => {
            ref_.parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
                .map_err(|e| anyhow::anyhow!("invalid address '{ref_}': {e:?}"))?;
        }
        RecordType::Input | RecordType::Output => {
            ref_.parse::<bitcoin::OutPoint>()
                .map_err(|e| anyhow::anyhow!("invalid outpoint '{ref_}': {e:?}"))?;
        }
        RecordType::Pubkey | RecordType::Xpub => {}
    }
    Ok(())
}

fn filter_record_type(filter: &AnnotationFilter) -> Option<RecordType> {
    if filter.tx          { Some(RecordType::Tx) }
    else if filter.addr   { Some(RecordType::Addr) }
    else if filter.pubkey { Some(RecordType::Pubkey) }
    else if filter.input  { Some(RecordType::Input) }
    else if filter.output { Some(RecordType::Output) }
    else if filter.xpub   { Some(RecordType::Xpub) }
    else                  { None }
}

fn print_annotation(a: &Annotation) {
    let type_str = format!("{:<6}", a.record_type);
    let desc = a.description.as_deref().unwrap_or("-");
    let tags = if a.tags.is_empty() {
        String::new()
    } else {
        format!("  [{}]", a.tags.iter().map(|t| t.to_token()).collect::<Vec<_>>().join(", "))
    };
    let origin = a.origin.as_deref()
        .map(|o| format!("  origin:{o}"))
        .unwrap_or_default();
    println!("{type_str}  {}  {desc}{tags}{origin}", a.ref_);
}

fn default_db_path() -> anyhow::Result<std::path::PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?;
    Ok(home.join(".bdk-bitcoin").join("bdk-labels.db"))
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let db_path = match cli.db {
        Some(p) => p,
        None => default_db_path()?,
    };
    let conn = db::open(&db_path)?;

    match cli.command {
        Command::Import { file, wallet } => {
            let wallet_name: Option<String> = wallet.or_else(|| {
                let stem = file.as_ref()?.file_stem()?.to_str()?;
                Some(stem.to_owned())
            });
            if file.is_none() && wallet_name.is_none() {
                anyhow::bail!("--wallet is required when reading from stdin");
            }
            let labels = match file {
                Some(path) => Labels::try_from_file(path)?,
                None => {
                    let mut content = String::new();
                    std::io::stdin().lock().read_to_string(&mut content)?;
                    Labels::try_from_str(&content)?
                }
            };
            let annotations = annotations_from_bip329(labels);
            let count = annotations.len();
            for annotation in &annotations {
                db::upsert(&conn, annotation, wallet_name.as_deref())?;
            }
            let wallet_display = wallet_name.as_deref().unwrap_or("<none>");
            println!("Imported {count} annotations into wallet '{wallet_display}'");
        }
        Command::Add { annotation, wallet, description, origin, tags, spendable, not_spendable } => {
            let (record_type, ref_) = annotation.split();
            validate_ref(record_type, ref_)?;
            let tag_values: Vec<Tag> = tags.iter().filter_map(|s| Tag::parse(s)).collect();
            let spendable_value = if spendable { Some(true) } else if not_spendable { Some(false) } else { None };
            let new_annotation = Annotation {
                record_type,
                ref_: ref_.to_owned(),
                description,
                origin,
                spendable: spendable_value,
                tags: tag_values,
            };
            db::upsert(&conn, &new_annotation, Some(&wallet))?;
            println!("Created annotation for {record_type} {ref_}");
        }
        Command::Describe { annotation, description, origin } => {
            let (record_type, ref_) = annotation.split();
            let found = db::set_description(&conn, record_type, ref_, &description, origin.as_deref())?;
            if found {
                println!("Updated description for {record_type} {ref_}");
            } else {
                anyhow::bail!("no annotation found for {record_type} {ref_} — use 'add' to create it first");
            }
        }
        Command::Tag { annotation, tags } => {
            let (record_type, ref_) = annotation.split();
            let tag_values: Vec<Tag> = tags.iter().filter_map(|s| Tag::parse(s)).collect();
            let found = db::add_tags(&conn, record_type, ref_, &tag_values)?;
            if found {
                println!("Added {} tag(s) to {record_type} {ref_}", tag_values.len());
            } else {
                anyhow::bail!("no annotation found for {record_type} {ref_} — use 'add' to create it first");
            }
        }
        Command::Rm { annotation, tags, force } => {
            if tags.is_empty() && !force {
                anyhow::bail!("pass --force to delete an entire annotation, or specify tags to remove");
            }
            todo!("rm annotation={:?} tags={:?} force={}", annotation, tags, force)
        }
        Command::Export { filter } => {
            todo!("export filter={:?}", filter)
        }
        Command::List { filter } => {
            let record_type = filter_record_type(&filter);
            let tag_filters: Vec<Tag> = filter.tags.iter()
                .filter_map(|s| Tag::parse(s))
                .collect();
            let annotations = db::list(&conn, record_type, filter.origin.as_deref(), filter.wallet.as_deref(), &tag_filters)?;
            if annotations.is_empty() {
                println!("No annotations found.");
            } else {
                for annotation in &annotations {
                    print_annotation(annotation);
                }
            }
        }
    }

    Ok(())
}
