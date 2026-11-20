use std::ops::Range;

pub const CURRENT_VERSION: u32 = 7;

use rune::{Block, MarkRange, TextEdit};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Kind {
    #[default]
    Text,
    Heading1,
    Heading2,
    Heading3,
    Bullet,
    Numbered,
    Todo,
    Toggle,
    ToggleHeading1,
    ToggleHeading2,
    ToggleHeading3,
    Quote,
    Callout,
    Code,
    Divider,
    PageLink,
    DatabaseView,
    Image,
    Attachment,
    TableOfContents,
}

impl Kind {
    pub const ALL: [Self; 20] = [
        Self::Text,
        Self::Heading1,
        Self::Heading2,
        Self::Heading3,
        Self::Bullet,
        Self::Numbered,
        Self::Todo,
        Self::Toggle,
        Self::ToggleHeading1,
        Self::ToggleHeading2,
        Self::ToggleHeading3,
        Self::Quote,
        Self::Callout,
        Self::Code,
        Self::Divider,
        Self::PageLink,
        Self::DatabaseView,
        Self::Image,
        Self::Attachment,
        Self::TableOfContents,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Heading1 => "Heading 1",
            Self::Heading2 => "Heading 2",
            Self::Heading3 => "Heading 3",
            Self::Bullet => "Bulleted list",
            Self::Numbered => "Numbered list",
            Self::Todo => "Task",
            Self::Toggle => "Toggle list",
            Self::ToggleHeading1 => "Toggle heading 1",
            Self::ToggleHeading2 => "Toggle heading 2",
            Self::ToggleHeading3 => "Toggle heading 3",
            Self::Quote => "Quote",
            Self::Callout => "Callout",
            Self::Code => "Code",
            Self::Divider => "Divider",
            Self::PageLink => "Link to page",
            Self::DatabaseView => "Embedded database view",
            Self::Image => "Image",
            Self::Attachment => "Attachment",
            Self::TableOfContents => "Table of contents",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Self::Text => "T",
            Self::Heading1 => "H₁",
            Self::Heading2 => "H₂",
            Self::Heading3 => "H₃",
            Self::Bullet => "•",
            Self::Numbered => "1.",
            Self::Todo => "☑",
            Self::Toggle | Self::ToggleHeading1 | Self::ToggleHeading2 | Self::ToggleHeading3 => {
                "▸"
            }
            Self::Quote => "❝",
            Self::Callout => "!",
            Self::Code => "<>",
            Self::Divider => "—",
            Self::PageLink => "↗",
            Self::DatabaseView => "▦",
            Self::Image => "▧",
            Self::Attachment => "↗",
            Self::TableOfContents => "≡",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Text => "Just an idea",
            Self::Heading1 => "A large section heading",
            Self::Heading2 => "A medium heading",
            Self::Heading3 => "A small heading",
            Self::Bullet => "An unordered list",
            Self::Numbered => "A step-by-step list",
            Self::Todo => "A checkbox, with optional subtasks",
            Self::Toggle => "A section you can expand or collapse",
            Self::ToggleHeading1 => "A large heading with collapsible content",
            Self::ToggleHeading2 => "A medium heading with collapsible content",
            Self::ToggleHeading3 => "A small heading with collapsible content",
            Self::Quote => "Highlight a quote",
            Self::Callout => "Information worth highlighting",
            Self::Code => "Code in a monospace font",
            Self::Divider => "A visual separator",
            Self::PageLink => "Link two notes in your notebook",
            Self::DatabaseView => "Display an existing database in this page",
            Self::Image => "A local image in your notes",
            Self::Attachment => "Attach a document to your notebook",
            Self::TableOfContents => "Navigate the headings on this page",
        }
    }

    pub fn matches(self, query: &str) -> bool {
        let aliases = match self {
            Self::Text => "paragraph texte",
            Self::Heading1 => "h1 heading title titre",
            Self::Heading2 => "h2 heading titre",
            Self::Heading3 => "h3 heading titre",
            Self::Bullet => "bullet list liste puce",
            Self::Numbered => "numbered ordered liste numero",
            Self::Todo => "todo to-do list to do task check tache checkbox checklist",
            Self::Toggle => "toggle details accordeon tiroir repliable",
            Self::ToggleHeading1 => "toggle h1 heading titre tiroir repliable",
            Self::ToggleHeading2 => "toggle h2 heading titre tiroir repliable",
            Self::ToggleHeading3 => "toggle h3 heading titre tiroir repliable",
            Self::Quote => "quote citation",
            Self::Callout => "callout encadre info",
            Self::Code => "code rust",
            Self::Divider => "divider separator separateur",
            Self::PageLink => "link lien page backlink",
            Self::DatabaseView => "database base table linked embed vue integree",
            Self::Image => "image photo picture capture",
            Self::Attachment => "file fichier attachment pdf piece jointe",
            Self::TableOfContents => "toc sommaire table contents navigation",
        };
        let normalize = |s: &str| {
            s.to_lowercase()
                .replace(['é', 'è', 'ê'], "e")
                .replace(['â', 'à'], "a")
        };
        normalize(&format!("{} {aliases}", self.name())).contains(&normalize(query))
    }

    pub fn is_toggle(self) -> bool {
        matches!(
            self,
            Self::Toggle | Self::ToggleHeading1 | Self::ToggleHeading2 | Self::ToggleHeading3
        )
    }

    pub fn is_list(self) -> bool {
        matches!(self, Self::Bullet | Self::Numbered | Self::Todo) || self.is_toggle()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteBlock {
    pub id: String,
    pub kind: Kind,
    pub html: String,
    #[serde(default)]
    pub checked: bool,
    #[serde(default)]
    pub indent: u8,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub collapsed: bool,
    #[serde(default)]
    pub number_start: Option<u32>,
    #[serde(default)]
    pub embedded: Option<crate::workspace::EmbeddedView>,
    #[serde(default)]
    pub asset: Option<crate::workspace::Asset>,
}

impl NoteBlock {
    pub fn new(kind: Kind) -> Self {
        Self::text(kind, "")
    }

    pub fn text(kind: Kind, text: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind,
            html: TextEdit::new(text).rich().to_html(),
            checked: false,
            indent: 0,
            target_id: None,
            collapsed: false,
            number_start: None,
            embedded: None,
            asset: None,
        }
    }

    pub fn plain_text(&self) -> String {
        TextEdit::from_html(&self.html).text.clone()
    }

    pub fn export_html(&self) -> String {
        let inner = self
            .html
            .strip_prefix("<p>")
            .and_then(|s| s.strip_suffix("</p>"))
            .unwrap_or(&self.html);
        match self.kind {
            Kind::Heading1 => format!("<h1>{inner}</h1>"),
            Kind::Heading2 => format!("<h2>{inner}</h2>"),
            Kind::Heading3 => format!("<h3>{inner}</h3>"),
            Kind::Bullet => format!("<ul><li>{inner}</li></ul>"),
            Kind::Numbered => format!("<ol><li>{inner}</li></ol>"),
            Kind::Todo => format!("<p>[{}] {inner}</p>", if self.checked { "x" } else { " " }),
            Kind::Quote | Kind::Callout => format!("<blockquote>{inner}</blockquote>"),
            Kind::Code => format!("<pre><code>{}</code></pre>", escape(&self.plain_text())),
            Kind::Divider => "<hr>".into(),
            Kind::Text => self.html.clone(),
            Kind::Toggle | Kind::ToggleHeading1 | Kind::ToggleHeading2 | Kind::ToggleHeading3 => {
                format!("<p>{inner}</p>")
            }
            Kind::PageLink => format!(
                "<p><a href=\"folio://page/{}\">{}</a></p>",
                escape(self.target_id.as_deref().unwrap_or("")),
                escape(&self.plain_text())
            ),
            Kind::DatabaseView => format!("<p>Database: {}</p>", escape(&self.plain_text())),
            Kind::Image | Kind::Attachment => self.asset.as_ref().map_or_else(
                || "<p>Attachment</p>".into(),
                |a| {
                    let path = format!("folio-assets/{}", crate::workspace::asset_export_name(a));
                    if self.kind == Kind::Image {
                        format!(
                            "<img src=\"{}\" alt=\"{}\">",
                            escape(&path),
                            escape(&a.name)
                        )
                    } else {
                        format!(
                            "<p><a href=\"{}\">{}</a></p>",
                            escape(&path),
                            escape(&a.name)
                        )
                    }
                },
            ),
            Kind::TableOfContents => "<p>Sommaire</p>".into(),
        }
    }
}

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Slice text and inline attributes together, keeping UTF-8 ranges valid.
pub fn rich_slice(edit: &TextEdit, range: Range<usize>) -> TextEdit {
    let marks = edit
        .marks()
        .iter()
        .filter_map(|mark| {
            let start = mark.start.max(range.start);
            let end = mark.end.min(range.end);
            (start < end)
                .then(|| MarkRange::new(start - range.start..end - range.start, mark.mark.clone()))
        })
        .collect();
    TextEdit::from_parts(&edit.text[range], marks, vec![Block::Paragraph])
}

