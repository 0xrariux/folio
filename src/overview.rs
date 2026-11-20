use crate::state::{Panel, Phase, State};
use folio::{
    document::{Kind, NoteBlock},
    model::Page,
    outline,
};
use rune::{TextEdit, Tx};
use rune_kit::search::Fold;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Tasks {
    #[default]
    All,
    Open,
    Done,
}

pub struct Overview {
    pub open: bool,
    pub search: TextEdit,
    pub tasks: Tasks,
    pub limit: usize,
}
impl Default for Overview {
    fn default() -> Self {
        Self {
            open: false,
            search: TextEdit::new(""),
            tasks: Tasks::All,
            limit: 40,
        }
    }
}

pub struct Item<'a> {
    pub page: &'a Page,
    pub block: &'a NoteBlock,
}
#[derive(Default)]
pub struct Summary<'a> {
    pub notes: Vec<&'a Page>,
    pub tasks: Vec<Item<'a>>,
    pub lists: Vec<Item<'a>>,
    pub databases: Vec<&'a Page>,
    pub done: usize,
}

impl State {
    pub fn open_dashboard(&mut self) {
        if !matches!(self.phase, Phase::Ready)
            || self.settings.locked
            || self.settings.busy
            || self.workspace.busy
        {
            return;
        }
        self.edit();
        // Leaving a PIN-unlocked note must lock it just as opening Settings does.
        self.open_settings();
        self.panel = Panel::Closed;
        self.selection.clear();
        self.block_drag = Default::default();
        self.selected = None;
        self.active = None;
        self.title = TextEdit::new("");
        self.blocks.clear();
        self.undo.clear();
        self.redo.clear();
        self.overview.open = true;
    }

    pub fn close_overview(&mut self) {
        let previous = self
            .history
            .get(self.history_at)
            .filter(|id| {
                self.pages
                    .iter()
                    .any(|p| p.id == **id && !p.archived && !self.legacy_overview(p))
            })
            .cloned()
            .or_else(|| {
                self.pages
                    .iter()
                    .find(|p| !p.archived && !self.legacy_overview(p))
                    .map(|p| p.id.clone())
            });
        self.overview.open = false;
        if let Some(id) = previous {
            self.select(&id);
        }
    }

    /// Older shortcut versions saved a blank dashboard page. Hide that exact
    /// shape in the regular tree, retaining it in storage and sidebar search.
    pub fn legacy_overview(&self, page: &Page) -> bool {
        page.title == "Overview"
            && page.emoji == "📊"
            && page.wide
            && page.parent_id.is_none()
            && !page.favorite
            && page.protected.is_none()
            && page.dashboard.is_some()
            && page.collection.is_none()
            && page.properties == Default::default()
            && page.cover == 0
            && page.blocks.len() == 1
            && page.plain_text.trim().is_empty()
            && page.blocks[0].kind == Kind::Text
            && page.blocks[0].plain_text().trim().is_empty()
            && page.blocks[0].asset.is_none()
            && page.blocks[0].embedded.is_none()
            && page.blocks[0].target_id.is_none()
            && !self
                .pages
                .iter()
                .any(|p| p.parent_id.as_deref() == Some(&page.id))
    }

    pub fn notebook_overview(&self) -> Summary<'_> {
        let mut summary = Summary::default();
        if self.settings.locked {
            return summary;
        }
        let query = self.overview.search.text.trim();
        let fold = Fold::for_language("fr");
        for page in self.pages.iter().filter(|p| {
            !p.archived && p.protected.is_none() && !self.settings.unlocked.contains(&p.id)
        }) {
            if page.dashboard.is_some() {
                continue;
            }
            if page.collection.is_some() {
                if page.matches(query) {
                    summary.databases.push(page);
                }
                continue;
            }
            if page.matches(query) {
                summary.notes.push(page);
            }
            let progress = outline::task_progress(&page.blocks);
            for (block, (_, children)) in page.blocks.iter().zip(progress) {
                let task = block.kind == Kind::Todo && children == 0;
                let list = matches!(block.kind, Kind::Bullet | Kind::Numbered);
                if !task && !list {
                    continue;
                }
                if !query.is_empty()
                    && !fold.contains(page.display_title(), query)
                    && !fold.contains(&block.plain_text(), query)
                {
                    continue;
                }
                if task {
                    summary.done += usize::from(block.checked);
                    summary.tasks.push(Item { page, block });
                } else {
                    summary.lists.push(Item { page, block });
                }
            }
        }
        summary.notes.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        summary
    }

    pub fn open_overview_item(&mut self, page: &str, block: Option<&str>, tx: &mut Tx) {
        if self.settings.locked
            || self.protected_page(page)
            || !self.pages.iter().any(|p| p.id == page && !p.archived)
        {
            return;
        }
        self.select(page);
        if let Some(block) = block {
            self.goto_block(block, tx);
        } else {
            tx.scroll_to_y("document-scroll", 0.0);
        }
    }
}
