//! Manual-only fixture producer, deliberately pinned to an older storage engine.
//! Do not run in CI or replace established fixtures during a dependency upgrade.
use std::{fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("output fixture path required"),
    );
    assert!(!output.exists(), "refusing to replace an existing fixture");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let notebook: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("tests/fixtures/notebook-v7.json"))?)?;
    tokio::runtime::Builder::new_current_thread()
        .build()?
        .block_on(async {
            let database = turso::Builder::new_local(output.to_str().unwrap())
                .build()
                .await?;
            let connection = database.connect()?;
            for name in ["001_initial.sql", "002_blocks.sql", "003_workspace.sql"] {
                for sql in fs::read_to_string(root.join("migrations").join(name))?
                    .split(';')
                    .filter(|s| !s.trim().is_empty())
                {
                    connection.execute(sql, ()).await?;
                }
            }
            for page in notebook["pages"].as_array().unwrap() {
                connection
                    .execute(
                        "INSERT INTO pages VALUES(?,?,?,?,?,?,?,?)",
                        (
                            page["id"].as_str().unwrap().to_owned(),
                            page["parent_id"].as_str().map(str::to_owned),
                            page["title"].as_str().unwrap().to_owned(),
                            page["body_html"].as_str().unwrap().to_owned(),
                            page["plain_text"].as_str().unwrap().to_owned(),
                            i64::from(page["favorite"].as_bool().unwrap()),
                            i64::from(page["archived"].as_bool().unwrap()),
                            page["updated_at"].as_i64().unwrap(),
                        ),
                    )
                    .await?;
                let mut content = page.as_object().unwrap().clone();
                for key in [
                    "id",
                    "parent_id",
                    "title",
                    "body_html",
                    "plain_text",
                    "favorite",
                    "archived",
                    "updated_at",
                ] {
                    content.remove(key);
                }
                content.insert("version".into(), 7.into());
                connection
                    .execute(
                        "INSERT INTO page_content VALUES(?,?)",
                        (
                            page["id"].as_str().unwrap().to_owned(),
                            serde_json::to_string(&content)?,
                        ),
                    )
                    .await?;
            }
            for h in notebook["history"].as_array().unwrap().iter().rev() {
                connection
                    .execute(
                        "INSERT INTO page_history VALUES(?,?,?,?)",
                        (
                            h["id"].as_str().unwrap().to_owned(),
                            h["page_id"].as_str().unwrap().to_owned(),
                            h["created_at"].as_i64().unwrap(),
                            h["page"].to_string(),
                        ),
                    )
                    .await?;
            }
            connection
                .execute(
                    "INSERT INTO preferences VALUES('app',?)",
                    (notebook["preferences"].to_string(),),
                )
                .await?;
            connection
                .execute("INSERT INTO preferences VALUES('initialized','true')", ())
                .await?;
            let mut rows = connection
                .query("PRAGMA wal_checkpoint(TRUNCATE)", ())
                .await?;
            while rows.next().await?.is_some() {}
            Ok::<(), Box<dyn std::error::Error>>(())
        })?;
    let wal = PathBuf::from(format!("{}-wal", output.display()));
    if wal.exists() {
        assert_eq!(wal.metadata()?.len(), 0, "fixture still needs its WAL");
        fs::remove_file(wal)?;
    }
    println!("Synthetic fixture generated with Turso 0.7.2");
    Ok(())
}
