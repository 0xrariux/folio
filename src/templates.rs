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
            Self::Meeting => "Note de réunion",
            Self::Project => "Nouveau projet",
            Self::Tasks => "Ma liste de tâches",
            Self::Tracker => "Suivi de projet",
            Self::Dashboard => "Dashboard de suivi",
            Self::Outline => "Carnet à tiroirs",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Meeting => "Un ordre du jour et des décisions claires",
            Self::Project => "Du contexte aux prochaines étapes",
            Self::Tasks => "De petites cases pour de grandes avancées",
            Self::Tracker => "Une base de fiches, en tableau ou en Kanban",
            Self::Dashboard => "Les indicateurs de toutes vos bases, au même endroit",
            Self::Outline => "Des idées, des étapes et des tâches que l’on peut replier",
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
                        "Dépliez les chevrons pour explorer. Entrée dans un titre repliable ajoute son contenu ; Tab et Maj+Tab organisent les sous-éléments.",
                    ),
                    block(Kind::ToggleHeading2, "Un projet, étape par étape"),
                    block(
                        Kind::Text,
                        "Un espace pour le contexte et les idées importantes.",
                    ),
                    block(Kind::Numbered, "Préparer le terrain"),
                    block(Kind::Bullet, "Rassembler les idées et les ressources"),
                    block(Kind::Bullet, "Définir ce qui compte vraiment"),
                    block(Kind::Numbered, "Construire et partager"),
                    block(Kind::Toggle, "La liste de lancement"),
                    block(Kind::Todo, "Poser les fondations"),
                    block(Kind::Todo, "Peaufiner la première version"),
                    block(Kind::Todo, "Recueillir les premiers retours"),
                    block(Kind::ToggleHeading2, "Les idées pour plus tard"),
                    block(Kind::Quote, "Une petite idée peut devenir un grand projet."),
                    block(Kind::Bullet, "Une piste à explorer demain"),
                    block(
                        Kind::Text,
                        "Les notes continuent ici, en dehors des tiroirs.",
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
                        "Objectif : qu’aimerions-nous avoir décidé à la fin de cette réunion ?",
                    ),
                    block(Kind::Heading2, "Ordre du jour"),
                    block(Kind::Bullet, "Les sujets à aborder"),
                    block(Kind::Heading2, "Notes & décisions"),
                    block(
                        Kind::Text,
                        "Les idées importantes, au fil de la discussion.",
                    ),
                    block(Kind::Heading2, "Prochaines actions"),
                    block(Kind::Todo, "Une action, une personne, une échéance"),
                ]
            }
            Self::Project => {
                page.emoji = "🚀".into();
                page.cover = 1;
                vec![
                    block(
                        Kind::Callout,
                        "Une ambition claire, des petits pas concrets.",
                    ),
                    block(Kind::Heading2, "Pourquoi ce projet ?"),
                    block(
                        Kind::Text,
                        "Le problème à résoudre et les personnes que nous voulons aider.",
                    ),
                    block(Kind::Heading2, "Les prochaines étapes"),
                    block(Kind::Todo, "Définir le périmètre"),
                    block(Kind::Todo, "Construire une première version"),
                    block(Kind::Todo, "Recueillir les premiers retours"),
                    block(Kind::Divider, ""),
                    block(Kind::Heading3, "Ressources & idées"),
                    block(Kind::Text, "Les liens et les notes utiles au projet."),
                ]
            }
            Self::Tasks => {
                page.emoji = "✅".into();
                page.cover = 3;
                vec![
                    block(Kind::Heading2, "Aujourd’hui"),
                    block(Kind::Todo, "La chose la plus importante"),
                    block(Kind::Todo, "Un petit pas qui fait avancer"),
                    block(Kind::Heading2, "Plus tard"),
                    block(Kind::Todo, "Une idée pour demain"),
                ]
            }
        };
        page.sync_document();
        page
    }
}
