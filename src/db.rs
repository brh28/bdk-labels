use rusqlite::Connection;

use crate::annotation::{Annotation, RecordType};
use crate::tag::Tag;

pub fn open(path: &std::path::Path) -> anyhow::Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS records (
            id          INTEGER PRIMARY KEY,
            type        TEXT NOT NULL CHECK(type IN ('tx', 'addr', 'pubkey', 'input', 'output', 'xpub')),
            ref         TEXT NOT NULL,
            description TEXT,
            origin      TEXT,
            spendable   INTEGER CHECK(spendable IS NULL OR spendable IN (0, 1)),
            wallet      TEXT,
            UNIQUE(type, ref)
        );

        CREATE TABLE IF NOT EXISTS tags (
            id          INTEGER PRIMARY KEY,
            record_id   INTEGER NOT NULL REFERENCES records(id) ON DELETE CASCADE,
            token       TEXT    NOT NULL,
            UNIQUE(record_id, token)
        );

        CREATE INDEX IF NOT EXISTS idx_records_type   ON records(type);
        CREATE INDEX IF NOT EXISTS idx_records_origin ON records(origin);
        CREATE INDEX IF NOT EXISTS idx_records_wallet ON records(wallet);
        CREATE INDEX IF NOT EXISTS idx_tags_token     ON tags(token);
    ")?;
    Ok(())
}

/// Insert or replace a full annotation. On conflict (same type + ref), updates all
/// fields and replaces all tags. Used by `import`.
pub fn upsert(conn: &Connection, annotation: &Annotation, wallet: Option<&str>) -> anyhow::Result<()> {
    let spendable = annotation.spendable.map(|s| s as i64);

    let record_id: i64 = conn.query_row(
        "INSERT INTO records (type, ref, description, origin, spendable, wallet)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(type, ref) DO UPDATE SET
             description = excluded.description,
             origin      = excluded.origin,
             spendable   = excluded.spendable,
             wallet      = excluded.wallet
         RETURNING id",
        rusqlite::params![
            annotation.record_type.to_string(),
            annotation.ref_,
            annotation.description,
            annotation.origin,
            spendable,
            wallet,
        ],
        |row| row.get(0),
    )?;

    conn.execute("DELETE FROM tags WHERE record_id = ?1", [record_id])?;

    for tag in &annotation.tags {
        conn.execute(
            "INSERT OR IGNORE INTO tags (record_id, token) VALUES (?1, ?2)",
            rusqlite::params![record_id, tag.to_token()],
        )?;
    }

    Ok(())
}

/// Update the description of an existing record, and optionally its origin.
/// Returns `true` if a record was found and updated, `false` if it doesn't exist.
/// Used by `describe`.
pub fn set_description(
    conn: &Connection,
    record_type: RecordType,
    ref_: &str,
    description: &str,
    origin: Option<&str>,
) -> anyhow::Result<bool> {
    let rows_changed = if let Some(o) = origin {
        conn.execute(
            "UPDATE records SET description = ?1, origin = ?2 WHERE type = ?3 AND ref = ?4",
            rusqlite::params![description, o, record_type.to_string(), ref_],
        )?
    } else {
        conn.execute(
            "UPDATE records SET description = ?1 WHERE type = ?2 AND ref = ?3",
            rusqlite::params![description, record_type.to_string(), ref_],
        )?
    };
    Ok(rows_changed > 0)
}

/// Append tags to an existing record. Used by `tag`.
pub fn add_tags(
    conn: &Connection,
    record_type: RecordType,
    ref_: &str,
    tags: &[Tag],
) -> anyhow::Result<()> {
    todo!("add_tags type={} ref={} tags={:?}", record_type, ref_, tags)
}

/// Remove specific tags from a record. Used by `rm <tags>`.
pub fn remove_tags(
    conn: &Connection,
    record_type: RecordType,
    ref_: &str,
    tags: &[Tag],
) -> anyhow::Result<()> {
    todo!("remove_tags type={} ref={} tags={:?}", record_type, ref_, tags)
}

