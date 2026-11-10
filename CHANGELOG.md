# Changelog

## 0.10.0

- Tableaux intégrés étirés sur toute la largeur disponible, y compris après
  redimensionnement ; cadre et en-tête communs aux tableaux et dashboards,
  avec bordure unique et séparateurs uniquement entre les fiches.
- Nouveau logo Folio créé dans Blender : F plein aux terminaisons de feuilles
  pliées, badge mieux proportionné, source `.blend`, rendus PNG et exports SVG.
  Le monogramme vectoriel est intégré à la barre latérale et au catalogue.
- Actions secondaires regroupées dans le menu « Plus » en bas de la barre latérale,
  avec nouvelle page/base, modèles et carnet ; les accès courants restent visibles.
- Alignement à gauche des noms dans « Avancement par base » et des commandes de
  navigation ; grille commune aux propriétés intégrées et personnalisées, avec
  un même départ pour les valeurs éditables, relations et résultats calculés.

- Historique durable (50 versions/page), points de restauration et restauration
  d’une version en conservant l’état actuel.
- Sauvegarde/restauration complète `.folio` : pages, vues, fichiers, historique
  et préférences ; aperçu avant remplacement, écriture SQL transactionnelle et
  sauvegarde de l’état précédent. Les 14 dernières sauvegardes automatiques sont conservées.
- Sélection multiple de blocs, copie/collage, duplication, transformation,
  suppression et déplacement groupés avec leurs descendants et une seule annulation.
- Déplacement à la souris des cartes Kanban, pages de l’arborescence et dates
  du calendrier ; ordre persistant, prévention des cycles, Échap et défilement automatique.
- Vues de bases intégrées aux notes et fiches, choix d’une vue existante et filtre
  de relation au projet ; création et modification des fiches sources en direct.
- Relations inter-bases avec colonne inverse automatique, agrégations nombre,
  terminées, progression, somme ou nombre de fiches restantes et formules numériques.
- Calendrier mensuel, détail d’un jour, création à une date et sélecteurs natifs Rune.
- Images et documents copiés dans un dossier géré, redimensionnement des images,
  ouverture système, récupération de l’original et médias joints à l’export Markdown.
- Palette globale ⌘K, mentions `@`, liens de blocs, sommaire dynamique et backlinks.
- Largeur, ordre et visibilité des colonnes par vue ; statut, priorité et échéance
  modifiables sur une sélection de fiches.
- Préférences persistantes : thème sombre initial, thème choisi, largeur de la
  barre latérale et dernière page ouverte.
- Format de document 6, avec lecture des formats 1 à 5. Démonstrations `workspace`
  et `calendar`, tests des parcours souris/clavier, fichiers et restaurations.

## 0.9.0

- Listes à tiroirs et titres H1/H2/H3 repliables, imbriqués jusqu’à six niveaux.
  L’état ouvert/fermé est enregistré, annulable et accessible au clavier.
- Organisation des sous-éléments par Tab/Maj+Tab, boutons et menu de bloc ;
  ajout de contenu dans les tiroirs et commandes pour tout replier ou déplier.
- Déplacement à la souris ou au clavier, duplication et suppression d’un groupe
  avec ses enfants, y compris masqués, en une étape d’annulation.
- Numérotation par niveau conservée au retour d’une sous-liste, avec reprise ou
  redémarrage ; puces distinctes selon la profondeur.
- Raccourcis de saisie Markdown, collage de plusieurs éléments avec styles et
  hiérarchie, sortie progressive des listes vides et désindentation par Retour arrière.
- Progression des tâches dans les tiroirs et commandes pour cocher/décocher
  toute la liste ; nouveau modèle « Carnet à tiroirs » et scène `lists`.
- Export Markdown avec numérotation réelle, six niveaux et arbres repliables
  en HTML `<details>` ; contenu masqué toujours recherché et exporté.
- Format de document 5 avec lecture des versions 1 à 4 ; les anciens clients
  refusent ce nouveau format pour protéger les états et points de redémarrage.
- Thème sombre par défaut, avec bascule vers le clair conservée.

## 0.8.0

- Extension de la famille Folio à 59 SVG : 43 nouveaux signes pour les commandes,
  la navigation, les blocs, les propriétés et les vues de bases.
- Intégration dans la barre latérale, les menus, les tableaux, le Kanban,
  les dashboards et l’éditeur, notamment ses poignées de glisser-déposer.
- Boutons sans texte dotés de libellés accessibles et d’infobulles ; sélections,
  favoris et cases à cocher utilisent des signes vectoriels cohérents.
- Deux planches autonomes claire/sombre et scènes natives `icon-catalog` et
  `icon-controls` pour contrôler les dessins à trois tailles.
- Les notes et identifiants existants sont conservés ; le format reste en version 4.

## 0.7.0

- Famille SVG dessinée pour Folio : 15 pictogrammes de pages et un monogramme,
  avec un trait commun et une couleur adaptée aux thèmes clair et sombre.
- Remplacement des emojis du catalogue dans la navigation, les titres de pages,
  les fiches, les liens, les sélecteurs et les encadrés.
- Sélecteur en grille avec noms accessibles, infobulles et indication du choix
  actif ; les anciens identifiants sont conservés sans migration des notes.
- Sources SVG modifiables, planche autonome claire/sombre et scène native
  `icon-catalog` affichant chaque signe à trois tailles.

## 0.6.1

- Séparation des onglets placée sous leurs boutons avec un espacement dédié,
  sans recouvrement par les fonds sélectionnés ou les états de survol.
- Tableau : contour indépendant du défilement, bordure réservée dans la mise
  en page et contenu découpé selon les coins arrondis du cadre.
