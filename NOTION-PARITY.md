# Folio et Notion — état des lieux

Comparaison fonctionnelle au 10 novembre 2026. « Partiel » décrit un périmètre
réduit et utilisable, sans prétendre reproduire toute la fonction de Notion.
Le tableau ci-dessous conserve le bilan de l’itération 0.3, partie de Folio 0.2.
La version actuelle est 0.10 ; les changements récents sont détaillés ci-dessous.

## État courant — Folio 0.10

| Domaine | Disponible dans Folio | Périmètre restant |
| --- | --- | --- |
| Éditeur | 20 types de blocs, tiroirs, listes imbriquées, sélection multiple et opérations groupées | Colonnes de blocs et réglages typographiques avancés |
| Organisation | Pages/sous-pages, favoris, archives, ordre et parenté à la souris | Permissions et espaces partagés |
| Bases | Tableau, Kanban, Calendrier et Dashboard ; vues nommées, filtres et tris | Galerie, timeline et filtres imbriqués complexes |
| Tableaux | Largeur, ordre, visibilité par vue ; modifications de fiches en lot | Édition de cellules au clavier comparable à un tableur |
| Relations et calculs | Relations avec inverse automatique, agrégations et formules numériques | Langage de formules complet et automatisations |
| Pages de projet | Vues de bases intégrées, données partagées, filtre sur la relation au projet | Disposition libre des widgets |
| Navigation | Palette globale ⌘K, mentions @, liens de blocs, sommaire et backlinks | Recherche indexée et liens externes publiés |
| Médias | Images redimensionnables et documents locaux copiés dans le carnet | Médias distants, lecteur PDF et stockage partagé |
| Historique et portabilité | 50 versions/page, checkpoints, sauvegarde/restauration `.folio` avec médias et préférences ; Markdown/CSV | Import Notion/Markdown/CSV et synchronisation |
| Préférences | Sombre par défaut, thème choisi, largeur latérale et dernière page persistants | Préférences synchronisées entre machines |

Le calendrier permet de déplacer les échéances à la souris. Les cartes Kanban et
les pages de l’arborescence se déplacent aussi par glisser-déposer, avec annulation
et prévention des cycles. Les vues intégrées partagent les fiches originales ;
une tâche ajoutée dans le projet entre automatiquement dans sa relation et ses
calculs lorsque le filtre de relation est activé.

Les formules sont numériques (`prop("Charge") * 80`, opérations et parenthèses).
Les agrégations proposent nombre, terminées, progression, somme et nombre de fiches restantes.
Le format de document est 6, avec lecture des formats 1 à 5.

Les sections suivantes conservent l’historique des itérations ; leurs limites
s’appliquent à la version décrite, pas à la version courante.

## Évolution Folio 0.9

- Listes à tiroirs et titres repliables H1/H2/H3 avec état enregistré ; les blocs
  imbriqués ont une hiérarchie commune, limitée à six niveaux de retrait.
- Les déplacements, duplications et suppressions emportent les sous-éléments.
  Le dépôt prend le niveau de sa cible en conservant les retraits relatifs.
- Numérotation par niveau, reprise/redémarrage, raccourcis Markdown, collage
  enrichi de listes, commandes de tâches groupées et modèle de carnet à tiroirs.
- L’export conserve les tiroirs via HTML `<details>` dans le fichier Markdown.
  Le format 5 protège ces données contre les anciens clients.
- Le thème sombre devient le choix initial. Le clair reste disponible.

Le geste déplace un groupe à la fois ; la sélection de plusieurs blocs distincts,
le choix libre de styles de puces et les colonnes de blocs restent à développer.

## Évolution Folio 0.8

- Extension de l’identité visuelle aux commandes et types de blocs : 43 signes
  supplémentaires, soit 59 SVG communs aux thèmes clair et sombre.
- Navigation, menus, vues, propriétés et poignées utilisent cette famille,
  avec des libellés accessibles pour les actions sans texte.
- Cette itération consolide l’interface ; le périmètre fonctionnel reste inchangé.

## Évolution Folio 0.7

- Identité visuelle commune : pictogrammes SVG pour les pages et bases, sélecteur
  de 15 icônes et monogramme Folio. Les identifiants précédents restent lisibles.