/// Delete an entire record and its tags. Used by `rm --force`.
pub fn delete(
    conn: &Connection,
    record_type: RecordType,
    ref_: &str,
) -> anyhow::Result<()> {
    todo!("delete type={} ref={}", record_type, ref_)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_in_memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate(&conn).unwrap();
        conn
    }

    fn tx(ref_: &str) -> Annotation {
        Annotation {
            record_type: RecordType::Tx,
            ref_: ref_.to_owned(),
            description: None,
            origin: None,
            spendable: None,
            tags: vec![],
        }
    }

    #[test]
    fn upsert_and_list_roundtrip() {
        let conn = open_in_memory();
        let a = Annotation {
            record_type: RecordType::Tx,
            ref_: "abcd1234".to_owned(),
            description: Some("test tx".to_owned()),
            origin: Some("wpkh([deadbeef/84h/0h/0h])".to_owned()),
            spendable: None,
            tags: vec![Tag::parse("kyc:").unwrap(), Tag::parse("exchange:kraken").unwrap()],
        };
        upsert(&conn, &a, Some("mywallet")).unwrap();

        let results = list(&conn, None, None, None, &[]).unwrap();
        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert_eq!(r.ref_, "abcd1234");
        assert_eq!(r.description.as_deref(), Some("test tx"));
        assert_eq!(r.origin.as_deref(), Some("wpkh([deadbeef/84h/0h/0h])"));
        assert_eq!(r.tags.len(), 2);
    }

    #[test]
    fn upsert_updates_on_conflict() {
        let conn = open_in_memory();
        let mut a = tx("abcd1234");
        a.description = Some("original".to_owned());
        upsert(&conn, &a, Some("wallet-a")).unwrap();

        a.description = Some("updated".to_owned());
        upsert(&conn, &a, Some("wallet-b")).unwrap();

        let results = list(&conn, None, None, None, &[]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description.as_deref(), Some("updated"));
    }

    #[test]
    fn upsert_replaces_tags() {
        let conn = open_in_memory();
        let mut a = tx("abcd1234");
        a.tags = vec![Tag::parse("kyc:").unwrap()];
        upsert(&conn, &a, None).unwrap();

        a.tags = vec![Tag::parse("exchange:kraken").unwrap()];
        upsert(&conn, &a, None).unwrap();

        let results = list(&conn, None, None, None, &[]).unwrap();
        assert_eq!(results[0].tags.len(), 1);
        assert_eq!(results[0].tags[0].to_token(), "exchange:kraken");
    }

    #[test]
    fn list_filter_by_type() {
        let conn = open_in_memory();
        upsert(&conn, &tx("tx1"), None).unwrap();
        upsert(&conn, &Annotation {
            record_type: RecordType::Addr,
            ref_: "addr1".to_owned(),
            description: None, origin: None, spendable: None, tags: vec![],
        }, None).unwrap();

        let results = list(&conn, Some(RecordType::Tx), None, None, &[]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].ref_, "tx1");
    }

    #[test]
    fn list_filter_by_origin() {
        let conn = open_in_memory();
        let mut a = tx("tx1");
        a.origin = Some("wpkh([aabbccdd])".to_owned());
        upsert(&conn, &a, None).unwrap();
        upsert(&conn, &tx("tx2"), None).unwrap();

        let results = list(&conn, None, Some("wpkh([aabbccdd])"), None, &[]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].ref_, "tx1");
    }

    #[test]
    fn list_filter_by_wallet() {
        let conn = open_in_memory();
        upsert(&conn, &tx("tx1"), Some("wallet-a")).unwrap();
        upsert(&conn, &tx("tx2"), Some("wallet-b")).unwrap();

        let results = list(&conn, None, None, Some("wallet-a"), &[]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].ref_, "tx1");
    }

    #[test]
    fn list_filter_by_tag() {
        let conn = open_in_memory();
        let mut a = tx("tx1");
        a.tags = vec![Tag::parse("kyc:").unwrap()];
        upsert(&conn, &a, None).unwrap();
        upsert(&conn, &tx("tx2"), None).unwrap();

        let results = list(&conn, None, None, None, &[Tag::parse("kyc:").unwrap()]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].ref_, "tx1");
    }

    #[test]
    fn list_filter_by_multiple_tags_and_semantics() {
        let conn = open_in_memory();
        let mut both = tx("tx-both");
        both.tags = vec![Tag::parse("kyc:").unwrap(), Tag::parse("exchange:kraken").unwrap()];
        upsert(&conn, &both, None).unwrap();

        let mut one = tx("tx-one");
        one.tags = vec![Tag::parse("kyc:").unwrap()];
        upsert(&conn, &one, None).unwrap();

        let filter = vec![Tag::parse("kyc:").unwrap(), Tag::parse("exchange:kraken").unwrap()];
        let results = list(&conn, None, None, None, &filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].ref_, "tx-both");
    }

    #[test]
    fn set_description_returns_true_when_found() {
        let conn = open_in_memory();
        upsert(&conn, &tx("abcd1234"), None).unwrap();

        let found = set_description(&conn, RecordType::Tx, "abcd1234", "new desc", None).unwrap();
        assert!(found);

        let results = list(&conn, None, None, None, &[]).unwrap();
        assert_eq!(results[0].description.as_deref(), Some("new desc"));
    }

    #[test]
    fn set_description_returns_false_when_missing() {
        let conn = open_in_memory();
        let found = set_description(&conn, RecordType::Tx, "doesnotexist", "desc", None).unwrap();
        assert!(!found);
    }

    #[test]
    fn set_description_with_origin_updates_both() {
        let conn = open_in_memory();
        upsert(&conn, &tx("abcd1234"), None).unwrap();

        set_description(&conn, RecordType::Tx, "abcd1234", "desc", Some("wpkh([aabbccdd])")).unwrap();

        let results = list(&conn, None, None, None, &[]).unwrap();
        assert_eq!(results[0].description.as_deref(), Some("desc"));
        assert_eq!(results[0].origin.as_deref(), Some("wpkh([aabbccdd])"));
    }
}

