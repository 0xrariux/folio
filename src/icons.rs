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
glyph!(BOLT, "bolt");
glyph!(CLOUD, "cloud");
glyph!(UMBRELLA, "umbrella");
glyph!(FLAME, "flame");
glyph!(DROPLET, "droplet");
glyph!(MOON_STARS, "moon-stars");
glyph!(HEADPHONES, "headphones");
glyph!(MICROPHONE, "microphone");
glyph!(FILM, "film");
glyph!(GAMEPAD, "gamepad");
glyph!(BRUSH, "brush");
glyph!(PEN_TOOL, "pen-tool");
glyph!(SCISSORS, "scissors");
glyph!(RULER, "ruler");
glyph!(PENCIL, "pencil");
glyph!(MAP, "map");
glyph!(PLANE, "plane");
glyph!(CAR, "car");
glyph!(BICYCLE, "bicycle");
glyph!(SHIP, "ship");
glyph!(TREE, "tree");
glyph!(FLOWER, "flower");
glyph!(PAW, "paw");
glyph!(FISH, "fish");
glyph!(FRUIT, "fruit");
glyph!(CAKE, "cake");
glyph!(GIFT, "gift");
glyph!(TROPHY, "trophy");
glyph!(SHIELD, "shield");
glyph!(KEY, "key");

