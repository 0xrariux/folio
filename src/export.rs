//! Portable, local exports. CSV keeps stored numbers instead of rounded display values.
use crate::{
    document::{Kind, escape},
    fields::CellValue,
    model::Page,
};
use anyhow::{Context, Result};
use rune::{Mark, TextEdit};
use std::{io::Write, path::Path};

pub struct ExportDocument {
    pub file_name: String,
    pub extension: &'static str,
    pub contents: String,
    pub assets: Vec<crate::workspace::Asset>,
}

pub fn file_name(title: &str, extension: &str) -> String {
    let name: String = title
        .chars()
        .take(80)
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let name = name.trim().trim_matches('.').trim();
    let name = if name.is_empty() { "Folio" } else { name };
    format!("{name}.{extension}")
}

fn md_escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if "\\`*_{}[]<>#+-.!|~".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

// Inline HTML preserves overlapping styles and underline, which Markdown lacks.
// Rune parses and canonicalizes the stored HTML; raw user HTML is never copied through.
fn inline(html: &str) -> String {
    let edit = TextEdit::from_html(html);
    let mut boundaries = vec![0, edit.text.len()];
    for mark in edit.marks() {
        boundaries.extend([mark.start, mark.end]);
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    let mut out = String::new();
    for pair in boundaries.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        let marks: Vec<_> = edit
            .marks()
            .iter()
            .filter(|m| m.start <= start && m.end >= end)
            .map(|m| &m.mark)
            .collect();
        let text = &edit.text[start..end];
        let mut part = if marks.iter().any(|m| matches!(m, Mark::Code)) {
            escape(text)
        } else {
            md_escape(text).replace('&', "&amp;")
        };
        part = part.replace('\n', "<br>\n");
        for mark in marks.iter().rev() {
            part = match mark {
                Mark::Bold => format!("<strong>{part}</strong>"),
                Mark::Italic => format!("<em>{part}</em>"),
                Mark::Underline => format!("<u>{part}</u>"),
                Mark::Strike => format!("<s>{part}</s>"),
                Mark::Code => format!("<code>{part}</code>"),
                Mark::Link(url) => format!("<a href=\"{}\">{part}</a>", escape(url)),
                _ => part,
            };
        }
        out.push_str(&part);
    }
    out
}

pub fn markdown(page: &Page, pages: &[Page]) -> ExportDocument {
    let mut contents = format!(
        "# {}\n\n",
        md_escape(page.display_title()).replace('\n', " ")
    );
    if let Some(collection) = pages
        .iter()
        .find(|p| Some(&p.id) == page.parent_id.as_ref())
        .and_then(|p| p.collection.as_ref())
    {
        for (name, value) in [
            ("Statut", page.properties.status.name().to_string()),
            ("Priorité", page.properties.priority.name().to_string()),
            ("Échéance", page.properties.due.clone().unwrap_or_default()),
            ("Étiquettes", page.properties.tags.join(", ")),
        ] {
            contents.push_str(&format!("- **{name}** : {}\n", md_escape(&value)));
        }
        for column in &collection.columns {
            let value = if column.kind == crate::fields::ColumnKind::Relation {
                crate::calculations::related(pages, page, column)
                    .iter()
                    .map(|p| format!("{} ({})", p.display_title(), p.id))
                    .collect::<Vec<_>>()
                    .join("; ")
            } else if matches!(
                column.kind,
                crate::fields::ColumnKind::Formula | crate::fields::ColumnKind::Rollup
            ) {
                crate::calculations::value(page, column, pages)
                    .map(|n| n.to_string())
                    .unwrap_or_else(|e| format!("Erreur : {e}"))
            } else {
                match page.properties.custom.get(&column.id) {
                    Some(CellValue::Text(v) | CellValue::Number(v) | CellValue::Date(v)) => {
                        v.clone()
                    }
                    Some(CellValue::Checkbox(v)) => v.to_string(),
                    Some(CellValue::Relation(ids)) => ids.join("; "),
                    None => String::new(),
                }
            };
            contents.push_str(&format!(
                "- **{}** : {}\n",
                md_escape(&column.name),
                md_escape(&value).replace('\n', "<br>")
            ));
        }
        contents.push('\n');
    }
    let ends = crate::outline::subtree_ends(&page.blocks);
    let numbers = crate::outline::numbers(&page.blocks);
    let mut index = 0;
    while index < page.blocks.len() {
        let block = &page.blocks[index];
        if block.kind.is_toggle() {
            // Markdown inside raw HTML is not portable. Emit the complete HTML tree.
            let prefix = "    ".repeat(usize::from(block.indent.min(crate::outline::MAX_INDENT)));
            let tree = crate::outline::html(&page.blocks[index..ends[index]]);
            for line in tree.lines() {
                contents.push_str(&prefix);
                contents.push_str(line);
                contents.push('\n');
            }
            contents.push('\n');
            index = ends[index];
            continue;
        }
        let rich = inline(&block.html);
        let prefix = "    ".repeat(usize::from(block.indent.min(crate::outline::MAX_INDENT)));
        let text = match block.kind {
            Kind::Text => rich,
            Kind::DatabaseView => format!(
                "[Base intégrée](folio://page/{})",
                block.embedded.as_ref().map_or("", |e| e.base_id.as_str())
            ),
            Kind::Image | Kind::Attachment => block.asset.as_ref().map_or_else(String::new, |a| {
                format!(
                    "{}[{}](folio-assets/{})",
                    if block.kind == Kind::Image { "!" } else { "" },
                    md_escape(&a.name),
                    crate::workspace::asset_export_name(a)
                )
            }),
            Kind::TableOfContents => "<!-- Sommaire -->".into(),
            Kind::Heading1 => format!("# {rich}"),
            Kind::Heading2 => format!("## {rich}"),
            Kind::Heading3 => format!("### {rich}"),
            Kind::Bullet => format!("{prefix}- {}", rich.replace('\n', &format!("\n{prefix}  "))),
            Kind::Numbered => format!(
                "{prefix}{}. {}",
                numbers[index],
                rich.replace('\n', &format!("\n{prefix}   "))
            ),
            Kind::Todo => format!(
                "{prefix}- [{}] {}",
                if block.checked { "x" } else { " " },
                rich.replace('\n', &format!("\n{prefix}      "))
            ),
            Kind::Quote | Kind::Callout => format!("> {}", rich.replace('\n', "\n> ")),
            Kind::Divider => "---".into(),
            Kind::Code => {
                let text = block.plain_text();
                let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
                let fence = "`".repeat((longest + 1).max(3));
                format!("{fence}\n{text}\n{fence}")
            }
            Kind::PageLink => {
                let target = block.target_id.as_deref().unwrap_or("");
                let title = pages
                    .iter()
                    .find(|p| p.id == target)
                    .map_or_else(|| block.plain_text(), |p| p.display_title().to_string());
                let id: String = target
                    .bytes()
                    .map(|b| {
                        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                            (b as char).to_string()
                        } else {
                            format!("%{b:02X}")
                        }
                    })
                    .collect();
                format!("[{}](folio://page/{id})", md_escape(&title))
            }
            Kind::Toggle | Kind::ToggleHeading1 | Kind::ToggleHeading2 | Kind::ToggleHeading3 => {
                unreachable!("handled with its children")
            }
        };
        if !matches!(block.kind, Kind::Bullet | Kind::Numbered | Kind::Todo) {
            contents.push_str(&prefix);
            contents.push_str(&text.replace('\n', &format!("\n{prefix}")));
        } else {
            contents.push_str(&text);
        }
        contents.push_str("\n\n");
        index += 1;
    }
    ExportDocument {
        file_name: file_name(page.display_title(), "md"),
        extension: "md",
        contents,
        assets: page.blocks.iter().filter_map(|b| b.asset.clone()).collect(),
    }
}

