use crate::editor::{EditorBlock, Snapshot};
use crate::worker::{Event, Worker};
use folio::model::{Page, now};
use rune::TextEdit;
use std::collections::{HashMap, HashSet};

pub enum Phase {
    Loading,
    Ready,
    Failed(String),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Favorites,
    Archive,
}
#[derive(Clone, Default)]
pub enum Panel {
    #[default]
    Closed,
    Settings,
    Library,
    ModuleReview,
    ModuleConfirm,
    Security,
    ConfirmDelete(String),
    Slash {
        id: String,
        selected: usize,
        typed: bool,
    },
    Block(String),
    Page,
    SidebarPage(String),
    Move,
    Templates,
    Icons,
    CollectionFilter,
    CollectionSort,
    Property {
        id: String,
        field: crate::collections::PropertyField,
        anchor: String,
    },
    Properties(Box<crate::collections::PropertyForm>),
    Link(Box<crate::linking::LinkForm>),
    Columns(Box<crate::custom_fields::ColumnForm>),
    Cell(Box<crate::custom_fields::CellForm>),
    DashboardSources,
    Views(Box<crate::saved_views::ViewForm>),
    SidebarMore,
    SidebarNavigation,
    Workspace,
    History,
    RestorePreview,
    Palette(Box<crate::command_palette::Form>),
    Date(Box<crate::calendar::DatePicker>),
    CalendarDay(String),
}
pub struct Pending {
    pub revision: u64,
    pub error: Option<String>,
}
pub struct State {
    pub overview: crate::overview::Overview,
    pub library: crate::module_library::Library,
    pub settings: crate::settings::Settings,
    pub workspace: crate::notebook::WorkspaceState,
    pub selection: crate::selection::Selection,
    pub pages: Vec<Page>,
    pub selected: Option<String>,
    pub title: TextEdit,
    pub blocks: Vec<EditorBlock>,
    pub active: Option<String>,
    pub block_drag: crate::block_drag::BlockDrag,
    pub search: TextEdit,
    pub collection_search: TextEdit,
    pub filter: Filter,
    pub dark: bool,
    pub phase: Phase,
    pub panel: Panel,
    pub collapsed: HashSet<String>,
    pub pending: HashMap<String, Pending>,
    pub worker: Option<Worker>,
    pub undo: Vec<Snapshot>,
    pub redo: Vec<Snapshot>,
    pub history: Vec<String>,
    pub history_at: usize,
    revision: u64,
    pub demo: bool,
    pub today: String,
    pub export_task: Option<crate::exporting::ExportTask>,
    pub export_wake: Option<rune::Waker>,
    pub export_busy: bool,
    pub export_message: Option<String>,
}
impl State {
    pub fn new() -> Self {
        Self {
            overview: Default::default(),
            library: Default::default(),
            settings: crate::settings::Settings::default(),
            workspace: crate::notebook::WorkspaceState::default(),
            selection: crate::selection::Selection::default(),
            pages: Vec::new(),
            selected: None,
            title: TextEdit::new(""),
            blocks: Vec::new(),
            active: None,
            block_drag: crate::block_drag::BlockDrag::default(),
            search: TextEdit::new(""),
            collection_search: TextEdit::new(""),
            filter: Filter::All,
            dark: true,
            phase: Phase::Loading,
            panel: Panel::Closed,
            collapsed: HashSet::new(),
            pending: HashMap::new(),
            worker: None,
            undo: Vec::new(),
            redo: Vec::new(),
            history: Vec::new(),
            history_at: 0,
            revision: 0,
            demo: false,
            today: folio::dashboard::local_today(),
            export_task: None,
            export_wake: None,
            export_busy: false,
            export_message: None,
        }
    }
    pub fn demo() -> Self {
        use folio::{
            document::{Kind, NoteBlock},
            templates::Template,
        };
        let mut state = Self::new();
        state.demo = true;
        state.library.loaded = true;
        state.today = "2026-11-10".into();
        state.phase = Phase::Ready;
        let mut home = Page::new(None);
        home.id = "demo-home".into();
        home.title = "Room for your ideas".into();
        home.emoji = "🌿".into();
        home.cover = 1;
        home.favorite = true;
        home.blocks = vec![
            NoteBlock::text(
                Kind::Callout,
                "Welcome to Folio. A notebook to think, create, and move your projects forward.",
            ),
            NoteBlock::text(Kind::Heading2, "Make room for your next idea"),
            NoteBlock::text(
                Kind::Text,
                "Meeting notes, the start of a project, or a thought to keep. Move blocks to organize your ideas.",
            ),
            NoteBlock::text(Kind::Todo, "Try slash commands to add a block"),
            NoteBlock::text(Kind::Todo, "Create a page from a template"),
            NoteBlock::text(Kind::Todo, "Take the next step on a project"),
            NoteBlock::text(Kind::Divider, ""),
            NoteBlock::text(Kind::Quote, "Great projects often start with a small note."),
        ];
        home.blocks[3].checked = true;
        home.sync_document();
        let mut ideas = Page::new(None);
        ideas.id = "demo-ideas".into();
        ideas.title = "Ideas".into();
        ideas.emoji = "💡".into();
        ideas.replace_html("<p>A place to collect ideas worth exploring.</p>");
        let mut child = Template::Project.create();
        child.id = "demo-project".into();
        child.parent_id = Some(ideas.id.clone());
        child.title = "Next project".into();
        let mut meeting = Template::Meeting.create();
        meeting.id = "demo-meeting".into();
        meeting.favorite = true;
        state.pages = vec![home, ideas, child, meeting];
        let mut tracker = Template::Tracker.create();
        tracker.id = "demo-tracker".into();
        tracker.title = "Project tracker".into();
        tracker.favorite = true;
        let mut link = NoteBlock::text(Kind::PageLink, &tracker.title);
        link.target_id = Some(tracker.id.clone());
        state.pages[0].blocks.push(link);
        state.pages[0].sync_document();
        state.pages.push(tracker);
        use folio::collection::{Priority, Status};
        for (i, (title, status, priority, due, tags)) in [
            (
                "Plan a new project",
                Status::Todo,
                Priority::Medium,
                "2026-11-20",
                "Ideas",
            ),
            (
                "Sketch the first screens",
                Status::InProgress,
                Priority::High,
                "2026-11-14",
                "Design, Sample",
            ),
            (
                "Connect related notes",
                Status::InProgress,
                Priority::High,
                "2026-11-16",
                "Sample",
            ),
            (
                "Lay the foundations",
                Status::Done,
                Priority::Low,
                "2026-11-10",
                "Development",
            ),
            (
                "Gather initial feedback",
                Status::Todo,
                Priority::Medium,
                "2026-11-22",
                "Product",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mut page = Page::new(Some("demo-tracker".into()));
            page.id = format!("demo-record-{i}");
            page.title = title.into();
            page.properties.status = status;
            page.properties.priority = priority;
            page.properties.due = Some(due.into());
            page.properties.tags = tags.split(", ").map(str::to_string).collect();
            page.replace_html("<p>Notes, decisions, and next steps for this record.</p>");
            state.pages.push(page);
        }
        state.collapsed.insert("demo-tracker".into());
        for page in &mut state.pages {
            for (i, block) in page.blocks.iter_mut().enumerate() {
                block.id = format!("{}-{i}", page.id);
            }
        }
        state.extend_demo_tracking();
        let first = state.pages[0].id.clone();
        state.select(&first);
        state
    }
    pub fn current(&self) -> Option<&Page> {
        self.pages
            .iter()
            .find(|p| Some(&p.id) == self.selected.as_ref())
    }
    pub fn edit(&mut self) {
        if self.settings.locked || self.current().is_some_and(|p| p.protected.is_some()) {
            return;
        }
        self.commit_blocks(true);
        let Some(id) = self.selected.clone() else {
            return;
        };
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            if page.title == self.title.text {
                return;
            }
            page.title.clone_from(&self.title.text);
            page.updated_at = now();
        }
        self.persist(&id);
        self.refresh_references(&id);
    }
    pub fn favorite(&mut self) {
        let Some(id) = self.selected.clone() else {
            return;
        };
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            page.favorite = !page.favorite;
            page.updated_at = now();
        }
        self.persist(&id);
    }
    pub fn archive(&mut self) {
        let Some(id) = self.selected.clone() else {
            return;
        };
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            page.archived = !page.archived;
            page.updated_at = now();
        }
        self.panel = Panel::Closed;
        self.persist(&id);
    }
    pub fn persist(&mut self, id: &str) {
        if self.demo {
            return;
        }
        let Some(page) = self.pages.iter().find(|p| p.id == id).cloned() else {
            return;
        };
        self.revision += 1;
        let revision = self.revision;
        let error = self
            .worker
            .as_ref()
            .ok_or_else(|| "The backup service is unavailable".to_string())
            .and_then(|worker| worker.save(page, revision))
            .err();
        self.pending.insert(id.into(), Pending { revision, error });
    }
    pub fn retry(&mut self) {
        let ids: Vec<_> = self.pending.keys().cloned().collect();
        for id in ids {
            self.persist(&id);
        }
    }
    pub fn accept(&mut self, event: Event) {
        match event {
            Event::Security(outcome) => self.accept_security(outcome),
            Event::SecurityFailed(message) => {
                self.settings.busy = false;
                self.settings.pending_security = None;
                self.settings.error = Some(message);
            }
            Event::Workspace(result) => self.accept_workspace(result),
            Event::Loaded(pages) => {
                self.pages = pages;
                self.pages.sort_by_key(|p| p.order);
                self.phase = Phase::Ready;
                if let Some(id) = self
                    .pages
                    .iter()
                    .find(|p| !p.archived && !self.legacy_overview(p))
                    .map(|p| p.id.clone())
                {
                    self.select(&id);
                }
            }
            Event::LoadFailed(message) => self.phase = Phase::Failed(message),
            Event::Saved { id, revision } => {
                if self
                    .pending
                    .get(&id)
                    .is_some_and(|p| p.revision == revision)
                {
                    self.pending.remove(&id);
                }
            }
            Event::SaveFailed {
                id,
                revision,
                message,
            } => {
                if let Some(p) = self.pending.get_mut(&id).filter(|p| p.revision == revision) {
                    p.error = Some(message);
                }
            }
        }
    }
    pub fn visible(&self, page: &Page) -> bool {
        let scope = match self.filter {
            Filter::All => !page.archived,
            Filter::Favorites => !page.archived && page.favorite,
            Filter::Archive => page.archived,
        };
        scope && page.matches(&self.search.text)
    }
    pub fn sidebar_pages(&self) -> Vec<(&Page, usize)> {
        let visible: Vec<_> = self
            .pages
            .iter()
            .filter(|page| {
                self.visible(page)
                    && (self.filter == Filter::Archive
                        || !self.search.text.trim().is_empty()
                        || !self.legacy_overview(page))
            })
            .collect();
        if !self.search.text.trim().is_empty() || self.filter != Filter::All {
            return visible.into_iter().map(|p| (p, 0)).collect();
        }
        fn walk<'a>(
            page: &'a Page,
            visible: &[&'a Page],
            depth: usize,
            collapsed: &HashSet<String>,
            visited: &mut HashSet<String>,
            rows: &mut Vec<(&'a Page, usize)>,
        ) {
            if !visited.insert(page.id.clone()) {
                return;
            }
            rows.push((page, depth));
            if collapsed.contains(&page.id) {
                return;
            }
            for child in visible
                .iter()
                .filter(|p| p.parent_id.as_deref() == Some(&page.id))
            {
                walk(child, visible, depth + 1, collapsed, visited, rows);
            }
        }
        let mut rows = Vec::new();
        let mut visited = HashSet::new();
        for page in &visible {
            if !visible
                .iter()
                .any(|p| Some(&p.id) == page.parent_id.as_ref())
            {
                walk(page, &visible, 0, &self.collapsed, &mut visited, &mut rows);
            }
        }
        for page in &visible {
            if self.has_ancestor_cycle(&page.id) {
                walk(page, &visible, 0, &self.collapsed, &mut visited, &mut rows);
            }
        }
        rows
    }
}
