//! Folio's monogram and 24-point line icons. Legacy emoji values keep their stored identity.
use rune::Icon;
mod colors;
pub mod controls;
pub use colors::IconColor;

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
glyph!(COMPASS, "compass");
glyph!(GLOBE, "globe");
glyph!(HOME, "home");
glyph!(BRIEFCASE, "briefcase");
glyph!(TERMINAL, "terminal");
glyph!(FLASK, "flask");
glyph!(PALETTE, "palette");
glyph!(CAMERA, "camera");
glyph!(MUSIC, "music");
glyph!(HEART, "heart");
glyph!(SPARKLES, "sparkles");
glyph!(COFFEE, "coffee");
glyph!(MOUNTAIN, "mountain");
glyph!(PLANT, "plant");
glyph!(GRADUATION, "graduation");
glyph!(WALLET, "wallet");
glyph!(TREND, "trend");
glyph!(LAYERS, "layers");
glyph!(PUZZLE, "puzzle");
glyph!(PEOPLE, "people");

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
        name: "Growing ideas",
        value: "🌿",
        glyph: &LEAF,
    },
    Choice {
        key: "idea",
        name: "Idea",
        value: "💡",
        glyph: &IDEA,
    },
    Choice {
        key: "note",
        name: "Notebook",
        value: "📝",
        glyph: &NOTE,
    },
    Choice {
        key: "project",
        name: "Project",
        value: "🚀",
        glyph: &PROJECT,
    },
    Choice {
        key: "tasks",
        name: "Tasks",
        value: "✅",
        glyph: &TASKS,
    },
    Choice {
        key: "library",
        name: "Library",
        value: "📚",
        glyph: &LIBRARY,
    },
    Choice {
        key: "target",
        name: "Goal",
        value: "🎯",
        glyph: &TARGET,
    },
    Choice {
        key: "database",
        name: "Database",
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
        name: "Calendar",
        value: "folio:calendar",
        glyph: &CALENDAR,
    },
    Choice {
        key: "folder",
        name: "Folder",
        value: "folio:folder",
        glyph: &FOLDER,
    },
    Choice {
        key: "bookmark",
        name: "Bookmark",
        value: "folio:bookmark",
        glyph: &BOOKMARK,
    },
    Choice {
        key: "link",
        name: "Link",
        value: "folio:link",
        glyph: &LINK,
    },
    Choice {
        key: "archive",
        name: "Archive",
        value: "folio:archive",
        glyph: &ARCHIVE,
    },
    Choice {
        key: "compass",
        name: "Compass",
        value: "folio:compass",
        glyph: &COMPASS,
    },
    Choice {
        key: "globe",
        name: "Globe",
        value: "folio:globe",
        glyph: &GLOBE,
    },
    Choice {
        key: "home",
        name: "Home",
        value: "folio:home",
        glyph: &HOME,
    },
    Choice {
        key: "briefcase",
        name: "Briefcase",
        value: "folio:briefcase",
        glyph: &BRIEFCASE,
    },
    Choice {
        key: "terminal",
        name: "Terminal",
        value: "folio:terminal",
        glyph: &TERMINAL,
    },
    Choice {
        key: "flask",
        name: "Flask",
        value: "folio:flask",
        glyph: &FLASK,
    },
    Choice {
        key: "palette",
        name: "Palette",
        value: "folio:palette",
        glyph: &PALETTE,
    },
    Choice {
        key: "camera",
        name: "Camera",
        value: "folio:camera",
        glyph: &CAMERA,
    },
    Choice {
        key: "music",
        name: "Music",
        value: "folio:music",
        glyph: &MUSIC,
    },
    Choice {
        key: "heart",
        name: "Heart",
        value: "folio:heart",
        glyph: &HEART,
    },
    Choice {
        key: "sparkles",
        name: "Sparkles",
        value: "folio:sparkles",
        glyph: &SPARKLES,
    },
    Choice {
        key: "coffee",
        name: "Coffee",
        value: "folio:coffee",
        glyph: &COFFEE,
    },
    Choice {
        key: "mountain",
        name: "Mountain",
        value: "folio:mountain",
        glyph: &MOUNTAIN,
    },
    Choice {
        key: "plant",
        name: "Plant",
        value: "folio:plant",
        glyph: &PLANT,
    },
    Choice {
        key: "graduation",
        name: "Graduation",
        value: "folio:graduation",
        glyph: &GRADUATION,
    },
    Choice {
        key: "wallet",
        name: "Wallet",
        value: "folio:wallet",
        glyph: &WALLET,
    },
    Choice {
        key: "trend",
        name: "Trend",
        value: "folio:trend",
        glyph: &TREND,
    },
    Choice {
        key: "layers",
        name: "Layers",
        value: "folio:layers",
        glyph: &LAYERS,
    },
    Choice {
        key: "puzzle",
        name: "Puzzle",
        value: "folio:puzzle",
        glyph: &PUZZLE,
    },
    Choice {
        key: "people",
        name: "People",
        value: "folio:people",
        glyph: &PEOPLE,
    },
];

pub fn choice(value: &str) -> Option<&'static Choice> {
    CHOICES.iter().find(|icon| {
        icon.value == value
            || value.strip_prefix("folio:") == Some(icon.key)
            || (icon.key == "database" && value == "🗂")
    })
}