fn csv_row(out: &mut String, values: impl IntoIterator<Item = String>) {
    for (i, value) in values.into_iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&value.replace('"', "\"\""));
        out.push('"');
    }
    out.push_str("\r\n");
}

pub fn csv(title: &str, sources: &[&Page], records: &[&Page]) -> ExportDocument {
    let context: Vec<_> = sources
        .iter()
        .chain(records.iter())
        .map(|p| (*p).clone())
        .collect();
    csv_with_context(title, sources, records, &context)
}
pub fn csv_with_context(
    title: &str,
    sources: &[&Page],
    records: &[&Page],
    context: &[Page],
) -> ExportDocument {
    let columns: Vec<_> = sources
        .iter()
        .flat_map(|base| {
            base.collection
                .iter()
                .flat_map(move |c| c.columns.iter().map(move |field| (*base, field)))
        })
        .collect();
    let mut header: Vec<String> = [
        "ID",
        "Base",
        "Nom",
        "Statut",
        "Priorité",
        "Échéance",
        "Étiquettes",
        "Notes",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    header.extend(columns.iter().map(|(base, column)| {
        if sources.len() > 1 {
            format!("{} / {}", base.display_title(), column.name)
        } else {
            column.name.clone()
        }
    }));
    let mut contents = String::new();
    csv_row(&mut contents, header);
    for page in records {
        let source = sources
            .iter()
            .find(|s| page.parent_id.as_deref() == Some(&s.id));
        if source.is_none() {
            continue;
        }
        let mut values = vec![
            page.id.clone(),
            source.unwrap().display_title().into(),
            page.display_title().into(),
            page.properties.status.name().into(),
            page.properties.priority.name().into(),
            page.properties.due.clone().unwrap_or_default(),
            page.properties.tags.join(", "),
            page.plain_text.clone(),
        ];
        values.extend(columns.iter().map(|(base, column)| {
            if page.parent_id.as_deref() != Some(&base.id) {
                return String::new();
            }
            if column.kind == crate::fields::ColumnKind::Relation {
                return crate::calculations::related(context, page, column)
                    .iter()
                    .map(|p| p.id.clone())
                    .collect::<Vec<_>>()
                    .join("; ");
            }
            if matches!(
                column.kind,
                crate::fields::ColumnKind::Rollup | crate::fields::ColumnKind::Formula
            ) {
                return crate::calculations::value(page, column, context)
                    .map(|n| n.to_string())
                    .unwrap_or_else(|e| format!("Erreur : {e}"));
            }
            match page.properties.custom.get(&column.id) {
                Some(CellValue::Text(v) | CellValue::Number(v) | CellValue::Date(v)) => v.clone(),
                Some(CellValue::Checkbox(v)) => v.to_string(),
                Some(CellValue::Relation(ids)) => ids.join("; "),
                None => String::new(),
            }
        }));
        csv_row(&mut contents, values);
    }
    ExportDocument {
        file_name: file_name(title, "csv"),
        extension: "csv",
        contents,
        assets: Vec::new(),
    }
}

/// Write beside the destination and replace it only after the complete file is synced.
pub fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)
        .context("Création du fichier temporaire d’export")?;
    file.write_all(contents.as_bytes())
        .context("Écriture de l’export")?;
    file.as_file()
        .sync_all()
        .context("Synchronisation de l’export")?;
    file.persist(path)
        .map_err(|e| e.error)
        .context("Enregistrement de l’export")?;
    Ok(())
}