/// Query annotations with optional filters. Used by `list` and `export`.
///
/// All tag filters must match (AND semantics).
pub fn list(
    conn: &Connection,
    record_type: Option<RecordType>,
    origin: Option<&str>,
    wallet: Option<&str>,
    tags: &[Tag],
) -> anyhow::Result<Vec<Annotation>> {
    let mut sql = String::from(
        "SELECT id, type, ref, description, origin, spendable FROM records WHERE 1=1",
    );
    let mut params: Vec<String> = Vec::new();

    if let Some(rt) = record_type {
        sql.push_str(" AND type = ?");
        params.push(rt.to_string());
    }
    if let Some(o) = origin {
        sql.push_str(" AND origin = ?");
        params.push(o.to_owned());
    }
    if let Some(w) = wallet {
        sql.push_str(" AND wallet = ?");
        params.push(w.to_owned());
    }
    for tag in tags {
        sql.push_str(" AND id IN (SELECT record_id FROM tags WHERE token = ?)");
        params.push(tag.to_token());
    }
    sql.push_str(" ORDER BY type, ref");

    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<(i64, String, String, Option<String>, Option<String>, Option<i64>)> = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .collect::<Result<_, _>>()?;

    let mut annotations = Vec::with_capacity(rows.len());
    for (id, type_str, ref_, description, origin, spendable) in rows {
        let record_type = type_str.parse::<RecordType>()?;

        let mut tag_stmt = conn.prepare("SELECT token FROM tags WHERE record_id = ?")?;
        let record_tags: Vec<Tag> = tag_stmt
            .query_map([id], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .filter_map(|token| Tag::parse(&token))
            .collect();

        annotations.push(Annotation {
            record_type,
            ref_,
            description,
            origin,
            spendable: spendable.map(|s| s != 0),
            tags: record_tags,
        });
    }

    Ok(annotations)
}