- Suppression de la double ligne inférieure ; séparateurs uniquement entre les
  lignes, en-tête plus lisible et cellules alignées avec leurs titres. L’en-tête
  reste visible pendant le défilement vertical.
- Scène de contrôle `launch-table` et captures de comparaison en clair et sombre.

## 0.6.0

- Glisser-déposer des blocs à la souris depuis leur poignée `⠿`, dans les deux
  directions et en fin de liste, avec aperçu flottant et repère d’insertion.
- Défilement automatique en bord de document ; Échap, perte de focus et dépôt
  hors éditeur annulent le geste. Les clics et le menu clavier restent disponibles.
- Une seule étape d’annulation et une sauvegarde au dépôt, sans réécrire les
  données pendant le survol ; texte enrichi, identifiants, indentation et cases
  cochées conservés. La numérotation suit l’ordre des blocs.
- Redessin des numéros après réorganisation et conservation du bloc actif lors
  de la synchronisation du focus des champs par Rune.
- Scène `drag-list`, scripts de démonstration et tests souris, annulation,
  défilement, contenu enrichi, historique et persistance Turso.
- Rune mis à jour par `git pull --ff-only` de `8a6f2f42` à `70eff811`
  (56 commits). Le format de document reste en version 4.

### Rune : changements récupérés dans la nuit du 9 au 10 novembre 2026

- macOS recharge les shaders Metal précompilés ; corrections de stabilisation
  des animations, glissements et tailles de lignes contenant du code inline.
- Corrections d’annulation dans les champs vides, de navigation Tab, de menus
  macOS et de libération de Secure Event Input au changement de fenêtre.
- Nouvelles lectures `Tx::focused` dans les gestionnaires et `Tx::by_keys` pour
  distinguer une activation clavier d’un clic, utilisée par les poignées Folio.
- Modes de rendu de texte supplémentaires, segmentation des mots CJK, direction
  par paragraphe, surbrillances de texte persistantes et infobulles multi-lignes.
- Améliorations des diagrammes Mermaid et de la pile de notifications ToastStack.

Les événements de glisser-déposer existaient déjà avant ce pull ; Folio utilise
`El::on_drag`, `Drag::cancelled`, les requêtes de cible et la répétition du geste.

## 0.5.0

- Vues nommées par base, avec disposition, filtres combinés et tri indépendants ;
  changements de la vue active enregistrés automatiquement et sélection persistante.
- Création, renommage et retrait des vues sans supprimer les fiches ; validation
  des noms, limite de 20 vues et conservation d’une dernière vue.
- Export Markdown des notes et fiches, avec blocs, styles, code et propriétés.
- Export CSV de la vue courante ou de toutes les fiches du périmètre, avec
  colonnes masquées et précision numérique conservées.
- Boîte native de destination, annulation, écriture atomique sur un thread dédié
  et confirmation ou erreur visible ; protection du fichier de base en cours.
- Lecture des formats de document 1 à 3, écriture du format 4 et tests de
  persistance des vues, de migration et des parcours d’export dans l’interface.
- Scènes de démonstration des vues enregistrées et du suivi des retards.

## 0.4.0

- Colonnes personnalisées texte, nombre, date et case à cocher : création,
  renommage, affichage/masquage et édition des cellules avec validation.
- Filtres combinables par statut, priorité et échéance, communs aux vues de base.
- Vue Dashboard par base et dashboards globaux avec sélection de plusieurs bases.
- Indicateurs dynamiques de progression, statuts et échéances ; graphiques
  cliquables, totaux numériques et décomptes de cases cochées.
- Avancement par base, échéances à surveiller et accès aux fiches détaillées.
- Modèle de dashboard, données de démonstration et scènes de prévisualisation.
- Lecture des anciens documents et enregistrement du format version 3, avec
  conservation des identifiants et des valeurs masquées dans Turso.

## 0.3.0

- Bases de fiches plein écran : chaque élément est une page de notes avec des
  propriétés intégrées (statut, priorité, échéance et étiquettes).
- Vues tableau et Kanban sur les mêmes fiches, avec création par statut,
  recherche indépendante, filtre de statut et quatre ordres de tri enregistrés.
- Formulaire d’échéance et d’étiquettes avec validation et annulation.
- Liens de pages via `/link` ou `/lien`, recherche de cible, navigation et liens
  retour automatiques ; les références suivent les renommages.
- Modèle « Suivi de projet » et exemple de base dans le mode démo.
- Lecture des documents de version 1 et enregistrement en version 2, avec valeurs
  par défaut pour les nouveaux champs et sauvegardes transactionnelles Turso.
- Comparatif fonctionnel documenté dans `NOTION-PARITY.md`.

## 0.2.0

- Éditeur composé de blocs avec identifiants stables : texte, trois niveaux de
  titres, listes à puces et numérotées, tâches à cocher, citations, encadrés, code
  et séparateurs.
- Commandes `/` filtrables en français et en anglais, sélection au clavier et
  transformation du type d’un bloc sans perdre son contenu.
- Ajout, duplication, suppression et déplacement des blocs depuis leur menu ;
  annulation et rétablissement au niveau du document pendant la session.
- Modèles de notes de réunion, de projets et de listes de tâches.
- Icônes de pages, trois couvertures colorées et affichage en pleine largeur.
- Duplication d’une page, déplacement dans l’arborescence avec protection contre
  les cycles, navigation précédent/suivant et liens vers les sous-pages.
- Conversion automatique des documents de la version initiale, avec conservation
  du texte enrichi et de l’indentation des listes. Écritures atomiques dans Turso.

## 0.1.0

- Premier carnet natif Rust avec Rune : pages et sous-pages, recherche, favoris,
  archives, texte enrichi, thèmes clair/sombre et stockage Turso local.
