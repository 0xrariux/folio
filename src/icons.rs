//! Folio's monogram and 24-point line icons. Legacy emoji values keep their stored identity.
use rune::Icon;
pub mod controls;

macro_rules! glyph {
    ($name:ident, $file:literal) => {
        pub static $name: Icon = Icon {
            name: concat!("folio-", $file),
            view: 24.0,
            body: include_str!(concat!("../assets/icons/", $file, ".svg")),
        };
    };
}
glyph!(PAGE, "page");
glyph!(LEAF, "leaf");
glyph!(IDEA, "idea");
glyph!(NOTE, "note");
glyph!(PROJECT, "project");
glyph!(TASKS, "tasks");
glyph!(LIBRARY, "library");
glyph!(TARGET, "target");
glyph!(DATABASE, "database");
glyph!(DASHBOARD, "dashboard");
glyph!(CALENDAR, "calendar");
glyph!(FOLDER, "folder");
glyph!(BOOKMARK, "bookmark");
glyph!(LINK, "link");
glyph!(ARCHIVE, "archive");
glyph!(FOLIO, "folio");

pub struct Choice {
    pub key: &'static str,
    pub name: &'static str,
    pub value: &'static str,
    pub glyph: &'static Icon,
}
pub const CHOICES: &[Choice] = &[
    Choice {
        key: "page",
        name: "Page",
        value: "📄",
        glyph: &PAGE,
    },
    Choice {
        key: "leaf",
        name: "Idées en germe",
        value: "🌿",
        glyph: &LEAF,
    },
    Choice {
        key: "idea",
        name: "Idée",
        value: "💡",
        glyph: &IDEA,
    },
    Choice {
        key: "note",
        name: "Carnet",
        value: "📝",
        glyph: &NOTE,
    },
    Choice {
        key: "project",
        name: "Projet",
        value: "🚀",
        glyph: &PROJECT,
    },
    Choice {
        key: "tasks",
        name: "Tâches",
        value: "✅",
        glyph: &TASKS,
    },
    Choice {
        key: "library",
        name: "Bibliothèque",
        value: "📚",
        glyph: &LIBRARY,
    },
    Choice {
        key: "target",
        name: "Objectif",
        value: "🎯",
        glyph: &TARGET,
    },
    Choice {
        key: "database",
        name: "Base de données",
        value: "🗂️",
        glyph: &DATABASE,
    },
    Choice {
        key: "dashboard",
        name: "Dashboard",
        value: "📊",
        glyph: &DASHBOARD,
    },
    Choice {
        key: "calendar",
        name: "Calendrier",
        value: "folio:calendar",
        glyph: &CALENDAR,
    },
    Choice {
        key: "folder",
        name: "Dossier",
        value: "folio:folder",
        glyph: &FOLDER,
    },
    Choice {
        key: "bookmark",
        name: "Repère",
        value: "folio:bookmark",
        glyph: &BOOKMARK,
    },
    Choice {
        key: "link",
        name: "Lien",
        value: "folio:link",
        glyph: &LINK,
    },
    Choice {
        key: "archive",
        name: "Archive",
        value: "folio:archive",
        glyph: &ARCHIVE,
    },
];

pub fn choice(value: &str) -> Option<&'static Choice> {
    CHOICES.iter().find(|icon| {
        icon.value == value
            || value.strip_prefix("folio:") == Some(icon.key)
            || (icon.key == "database" && value == "🗂")
    })
}
