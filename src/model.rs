use std::time::{SystemTime, UNIX_EPOCH};

use crate::document::{Kind, NoteBlock, PageContent, import_legacy};

/// HTML is Rune's portable rich-text representation; plain text is the search index.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    pub cover: u8,
    pub wide: bool,
    pub collection: Option<crate::collection::Collection>,
    pub properties: crate::collection::Properties,
    pub dashboard: Option<crate::dashboard::DashboardConfig>,
    #[serde(default)]
    pub order: i64,
}

impl Page {
    pub fn new(parent_id: Option<String>) -> Self {
        let mut page = Self {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id,
            title: "Sans titre".into(),
            body_html: String::new(),
            plain_text: String::new(),
            favorite: false,
            archived: false,
            updated_at: now(),
            blocks: vec![NoteBlock::new(Kind::Text)],
            emoji: "📄".into(),
            cover: 0,
            wide: false,
            collection: None,
            properties: crate::collection::Properties::default(),
            dashboard: None,
            order: 0,
        };
        page.sync_document();
        page
    }

    pub fn display_title(&self) -> &str {
        if self.title.trim().is_empty() {
            "Sans titre"
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
            version: 6,
            blocks: self.blocks.clone(),
            emoji: self.emoji.clone(),
            cover: self.cover,
            wide: self.wide,
            collection: self.collection.clone(),
            properties: self.properties.clone(),
            dashboard: self.dashboard.clone(),
            order: self.order,
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
    page.title = "Un peu d’espace pour vos idées".into();
    page.body_html = concat!(
        "<p>Bienvenue dans <b>Folio</b>, votre carnet personnel.</p>",
        "<p>Une idée, une note de réunion, les premières lignes d’un projet : tout commence ici.</p>",
        "<p></p>",
        "<p><b>À vous de jouer</b></p>",
        "<ul><li>Créez une page ou une sous-page depuis la barre latérale.</li>",
        "<li>Écrivez, sélectionnez du texte et essayez la barre de mise en forme.</li>",
        "<li>Retrouvez vos notes avec la recherche, ou ajoutez-les aux favoris.</li></ul>",
        "<blockquote>Vos idées restent sur cet ordinateur. Chaque modification est enregistrée automatiquement.</blockquote>",
        "<p></p><p>Faites de la place à la prochaine idée.</p>"
    ).into();
    let html = page.body_html.clone();
    page.replace_html(&html);
    page.favorite = true;
    page
}