- Cette itération concerne l’apparence. Les relations, calendriers et autres
  priorités fonctionnelles ci-dessous restent à développer.

## Évolution Folio 0.6

- Les blocs des pages et les éléments de listes se réorganisent à la souris
  depuis leur poignée, avec aperçu, repère avant/après et défilement automatique.
- Le dépôt conserve le contenu enrichi et constitue une seule étape d’annulation.
  Échap, perte de focus et dépôt hors éditeur annulent sans réordonner les données.
- Les cartes Kanban et les pages de l’arborescence se déplacent encore par leurs
  commandes existantes ; le geste actuel déplace un seul bloc et garde son indentation.

## Évolution Folio 0.5

- Les bases ont désormais des vues nommées avec leurs propres filtres, tris et
  dispositions Tableau/Kanban/Dashboard. La sélection et les réglages sont
  persistants ; création, renommage et retrait sont disponibles dans l’interface.
- Les notes et fiches s’exportent en Markdown. Les bases et dashboards s’exportent
  en CSV, au choix selon la vue courante ou pour toutes les fiches du périmètre.
- L’export utilise une boîte native, une écriture atomique en arrière-plan et
  une confirmation ou erreur visible. Les tests couvrent aussi son annulation.
- Les formats antérieurs restent lisibles ; le format 4 protège les nouvelles
  vues contre une réécriture par les anciens clients.

Ces fonctions restent limitées : recherche temporaire, colonnes communes à la
base, export Markdown d’une page seule avec liens internes Folio, absence
d’import et de sauvegarde complète du carnet. Les relations, formules, vues
calendrier et disposition libre des widgets restent à construire.

## Évolution Folio 0.4

- Le schéma de base devient extensible : jusqu’à 12 colonnes personnalisées
  texte, nombre, date ou case à cocher, avec renommage et visibilité configurables.
- Les filtres de statut, priorité et échéance se combinent. Ils restent communs
  aux trois vues Tableau, Kanban et Dashboard d’une base.
- Des dashboards globaux regroupent plusieurs bases avec un périmètre et des
  filtres propres, indépendants des filtres des bases sources.
- Les widgets affichent des indicateurs, graphiques, sommes et décomptes issus
  des fiches. Ils réagissent aux changements de données et de filtres.
- Les anciennes données restent lisibles ; les nouvelles sauvegardes utilisent
  le format de document 3 pour protéger les champs inconnus des anciens clients.

La parité reste partielle : les types de colonnes sont limités à quatre, leur
type est fixé à la création, les widgets ont une disposition prédéfinie et les
relations, formules et sources externes restaient absentes à ce stade ; les vues
nommées ont été ajoutées en 0.5.

## Bilan de l’itération 0.3

| Domaine | Notion | Folio 0.2 au départ | Folio 0.3 après cette itération |
| --- | --- | --- | --- |
| Pages et organisation | Pages et sous-pages organisables | Pages, sous-pages, favoris, recherche, archives, déplacements | Conservé ; les fiches de bases restent navigables dans l’arborescence |
| Édition par blocs | Éditeur riche avec nombreux types de blocs | 11 types, commandes `/`, styles, déplacement par menu, annulation en session | 12 types avec le lien de page ; glisser-déposer, médias, colonnes et toggles encore absents |
| Modèles et apparence | Modèles et personnalisation de pages | 3 modèles, icônes, couvertures colorées, pleine largeur et thèmes | 4 modèles, dont un suivi de projet en Kanban |
| Bases de données | Collections dont chaque élément est une page éditable | Absent | Ajouté, partiel : bases plein écran, fiches contenant leurs blocs et propriétés |
| Propriétés | Types configurables, dont statut, date, sélections, relations et formules | Absent | Statut, priorité, échéance et étiquettes intégrés ; schéma fixe, sans ajout de colonnes personnalisées |
| Tableau et Kanban | Plusieurs représentations d’une même base | Absent | Ajouté : mêmes fiches dans les deux vues ; changement de statut par menu ou chevrons |
| Filtres, tri et recherche de base | Réglages avancés et vues configurables | Absent | Filtre de statut, recherche titre/contenu/étiquettes, tri nom/modification/échéance/priorité ; réglages communs aux deux vues |
| Autres vues | Notamment calendrier, galerie et timeline | Absent | Toujours absent |
| Liens et backlinks | Liens de pages et de blocs, mentions, backlinks | Absent | Liens de pages en blocs via `/link` ou `/lien`, recherche de cible et liens retour automatiques ; mentions inline et ancres de blocs encore absentes |
| Relations et calculs | Relations, rollups et formules | Absent | Toujours absent ; les liens de navigation ne fournissent pas ces calculs |
| Portabilité | Exports de pages et bases, notamment Markdown/CSV | Absent | Toujours absent ; l’HTML interne ne constitue pas un export utilisateur |
| Collaboration | Partage, permissions et édition collaborative | Absent | Toujours absent ; carnet local personnel |
| Historique durable | Restauration de versions, selon l’offre | Annulation pendant l’édition de la page | Conservé ; aucune restauration de versions après fermeture |

