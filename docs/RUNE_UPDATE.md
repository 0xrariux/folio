# Mise à jour Rune — 15 novembre 2026

Rune a été mis à jour par `git pull --ff-only`, de
`70eff81165e9854834a75f17753edb6c96ab2ab7` à
`7a92d24d63d9be8480e3e475489d2a4d0ed48c8d` : 627 commits, fusions comprises.
Son `CHANGELOG.md` n'a pas changé sur cet intervalle ; les nouveautés sont dans
343 fragments ajoutés et un fragment modifié sous `changelog/unreleased/`.

## Adaptations appliquées

| Sujet | Impact constaté | Adaptation |
| --- | --- | --- |
| Dépendances corrigées par Rune | Le premier `cargo check --locked --offline` échouait : le `parley_engine` publié ne fournit pas `line-dictionaries`. Les patches d'une dépendance ne se propagent pas à l'application. | Patches applicatifs vers `rune/vendor` pour `parley_engine`, `winit`, `wgpu-hal`, `accesskit_windows` et `accesskit_macos`. |
| Reproductibilité | Les guides installaient encore l'ancien Rune. | Révision README/guide actualisée et `Cargo.lock` recalculé, sans mise à jour générale des bibliothèques publiées. Les chemins des patches suivent le checkout voisin. |
| Fermeture | Le nouvel arrêt rapide de Rune saute `Worker::drop`, donc l'attente des écritures FIFO et la sauvegarde automatique de fin de session. | `rune::orderly_exit()` avant le lancement pour conserver ces garanties. |
| Captures release | `--snap`, `--script`, `--tree`, etc. nécessitent désormais `rune/tooling` en release. | Feature applicative `tooling`, activable par `--features tooling`. Les captures debug restent disponibles. |

Le `tiny-skia-path` corrigé est déjà une dépendance directe de Rune : aucun patch
supplémentaire pour celui-ci. L'arrêt ordonné conserve le coût du démontage ;
un arrêt rapide demanderait un protocole explicite de fermeture du stockage avant
la sortie du processus.

Les ruptures annoncées sur `TextEdit::preedit`, les états de combobox et les
options de zoom des composants ne nécessitent pas d'adaptation dans les usages
actuels de Folio. La compilation de toutes les cibles du projet sur macOS réussit.

## Effets reçus avec Rune

- Texte mesuré au 1/64 de pixel, corrections d'origine et d'icônes, couleurs Metal
  et respect du lissage macOS : les titres, cellules, menus et pictogrammes peuvent
  se placer ou se dessiner légèrement différemment. Les captures historiques
  restent la trace de l'ancien rendu.
- Annulation des champs plus proche de Chromium, améliorations de la sélection,
  du texte bidirectionnel, du focus après disparition d'un contrôle et du
  défilement imbriqué. Revoir l'édition, les listes repliables et les menus.
- Les patches AccessKit apportent les titres natifs aux lecteurs d'écran et
  corrigent un risque d'abandon du processus sur certaines requêtes macOS.
  Les repères de navigation et les titres nommés sont maintenant fournis par Folio.
- Sous Windows : premier affichage CPU avant relais GPU et améliorations de
  lancement et de rendu. Aucun gain chiffré propre à Folio n'est revendiqué ici.

## Suites appliquées dans Folio

1. **Grandes bases : lignes de tableau virtualisées.** `src/ui/table.rs` utilise
   `virtual_rows` avec des clés `table-row-{id}`, une hauteur de cellule de 44 px
   et un séparateur de 1 px entre les lignes. L’en-tête reste fixe et les colonnes
   conservent leur largeur. Le filtrage, le tri et la résolution des positions sont
   effectués une fois par construction de vue, puis seules les cellules des lignes
   montées sont construites. Les actions capturent les identifiants des fiches.
   La sélection globale porte sur toutes les fiches filtrées, même non montées.
   Les tests couvrent le saut vers la dernière ligne non montée, le focus clavier
   hors écran, le tri et la sélection après filtrage sur 1 000 fiches.
2. **Recherche Unicode commune.** La recherche de pages, la palette (commandes,
   mentions, blocs, bases, vues et relations), le sélecteur de liens et les filtres
   de fiches/dashboard utilisent `rune_kit::search::Fold::for_language("fr")`.
   `reunion` trouve `Réunion` ; une requête accentuée `réunion` écarte `Reunion`.
   La casse Unicode et les variantes de largeur sont aussi prises en charge.
   Les résultats de la palette surlignent les plages UTF-8 du titre original.
   Les règles d’unicité des noms et le tri des titres restent distincts de la recherche.