glyph!(COINS, "coins");
glyph!(BANK, "bank");
glyph!(CREDIT_CARD, "credit-card");
glyph!(RECEIPT, "receipt");
glyph!(CALCULATOR, "calculator");
glyph!(MEDICAL_KIT, "medical-kit");
glyph!(STETHOSCOPE, "stethoscope");
glyph!(PILL, "pill");
glyph!(ACTIVITY, "activity");
glyph!(BRAIN, "brain");
glyph!(MICROSCOPE, "microscope");
glyph!(ATOM, "atom");
glyph!(DNA, "dna");
glyph!(TELESCOPE, "telescope");
glyph!(MAGNET, "magnet");
glyph!(THEATRE, "theatre");
glyph!(PICTURE, "picture");
glyph!(EASEL, "easel");
glyph!(SCULPTURE, "sculpture");
glyph!(SWATCHES, "swatches");
glyph!(GUITAR, "guitar");
glyph!(PIANO, "piano");
glyph!(DRUM, "drum");
glyph!(SPEAKER, "speaker");
glyph!(WAVEFORM, "waveform");
glyph!(DICE, "dice");
glyph!(CHESS_KNIGHT, "chess-knight");
glyph!(CARDS, "cards");
glyph!(JOYSTICK, "joystick");
glyph!(GHOST, "ghost");
glyph!(LAMP, "lamp");
glyph!(SOFA, "sofa");
glyph!(BED, "bed");
glyph!(COOKING_POT, "cooking-pot");
glyph!(BATH, "bath");
glyph!(HARD_HAT, "hard-hat");
glyph!(WRENCH, "wrench");
glyph!(BUILDING, "building");
glyph!(SCALES, "scales");
glyph!(PACKAGE, "package");

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
    Choice {
        key: "bolt",
        name: "Lightning",
        value: "folio:bolt",
        glyph: &BOLT,
    },
    Choice {
        key: "cloud",
        name: "Cloud",
        value: "folio:cloud",
        glyph: &CLOUD,
    },
    Choice {
        key: "umbrella",
        name: "Umbrella",
        value: "folio:umbrella",
        glyph: &UMBRELLA,
    },
    Choice {
        key: "flame",
        name: "Flame",
        value: "folio:flame",
        glyph: &FLAME,
    },
    Choice {
        key: "droplet",
        name: "Water",
        value: "folio:droplet",
        glyph: &DROPLET,
    },
    Choice {
        key: "moon-stars",
        name: "Moon and stars",
        value: "folio:moon-stars",
        glyph: &MOON_STARS,
    },
    Choice {
        key: "headphones",
        name: "Headphones",
        value: "folio:headphones",
        glyph: &HEADPHONES,
    },
    Choice {
        key: "microphone",
        name: "Microphone",
        value: "folio:microphone",
        glyph: &MICROPHONE,
    },
    Choice {
        key: "film",
        name: "Film",
        value: "folio:film",
        glyph: &FILM,
    },
    Choice {
        key: "gamepad",
        name: "Game controller",
        value: "folio:gamepad",
        glyph: &GAMEPAD,
    },
    Choice {
        key: "brush",
        name: "Paintbrush",
        value: "folio:brush",
        glyph: &BRUSH,
    },
    Choice {
        key: "pen-tool",
        name: "Design pen",
        value: "folio:pen-tool",
        glyph: &PEN_TOOL,
    },
    Choice {
        key: "scissors",
        name: "Scissors",
        value: "folio:scissors",
        glyph: &SCISSORS,
    },
    Choice {
        key: "ruler",
        name: "Ruler",
        value: "folio:ruler",
        glyph: &RULER,
    },
    Choice {
        key: "pencil",
        name: "Pencil",
        value: "folio:pencil",
        glyph: &PENCIL,
    },
    Choice {
        key: "map",
        name: "Map",
        value: "folio:map",
        glyph: &MAP,
    },
    Choice {
        key: "plane",
        name: "Travel",
        value: "folio:plane",
        glyph: &PLANE,
    },
    Choice {
        key: "car",
        name: "Car",
        value: "folio:car",
        glyph: &CAR,
    },
    Choice {
        key: "bicycle",
        name: "Cycling",
        value: "folio:bicycle",
        glyph: &BICYCLE,
    },
    Choice {
        key: "ship",
        name: "Sailing",
        value: "folio:ship",
        glyph: &SHIP,
    },
    Choice {
        key: "tree",
        name: "Tree",
        value: "folio:tree",
        glyph: &TREE,
    },
    Choice {
        key: "flower",
        name: "Flower",
        value: "folio:flower",
        glyph: &FLOWER,
    },
    Choice {
        key: "paw",
        name: "Pets",
        value: "folio:paw",
        glyph: &PAW,
    },
    Choice {
        key: "fish",
        name: "Fish",
        value: "folio:fish",
        glyph: &FISH,
    },
    Choice {
        key: "fruit",
        name: "Food",
        value: "folio:fruit",
        glyph: &FRUIT,
    },
    Choice {
        key: "cake",
        name: "Celebration",
        value: "folio:cake",
        glyph: &CAKE,
    },
    Choice {
        key: "gift",
        name: "Gift",
        value: "folio:gift",
        glyph: &GIFT,
    },
    Choice {
        key: "trophy",
        name: "Achievement",
        value: "folio:trophy",
        glyph: &TROPHY,
    },
    Choice {
        key: "shield",
        name: "Protection",
        value: "folio:shield",
        glyph: &SHIELD,
    },
    Choice {
        key: "key",
        name: "Key",
        value: "folio:key",
        glyph: &KEY,
    },
    Choice {
        key: "coins",
        name: "Coins",
        value: "folio:coins",
        glyph: &COINS,
    },
    Choice {
        key: "bank",
        name: "Banking",
        value: "folio:bank",
        glyph: &BANK,
    },
    Choice {
        key: "credit-card",
        name: "Payments",
        value: "folio:credit-card",
        glyph: &CREDIT_CARD,
    },
    Choice {
        key: "receipt",
        name: "Expenses",
        value: "folio:receipt",
        glyph: &RECEIPT,
    },
    Choice {
        key: "calculator",
        name: "Accounting",
        value: "folio:calculator",
        glyph: &CALCULATOR,
    },
    Choice {
        key: "medical-kit",
        name: "Healthcare",
        value: "folio:medical-kit",
        glyph: &MEDICAL_KIT,
    },
    Choice {
        key: "stethoscope",
        name: "Medicine",
        value: "folio:stethoscope",
        glyph: &STETHOSCOPE,
    },
    Choice {
        key: "pill",
        name: "Medication",
        value: "folio:pill",
        glyph: &PILL,
    },
    Choice {
        key: "activity",
        name: "Fitness",
        value: "folio:activity",
        glyph: &ACTIVITY,
    },
    Choice {
        key: "brain",
        name: "Mental wellbeing",
        value: "folio:brain",
        glyph: &BRAIN,
    },
    Choice {
        key: "microscope",
        name: "Research",
        value: "folio:microscope",
        glyph: &MICROSCOPE,
    },
    Choice {
        key: "atom",
        name: "Physics",
        value: "folio:atom",
        glyph: &ATOM,
    },
    Choice {
        key: "dna",
        name: "Biology",
        value: "folio:dna",
        glyph: &DNA,
    },
    Choice {
        key: "telescope",
        name: "Astronomy",
        value: "folio:telescope",
        glyph: &TELESCOPE,
    },
    Choice {
        key: "magnet",
        name: "Engineering",
        value: "folio:magnet",
        glyph: &MAGNET,
    },
    Choice {
        key: "theatre",
        name: "Performing arts",
        value: "folio:theatre",
        glyph: &THEATRE,
    },
    Choice {
        key: "picture",
        name: "Visual arts",
        value: "folio:picture",
        glyph: &PICTURE,
    },
    Choice {
        key: "easel",
        name: "Painting",
        value: "folio:easel",
        glyph: &EASEL,
    },
    Choice {
        key: "sculpture",
        name: "Sculpture",
        value: "folio:sculpture",
        glyph: &SCULPTURE,
    },
    Choice {
        key: "swatches",
        name: "Color design",
        value: "folio:swatches",
        glyph: &SWATCHES,
    },
    Choice {
        key: "guitar",
        name: "Guitar",
        value: "folio:guitar",
        glyph: &GUITAR,
    },
    Choice {
        key: "piano",
        name: "Piano",
        value: "folio:piano",
        glyph: &PIANO,
    },
    Choice {
        key: "drum",
        name: "Percussion",
        value: "folio:drum",
        glyph: &DRUM,
    },
    Choice {
        key: "speaker",
        name: "Audio",
        value: "folio:speaker",
        glyph: &SPEAKER,
    },
    Choice {
        key: "waveform",
        name: "Sound production",
        value: "folio:waveform",
        glyph: &WAVEFORM,
    },
    Choice {
        key: "dice",
        name: "Board games",
        value: "folio:dice",
        glyph: &DICE,
    },
    Choice {
        key: "chess-knight",
        name: "Chess",
        value: "folio:chess-knight",
        glyph: &CHESS_KNIGHT,
    },
    Choice {
        key: "cards",
        name: "Card games",
        value: "folio:cards",
        glyph: &CARDS,
    },
    Choice {
        key: "joystick",
        name: "Arcade",
        value: "folio:joystick",
        glyph: &JOYSTICK,
    },
    Choice {
        key: "ghost",
        name: "Video games",
        value: "folio:ghost",
        glyph: &GHOST,
    },
    Choice {
        key: "lamp",
        name: "Lighting",
        value: "folio:lamp",
        glyph: &LAMP,
    },
    Choice {
        key: "sofa",
        name: "Living room",
        value: "folio:sofa",
        glyph: &SOFA,
    },
    Choice {
        key: "bed",
        name: "Bedroom",
        value: "folio:bed",
        glyph: &BED,
    },
    Choice {
        key: "cooking-pot",
        name: "Cooking",
        value: "folio:cooking-pot",
        glyph: &COOKING_POT,
    },
    Choice {
        key: "bath",
        name: "Bathroom",
        value: "folio:bath",
        glyph: &BATH,
    },
    Choice {
        key: "hard-hat",
        name: "Construction",
        value: "folio:hard-hat",
        glyph: &HARD_HAT,
    },
    Choice {
        key: "wrench",
        name: "Maintenance",
        value: "folio:wrench",
        glyph: &WRENCH,
    },
    Choice {
        key: "building",
        name: "Real estate",
        value: "folio:building",
        glyph: &BUILDING,
    },
    Choice {
        key: "scales",
        name: "Legal",
        value: "folio:scales",
        glyph: &SCALES,
    },
    Choice {
        key: "package",
        name: "Logistics",
        value: "folio:package",
        glyph: &PACKAGE,
    },
];

pub fn choice(value: &str) -> Option<&'static Choice> {
    CHOICES.iter().find(|icon| {
        icon.value == value
            || value.strip_prefix("folio:") == Some(icon.key)
            || (icon.key == "database" && value == "🗂")
    })
}
