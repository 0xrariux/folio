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
        name: "Nombre",
        glyph: &NUMBER,
    },
    Symbol {
        key: "search",
        name: "Rechercher",
        glyph: &SEARCH,
    },
    Symbol {
        key: "plus",
        name: "Ajouter",
        glyph: &PLUS,
    },
    Symbol {
        key: "close",
        name: "Fermer",
        glyph: &CLOSE,
    },
    Symbol {
        key: "chevron-left",
        name: "Précédent",
        glyph: &CHEVRON_LEFT,
    },
    Symbol {
        key: "chevron-right",
        name: "Suivant",
        glyph: &CHEVRON_RIGHT,
    },
    Symbol {
        key: "chevron-down",
        name: "Déplier",
        glyph: &CHEVRON_DOWN,
    },
    Symbol {
        key: "arrow-up",
        name: "Monter",
        glyph: &ARROW_UP,
    },
    Symbol {
        key: "arrow-down",
        name: "Descendre",
        glyph: &ARROW_DOWN,
    },
    Symbol {
        key: "arrow-up-right",
        name: "Ouvrir le lien",
        glyph: &ARROW_UP_RIGHT,
    },
    Symbol {
        key: "undo",
        name: "Annuler",
        glyph: &UNDO,
    },
    Symbol {
        key: "redo",
        name: "Rétablir",
        glyph: &REDO,
    },
    Symbol {
        key: "grip",
        name: "Déplacer le bloc",
        glyph: &GRIP,
    },
    Symbol {
        key: "copy",
        name: "Dupliquer",
        glyph: &COPY,
    },
    Symbol {
        key: "trash",
        name: "Supprimer",
        glyph: &TRASH,
    },
    Symbol {
        key: "expand",
        name: "Pleine largeur",
        glyph: &EXPAND,
    },
    Symbol {
        key: "cover",
        name: "Couverture",
        glyph: &COVER,
    },
    Symbol {
        key: "export",
        name: "Exporter",
        glyph: &EXPORT,
    },
    Symbol {
        key: "sun",
        name: "Mode clair",
        glyph: &SUN,
    },
    Symbol {
        key: "moon",
        name: "Mode sombre",
        glyph: &MOON,
    },
    Symbol {
        key: "star",
        name: "Favori",
        glyph: &STAR,
    },
    Symbol {
        key: "star-filled",
        name: "Favori actif",
        glyph: &STAR_FILLED,
    },
    Symbol {
        key: "table",
        name: "Tableau",
        glyph: &TABLE,
    },
    Symbol {
        key: "board",
        name: "Kanban",
        glyph: &BOARD,
    },
    Symbol {
        key: "filter",
        name: "Filtrer",
        glyph: &FILTER,
    },
    Symbol {
        key: "sort",
        name: "Trier",
        glyph: &SORT,
    },
    Symbol {
        key: "columns",
        name: "Colonnes",
        glyph: &COLUMNS,
    },
    Symbol {
        key: "flag",
        name: "Priorité",
        glyph: &FLAG,
    },
    Symbol {
        key: "clock",
        name: "Échéance",
        glyph: &CLOCK,
    },
    Symbol {
        key: "alert",
        name: "Attention",
        glyph: &ALERT,
    },
    Symbol {
        key: "check",
        name: "Validé",
        glyph: &CHECK,
    },
    Symbol {
        key: "status-dot",
        name: "Statut",
        glyph: &STATUS_DOT,
    },
    Symbol {
        key: "text",
        name: "Texte",
        glyph: &TEXT,
    },
    Symbol {
        key: "heading-1",
        name: "Titre 1",
        glyph: &HEADING_1,
    },
    Symbol {
        key: "heading-2",
        name: "Titre 2",
        glyph: &HEADING_2,
    },
    Symbol {
        key: "heading-3",
        name: "Titre 3",
        glyph: &HEADING_3,
    },
    Symbol {
        key: "bullet-list",
        name: "Liste à puces",
        glyph: &BULLET_LIST,
    },
    Symbol {
        key: "numbered-list",
        name: "Liste numérotée",
        glyph: &NUMBERED_LIST,
    },
    Symbol {
        key: "quote",
        name: "Citation",
        glyph: &QUOTE,
    },
    Symbol {
        key: "code",
        name: "Code",
        glyph: &CODE,
    },
    Symbol {
        key: "divider",
        name: "Séparateur",
        glyph: &DIVIDER,
    },
    Symbol {
        key: "templates",
        name: "Modèles",
        glyph: &TEMPLATES,
    },
    Symbol {
        key: "more",
        name: "Autres actions",
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