3. **Structure accessible.** La barre latérale est un repère `navigation` et le
   contenu un repère `main`, tous deux nommés. Le titre de page est de niveau 1,
   les titres de blocs de niveaux 2 à 4, y compris les titres repliables ; leurs
   champs restent éditables. Les fenêtres de dialogue ont aussi un titre nommé.
   F6/Maj+F6 et l’édition après navigation sont testés dans le runtime Rune.
   `Tx::announce` annonce la fin des écritures, les messages de sauvegarde/export
   et les erreurs asynchrones. Les erreurs sont prioritaires et les accusés de
   réception d’anciennes révisions ne déclenchent pas de fausse fin de sauvegarde.
4. **Mesures reproductibles avant cache.** Le test `measure_large_views` mesure
   le premier affichage et une mise à jour de thème, sans introduire `memo`.
   Il utilise des données isolées, une fenêtre de 1 200 × 820 et une arborescence
   repliée. Exécution séparée, en profil debug, sur cette machine :

   | Vue | Premier affichage | Mise à jour du thème | Lignes de tableau montées |
   | --- | ---: | ---: | ---: |
   | 1 000 fiches | 244 ms | 13 ms | 19 |
   | 10 000 fiches | 405 ms | 195 ms | 19 |
   | 500 blocs éditables | 211 ms | 44 ms | — |

   Ces temps englobent le runtime et toute la vue, dont la barre latérale, les
   recherches et les tris ; ils ne mesurent pas uniquement les cellules ni le GPU.
   Ils varient selon le profil et les caches de polices ; aucun gain comparatif
   avant/après n’est revendiqué. Le parcours des ancêtres dans la barre latérale
   et les blocs éditables restent des pistes de travail. Leur virtualisation
   demande de traiter explicitement les cibles hors écran de sélection et de drag.

   ```sh
   cargo test --bin folio --locked --offline measure_large_views -- --nocapture --test-threads=1
   ```

5. **Rendu macOS inspecté.** Captures Metal du tableau personnalisé sombre, du
   tableau clair après redimensionnement de fenêtre et de colonne, de la palette
   Unicode et d’un déplacement de blocs. Les scripts `rune-table-resize.json` et
   `rune-unicode-search.json` rendent les deux nouveaux scénarios reproductibles.
   Les tests existants couvrent aussi l’édition, la sélection, le repli et les
   déplacements groupés avec défilement automatique. VoiceOver reste à qualifier
   sur un lecteur d’écran réel.

`pretty_wrap` est optionnel et n'impose aucun changement des paragraphes. Le zoom
global `Cx::zoom` est disponible pour un futur réglage d'accessibilité ; il ne faut
pas le confondre avec un changement de taille de fenêtre ou de largeur de colonne.

## Validation

- `cargo check --all-targets --offline` : réussi après actualisation du verrouillage.
- `cargo fmt --package folio -- --check` : réussi.
- `cargo clippy --all-targets --locked --offline -- -D warnings` : réussi.
- `cargo test --locked --offline` : **101 réussis**, aucun échec ou test ignoré.
- `cargo check --release --locked --offline --features tooling` : réussi.
- Quatre captures Metal produites et inspectées sous `artifacts/folio-rune-*` :
  tableau sombre, tableau clair redimensionné, palette Unicode et scène `drag-list`.
  L'accès Metal est indisponible dans le bac à sable ; le même
  binaire a réussi hors de cette restriction, avec `FOLIO_DEMO=1` et sans accès
  aux notes réelles.
- Rune et ses sources embarquées émettent encore des avertissements ; aucun
  changement local dans Rune pour les masquer.

Le format des documents et les migrations Turso ne changent pas. La fermeture
native sur des notes réelles, VoiceOver et l'exécution Windows/Linux restent à
qualifier ; les tests de stockage et d'interface utilisent leurs données isolées.

## Sources Rune

- [Dépendances embarquées et patches](../../rune/Cargo.toml).
- [Arrêt rapide par défaut](../../rune/changelog/unreleased/2026-11-13-fast-exit-default.md).
- [Tooling en release](../../rune/changelog/unreleased/2026-11-13-tooling-feature.md).
- [Recherche Unicode](../../rune/changelog/unreleased/2026-11-12-search-fold.md).
- [Focus et navigation F6](../../rune/changelog/unreleased/2026-11-12-a11y-focus-fallback-f6.md).
- [Titres macOS](../../rune/changelog/unreleased/2026-11-15-macos-headings.md).
- [Lignes virtuelles non montées](../../rune/changelog/unreleased/2026-11-15-scroll-key-virtual.md).