/// One native rich editor becomes several independently addressable blocks.
pub fn import_legacy(html: &str) -> Vec<NoteBlock> {
    let edit = TextEdit::from_html(html);
    let mut blocks: Vec<NoteBlock> = Vec::new();
    for paragraph in edit.paragraphs() {
        let kind = match paragraph.block {
            Block::Paragraph => Kind::Text,
            Block::Bullet(_) => Kind::Bullet,
            Block::Numbered(_) => Kind::Numbered,
            Block::Quote => Kind::Quote,
            Block::Code => Kind::Code,
        };
        // Keep soft line breaks and multi-line code inside their original block.
        if (paragraph.soft_break || kind == Kind::Code)
            && blocks.last().is_some_and(|block| block.kind == kind)
        {
            let block = blocks.last_mut().expect("checked above");
            let previous = TextEdit::from_html(&block.html);
            let next = rich_slice(&edit, paragraph.range);
            block.html = join_rich(&previous, &next, "\n").to_html();
        } else {
            let mut block = NoteBlock::new(kind);
            block.indent = paragraph.block.list_level().unwrap_or(0);
            block.html = rich_slice(&edit, paragraph.range).to_html();
            blocks.push(block);
        }
    }
    if blocks.is_empty() {
        blocks.push(NoteBlock::new(Kind::Text));
    }
    blocks
}