Les fonctions de bases, propriétés et vues de Notion sont documentées dans
[Introduction aux bases](https://www.notion.com/help/intro-to-databases),
[Propriétés](https://www.notion.com/help/database-properties) et
[Vues de bases](https://www.notion.com/help/category/database-views).
La comparaison des liens suit [Liens et backlinks](https://www.notion.com/help/create-links-and-backlinks).
Les autres périmètres se réfèrent à
[Export](https://www.notion.com/help/export-your-content),
[Partage et permissions](https://www.notion.com/help/sharing-and-permissions) et
[Sauvegarde et historique](https://www.notion.com/help/back-up-your-data).

## Choix de cette itération

Les bases et les liens documentaires étaient les deux écarts les plus structurants
avec le carnet 0.2. Ils permettent maintenant un suivi de projet local : une vue
Kanban pour les statuts, une vue tableau pour les propriétés, puis une page de
notes derrière chaque fiche. Une note peut pointer vers cette fiche et retrouver
le chemin inverse grâce aux liens retour.

Les statuts proposés sont À faire, En cours et Terminé. Les priorités sont Basse,
Moyenne et Haute. Les dates sont validées au format AAAA-MM-JJ. Une fiche peut
porter jusqu’à 12 étiquettes de 40 caractères chacune. Les fiches archivées sont
exclues des vues. La recherche dans une base ne change pas le filtre global de
la barre latérale. Le filtre de statut et le tri sont conservés à la réouverture.

Les liens utilisent les identifiants stables des pages. Renommer une cible met à
jour le titre affiché et les libellés enregistrés des références. Les liens retour
ignorent les sources archivées et regroupent plusieurs liens d’une même source.
Une référence vers une cible manquante reste visible comme « Page introuvable ».

La duplication d’une page conserve le comportement 0.2 : copie de son contenu et
de son apparence, sans ses sous-pages. Pour une base, elle copie donc sa
configuration ; les fiches restent dans la base originale. L’archivage d’un parent
conserve également le comportement précédent : ses enfants actifs restent
accessibles à la racine jusqu’à restauration du parent.

## Données et compatibilité

Les propriétés, la configuration de base et les références de pages sont
conservées dans le document JSON versionné existant, avec des valeurs par défaut
pour les documents 0.2. Les identifiants de pages et de blocs sont conservés.
Les sauvegardes 0.3 utilisent la version de document 2 ; les documents de version 1
restent lisibles. L’ancienne application 0.2 refuse la version 2, ce qui évite de
réenregistrer ces documents avec perte des nouvelles propriétés.

Les pages et leurs documents sont toujours écrits dans la même transaction Turso.
Les réglages de base et propriétés passent par la même file de sauvegarde et le
même suivi des accusés de réception que les notes.

## Priorités restantes après 0.10

1. Import de Markdown/CSV et d’exports Notion avec correspondance des liens et médias.
2. Recherche indexée, pagination et navigation virtualisée pour les gros carnets.
3. Colonnes de blocs, disposition des widgets, galerie et timeline.
4. Formules plus riches et automatisations locales.
5. Synchronisation et collaboration après définition de la résolution des conflits.

Aucune mesure ne permet encore d’affirmer une parité de performance avec Notion.
Le carnet est chargé en mémoire et les index de recherche restent simples.
