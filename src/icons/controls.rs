//! Command and block pictograms, separate from the icons stored on pages.
use crate::document::Kind;
use rune::Icon;

macro_rules! glyph {
    ($name:ident, $file:literal) => {
        pub static $name: Icon = Icon {
            name: concat!("folio-", $file),
            view: 24.0,
            body: include_str!(concat!("../../assets/icons/", $file, ".svg")),
        };
    };
}
glyph!(SEARCH, "search");
glyph!(SETTINGS, "settings");
glyph!(LOCK, "lock");
glyph!(PLUS, "plus");
glyph!(CLOSE, "close");
glyph!(CHEVRON_LEFT, "chevron-left");
glyph!(CHEVRON_RIGHT, "chevron-right");
glyph!(CHEVRON_DOWN, "chevron-down");
glyph!(ARROW_UP, "arrow-up");
glyph!(ARROW_DOWN, "arrow-down");
glyph!(ARROW_UP_RIGHT, "arrow-up-right");
glyph!(UNDO, "undo");
glyph!(REDO, "redo");
glyph!(GRIP, "grip");
glyph!(COPY, "copy");
glyph!(TRASH, "trash");
glyph!(EXPAND, "expand");
glyph!(COVER, "cover");
glyph!(EXPORT, "export");
glyph!(SUN, "sun");
glyph!(MOON, "moon");
glyph!(STAR, "star");
glyph!(STAR_FILLED, "star-filled");
glyph!(TABLE, "table");
glyph!(BOARD, "board");
glyph!(FILTER, "filter");
glyph!(SORT, "sort");
glyph!(COLUMNS, "columns");
glyph!(FLAG, "flag");
glyph!(CLOCK, "clock");
glyph!(ALERT, "alert");
glyph!(CHECK, "check");
glyph!(STATUS_DOT, "status-dot");
glyph!(TEXT, "text");
glyph!(HEADING_1, "heading-1");
glyph!(HEADING_2, "heading-2");
glyph!(HEADING_3, "heading-3");
glyph!(BULLET_LIST, "bullet-list");
glyph!(NUMBERED_LIST, "numbered-list");
glyph!(QUOTE, "quote");
glyph!(CODE, "code");
glyph!(DIVIDER, "divider");
glyph!(TEMPLATES, "templates");
glyph!(MORE, "more");
glyph!(NUMBER, "number");

