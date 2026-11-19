use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IconColor {
    Sage,
    Teal,
    Blue,
    Violet,
    Rose,
    Coral,
    Amber,
    Gray,
    #[default]
    #[serde(other)]
    Default,
}

impl IconColor {
    pub const ALL: [Self; 9] = [
        Self::Default,
        Self::Sage,
        Self::Teal,
        Self::Blue,
        Self::Violet,
        Self::Rose,
        Self::Coral,
        Self::Amber,
        Self::Gray,
    ];

    pub fn is_default(&self) -> bool {
        *self == Self::Default
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sage => "sage",
            Self::Teal => "teal",
            Self::Blue => "blue",
            Self::Violet => "violet",
            Self::Rose => "rose",
            Self::Coral => "coral",
            Self::Amber => "amber",
            Self::Gray => "gray",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Sage => "Sage",
            Self::Teal => "Teal",
            Self::Blue => "Blue",
            Self::Violet => "Violet",
            Self::Rose => "Rose",
            Self::Coral => "Coral",
            Self::Amber => "Amber",
            Self::Gray => "Gray",
        }
    }

    /// Dark and light variants keep the same hue with enough contrast for line icons.
    pub fn rgb(self, dark: bool) -> Option<[u8; 3]> {
        let (light, dark_color) = match self {
            Self::Default => return None,
            Self::Sage => ([65, 101, 62], [153, 189, 146]),
            Self::Teal => ([45, 119, 115], [126, 192, 183]),
            Self::Blue => ([57, 105, 160], [139, 176, 218]),
            Self::Violet => ([112, 86, 160], [182, 162, 218]),
            Self::Rose => ([159, 75, 114], [215, 157, 184]),
            Self::Coral => ([171, 80, 60], [225, 165, 139]),
            Self::Amber => ([151, 106, 34], [217, 188, 121]),
            Self::Gray => ([99, 112, 120], [174, 184, 188]),
        };
        Some(if dark { dark_color } else { light })
    }
}
