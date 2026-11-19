use std::time::{SystemTime, UNIX_EPOCH};

use crate::document::{Kind, NoteBlock, PageContent, import_legacy};

/// HTML is Rune's portable rich-text representation; plain text is the search index.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub body_html: String,
    pub plain_text: String,
    pub favorite: bool,
    pub archived: bool,
    pub updated_at: i64,
    pub blocks: Vec<NoteBlock>,
    pub emoji: String,
    #[serde(default, skip_serializing_if = "crate::icons::IconColor::is_default")]
    pub icon_color: crate::icons::IconColor,
    pub cover: u8,
    pub wide: bool,
    pub collection: Option<crate::collection::Collection>,
    pub properties: crate::collection::Properties,
    pub dashboard: Option<crate::dashboard::DashboardConfig>,
    #[serde(default)]
    pub order: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected: Option<crate::security::Sealed>,
}

impl Page {
    pub fn new(parent_id: Option<String>) -> Self {
        let mut page = Self {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id,
            title: "Untitled".into(),
            body_html: String::new(),
            plain_text: String::new(),
            favorite: false,
            archived: false,
            updated_at: now(),
            blocks: vec![NoteBlock::new(Kind::Text)],
            emoji: "📄".into(),
            icon_color: Default::default(),
            cover: 0,
            wide: false,
            collection: None,
            properties: crate::collection::Properties::default(),
            dashboard: None,
            order: 0,
            protected: None,
        };
        page.sync_document();
        page
    }

    pub fn display_title(&self) -> &str {
        if self.title.trim().is_empty() {
            "Untitled"
        } else {
            &self.title
        }
    }

    pub fn matches(&self, query: &str) -> bool {
        let fold = rune_kit::search::Fold::for_language("fr");
        fold.contains(&self.title, query) || fold.contains(&self.plain_text, query)
    }

    pub fn replace_html(&mut self, html: &str) {
        self.blocks = import_legacy(html);
        self.sync_document();
    }

    pub fn sync_document(&mut self) {
        self.body_html = crate::outline::html(&self.blocks);
        self.plain_text = self
            .blocks
            .iter()
            .map(NoteBlock::plain_text)
            .collect::<Vec<_>>()
            .join("\n");
    }

    pub fn content(&self) -> PageContent {
        PageContent {
            version: crate::document::CURRENT_VERSION,
            blocks: self.blocks.clone(),
            emoji: self.emoji.clone(),
            icon_color: self.icon_color,
            cover: self.cover,
            wide: self.wide,
            collection: self.collection.clone(),
            properties: self.properties.clone(),
            dashboard: self.dashboard.clone(),
            order: self.order,
            protected: self.protected.clone(),
        }
    }
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn welcome() -> Page {
    let mut page = Page::new(None);
    page.title = "Room for your ideas".into();
    page.body_html = concat!(
        "<p>Welcome to <b>Folio</b>, your local notebook.</p>",
        "<p>An idea, meeting notes, the first lines of a project: it all starts here.</p>",
        "<p></p>",
        "<p><b>Get started</b></p>",
        "<ul><li>Create a page or subpage from the sidebar.</li>",
        "<li>Write, select some text, and try the formatting toolbar.</li>",
        "<li>Find your notes with search, or add them to your favorites.</li></ul>",
        "<blockquote>Your notes stay on this computer. Changes are saved automatically.</blockquote>",
        "<p></p><p>Make room for your next idea.</p>"
    ).into();
    let html = page.body_html.clone();
    page.replace_html(&html);
    page.favorite = true;
    page
}