pub fn join_rich(left: &TextEdit, right: &TextEdit, separator: &str) -> TextEdit {
    let offset = left.text.len() + separator.len();
    let mut marks = left.marks().to_vec();
    marks.extend(
        right
            .marks()
            .iter()
            .map(|mark| MarkRange::new(mark.start + offset..mark.end + offset, mark.mark.clone())),
    );
    TextEdit::from_parts(
        format!("{}{separator}{}", left.text, right.text),
        marks,
        vec![Block::Paragraph],
    )
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageContent {
    pub version: u32,
    pub blocks: Vec<NoteBlock>,
    pub emoji: String,
    #[serde(default, skip_serializing_if = "crate::icons::IconColor::is_default")]
    pub icon_color: crate::icons::IconColor,
    pub cover: u8,
    pub wide: bool,
    #[serde(default)]
    pub collection: Option<crate::collection::Collection>,
    #[serde(default)]
    pub properties: crate::collection::Properties,
    #[serde(default)]
    pub dashboard: Option<crate::dashboard::DashboardConfig>,
    #[serde(default)]
    pub order: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected: Option<crate::security::Sealed>,
}

impl PageContent {
    pub fn legacy(html: &str) -> Self {
        Self {
            version: CURRENT_VERSION,
            blocks: import_legacy(html),
            emoji: "📄".into(),
            icon_color: Default::default(),
            cover: 0,
            wide: false,
            collection: None,
            properties: crate::collection::Properties::default(),
            dashboard: None,
            order: 0,
            protected: None,
        }
    }
}
