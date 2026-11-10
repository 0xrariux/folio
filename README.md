# Folio

Notes, tâches et suivi de projets dans une application native inspirée de Notion.
Écrite en **Rust**, avec **Rune** et **Turso**. Données locales, sans compte.

## Fonctionnalités

- Pages, blocs, listes repliables, tâches et glisser-déposer.
- Bases en tableau, Kanban, calendrier et dashboards.
- Relations, agrégations, formules et vues intégrées aux notes.
- Recherche, liens internes, mentions et fichiers joints.
- Historique, sauvegarde/restauration et exports Markdown/CSV.

Thème sombre par défaut. Synchronisation et collaboration non disponibles.

## Installation

Prérequis : Rust installé avec `rustup`.

```sh
git clone https://github.com/anomalyco/rune.git
git -C rune checkout 70eff81165e9854834a75f17753edb6c96ab2ab7
git clone https://github.com/0xrariux/folio.git
cd folio
cargo run --locked
```

Garder les dossiers `rune` et `folio` côte à côte. La version de Rust est fixée
par `rust-toolchain.toml`.

## Démonstration

Pour explorer un projet sans modifier ses notes :

```sh
FOLIO_DEMO=1 cargo run --locked -- --scene workspace
```

## Documentation

[Guide d’utilisation et développement](docs/GUIDE.md) ·
[Changelog](CHANGELOG.md) · [État des fonctionnalités](NOTION-PARITY.md) ·
[Logo et sources Blender](assets/brand/README.md)

Licence [MIT](LICENSE).
