use crate::document::{Kind, NoteBlock};
use crate::model::Page;

#[derive(Clone, Copy)]
pub enum Template {
    Meeting,
    Project,
    Tasks,
    Tracker,
    Dashboard,
    Outline,
}

impl Template {
    pub const ALL: [Self; 6] = [
        Self::Meeting,
        Self::Project,
        Self::Tasks,
        Self::Tracker,
        Self::Dashboard,
        Self::Outline,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Meeting => "Meeting notes",
            Self::Project => "New project",
            Self::Tasks => "Task list",
            Self::Tracker => "Project tracker",
            Self::Dashboard => "Project dashboard",
            Self::Outline => "Outline notebook",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Meeting => "An agenda, decisions, and action items",
            Self::Project => "From context to next steps",
            Self::Tasks => "Keep track of what needs to happen next",
            Self::Tracker => "A collection of records in a table or Kanban board",
            Self::Dashboard => "Metrics from all your collections in one place",
            Self::Outline => "Organize ideas, steps, and tasks in collapsible sections",
        }
    }

    pub fn create(self) -> Page {
        let mut page = Page::new(None);
        page.title = self.name().into();
        let block = NoteBlock::text;
        page.blocks = match self {
            Self::Outline => {
                page.emoji = "📚".into();
                let mut blocks = vec![
                    block(
                        Kind::Callout,
                        "Expand the arrows to explore. Press Enter in a collapsible heading to add content; use Tab and Shift+Tab to organize nested items.",
                    ),
                    block(Kind::ToggleHeading2, "A project, step by step"),
                    block(Kind::Text, "A place for context and key ideas."),
                    block(Kind::Numbered, "Prepare the groundwork"),
                    block(Kind::Bullet, "Gather ideas and resources"),
                    block(Kind::Bullet, "Define what matters"),
                    block(Kind::Numbered, "Build and share"),
                    block(Kind::Toggle, "Launch checklist"),
                    block(Kind::Todo, "Lay the foundations"),
                    block(Kind::Todo, "Refine the first version"),
                    block(Kind::Todo, "Gather initial feedback"),
                    block(Kind::ToggleHeading2, "Ideas for later"),
                    block(Kind::Quote, "A small idea can grow into a great project."),
                    block(Kind::Bullet, "An idea to explore tomorrow"),
                    block(
                        Kind::Text,
                        "Continue your notes here, outside the collapsible sections.",
                    ),
                ];
                for (b, level) in blocks
                    .iter_mut()
                    .zip([0, 0, 1, 1, 2, 2, 1, 1, 2, 2, 2, 0, 1, 1, 0])
                {
                    b.indent = level;
                }
                blocks[8].checked = true;
                blocks[11].collapsed = true;
                blocks
            }
            Self::Dashboard => {
                page.emoji = "📊".into();
                page.wide = true;
                page.dashboard = Some(crate::dashboard::DashboardConfig::default());
                vec![block(Kind::Text, "")]
            }
            Self::Tracker => {
                page.emoji = "🗂️".into();
                page.wide = true;
                page.collection = Some(crate::collection::Collection {
                    view: crate::collection::View::Board,
                    ..Default::default()
                });
                vec![block(Kind::Text, "")]
            }
            Self::Meeting => {
                page.emoji = "📝".into();
                page.cover = 2;
                vec![
                    block(
                        Kind::Callout,
                        "Goal: what should we have decided by the end of this meeting?",
                    ),
                    block(Kind::Heading2, "Agenda"),
                    block(Kind::Bullet, "Topics to discuss"),
                    block(Kind::Heading2, "Notes & decisions"),
                    block(Kind::Text, "Capture key ideas as the discussion unfolds."),
                    block(Kind::Heading2, "Action items"),
                    block(Kind::Todo, "One action, one owner, one deadline"),
                ]
            }
            Self::Project => {
                page.emoji = "🚀".into();
                page.cover = 1;
                vec![
                    block(Kind::Callout, "A clear goal and practical next steps."),
                    block(Kind::Heading2, "Why this project?"),
                    block(
                        Kind::Text,
                        "The problem to solve and the people this project will help.",
                    ),
                    block(Kind::Heading2, "Next steps"),
                    block(Kind::Todo, "Define the scope"),
                    block(Kind::Todo, "Build a first version"),
                    block(Kind::Todo, "Gather initial feedback"),
                    block(Kind::Divider, ""),
                    block(Kind::Heading3, "Resources & ideas"),
                    block(Kind::Text, "Useful links and notes for this project."),
                ]
            }
            Self::Tasks => {
                page.emoji = "✅".into();
                page.cover = 3;
                vec![
                    block(Kind::Heading2, "Today"),
                    block(Kind::Todo, "The most important task"),
                    block(Kind::Todo, "One small step forward"),
                    block(Kind::Heading2, "Later"),
                    block(Kind::Todo, "An idea for tomorrow"),
                ]
            }
        };
        page.sync_document();
        page
    }
}