pub struct Symbol {
    pub key: &'static str,
    pub name: &'static str,
    pub glyph: &'static Icon,
}
pub const ALL: &[Symbol] = &[
    Symbol {
        key: "number",
        name: "Number",
        glyph: &NUMBER,
    },
    Symbol {
        key: "search",
        name: "Search",
        glyph: &SEARCH,
    },
    Symbol {
        key: "plus",
        name: "Add",
        glyph: &PLUS,
    },
    Symbol {
        key: "close",
        name: "Close",
        glyph: &CLOSE,
    },
    Symbol {
        key: "chevron-left",
        name: "Back",
        glyph: &CHEVRON_LEFT,
    },
    Symbol {
        key: "chevron-right",
        name: "Next",
        glyph: &CHEVRON_RIGHT,
    },
    Symbol {
        key: "chevron-down",
        name: "Expand",
        glyph: &CHEVRON_DOWN,
    },
    Symbol {
        key: "arrow-up",
        name: "Move up",
        glyph: &ARROW_UP,
    },
    Symbol {
        key: "arrow-down",
        name: "Move down",
        glyph: &ARROW_DOWN,
    },
    Symbol {
        key: "arrow-up-right",
        name: "Open link",
        glyph: &ARROW_UP_RIGHT,
    },
    Symbol {
        key: "undo",
        name: "Undo",
        glyph: &UNDO,
    },
    Symbol {
        key: "redo",
        name: "Redo",
        glyph: &REDO,
    },
    Symbol {
        key: "grip",
        name: "Move block",
        glyph: &GRIP,
    },
    Symbol {
        key: "copy",
        name: "Duplicate",
        glyph: &COPY,
    },
    Symbol {
        key: "trash",
        name: "Delete",
        glyph: &TRASH,
    },
    Symbol {
        key: "expand",
        name: "Full width",
        glyph: &EXPAND,
    },
    Symbol {
        key: "cover",
        name: "Cover",
        glyph: &COVER,
    },
    Symbol {
        key: "export",
        name: "Export",
        glyph: &EXPORT,
    },
    Symbol {
        key: "sun",
        name: "Light mode",
        glyph: &SUN,
    },
    Symbol {
        key: "moon",
        name: "Dark mode",
        glyph: &MOON,
    },
    Symbol {
        key: "star",
        name: "Favorite",
        glyph: &STAR,
    },
    Symbol {
        key: "star-filled",
        name: "Favorited",
        glyph: &STAR_FILLED,
    },
    Symbol {
        key: "table",
        name: "Table",
        glyph: &TABLE,
    },
    Symbol {
        key: "board",
        name: "Kanban",
        glyph: &BOARD,
    },
    Symbol {
        key: "filter",
        name: "Filter",
        glyph: &FILTER,
    },
    Symbol {
        key: "sort",
        name: "Sort",
        glyph: &SORT,
    },
    Symbol {
        key: "columns",
        name: "Columns",
        glyph: &COLUMNS,
    },
    Symbol {
        key: "flag",
        name: "Priority",
        glyph: &FLAG,
    },
    Symbol {
        key: "clock",
        name: "Due date",
        glyph: &CLOCK,
    },
    Symbol {
        key: "alert",
        name: "Warning",
        glyph: &ALERT,
    },
    Symbol {
        key: "check",
        name: "Approved",
        glyph: &CHECK,
    },
    Symbol {
        key: "status-dot",
        name: "Status",
        glyph: &STATUS_DOT,
    },
    Symbol {
        key: "text",
        name: "Text",
        glyph: &TEXT,
    },
    Symbol {
        key: "heading-1",
        name: "Heading 1",
        glyph: &HEADING_1,
    },
    Symbol {
        key: "heading-2",
        name: "Heading 2",
        glyph: &HEADING_2,
    },
    Symbol {
        key: "heading-3",
        name: "Heading 3",
        glyph: &HEADING_3,
    },
    Symbol {
        key: "bullet-list",
        name: "Bulleted list",
        glyph: &BULLET_LIST,
    },
    Symbol {
        key: "numbered-list",
        name: "Numbered list",
        glyph: &NUMBERED_LIST,
    },
    Symbol {
        key: "quote",
        name: "Quote",
        glyph: &QUOTE,
    },
    Symbol {
        key: "code",
        name: "Code",
        glyph: &CODE,
    },
    Symbol {
        key: "divider",
        name: "Divider",
        glyph: &DIVIDER,
    },
    Symbol {
        key: "templates",
        name: "Templates",
        glyph: &TEMPLATES,
    },
    Symbol {
        key: "more",
        name: "More actions",
        glyph: &MORE,
    },
];

pub fn block(kind: Kind) -> &'static Icon {
    match kind {
        Kind::Text => &TEXT,
        Kind::Heading1 => &HEADING_1,
        Kind::Heading2 => &HEADING_2,
        Kind::Heading3 => &HEADING_3,
        Kind::Bullet => &BULLET_LIST,
        Kind::Numbered => &NUMBERED_LIST,
        Kind::Todo => &super::TASKS,
        Kind::Toggle => &CHEVRON_RIGHT,
        Kind::ToggleHeading1 => &HEADING_1,
        Kind::ToggleHeading2 => &HEADING_2,
        Kind::ToggleHeading3 => &HEADING_3,
        Kind::Quote => &QUOTE,
        Kind::Callout => &super::IDEA,
        Kind::Code => &CODE,
        Kind::Divider => &DIVIDER,
        Kind::PageLink => &super::LINK,
        Kind::DatabaseView => &super::DATABASE,
        Kind::Image => &COVER,
        Kind::Attachment => &super::LINK,
        Kind::TableOfContents => &BULLET_LIST,
    }
}
