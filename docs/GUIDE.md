# Guide Folio

[Retour au README](../README.md)

## Démarrer

Les dépôts doivent être voisins :

```text
repos/
├── rune/
└── folio/
```

```sh
git clone https://github.com/anomalyco/rune.git
git -C rune checkout 70eff81165e9854834a75f17753edb6c96ab2ab7
git clone https://github.com/0xrariux/folio.git
cd folio
cargo run --locked
```

Rust `1.99.1` est sélectionné par `rust-toolchain.toml`. La première compilation
peut prendre quelques minutes. Rune utilise le dépôt voisin, avec comme révision
de référence `70eff81165e9854834a75f17753edb6c96ab2ab7` de `dev`. Conserver cette
révision pour reproduire le socle ; ses API évoluent encore. Les dépendances
publiées, dont `turso 0.8.3`, sont verrouillées dans `Cargo.lock`.

Le tag Git `0.1` correspond au premier instantané publié du projet. La version
interne `0.10.0` conserve la numérotation des itérations décrites dans le changelog.

## Folio 0.10

- Pages et sous-pages, recherche dans les titres et contenus, favoris et archives.
- Blocs indépendants : texte enrichi, titres H1/H2/H3, listes à puces ou
  numérotées, tâches à cocher, citations, encadrés, code et séparateurs.
- Listes à tiroirs et titres repliables H1/H2/H3, avec six niveaux de sous-éléments.
- Déplacement, duplication et suppression des groupes avec leurs enfants.
- Raccourcis Markdown, collage de listes enrichies et numérotation par niveau.
- Menu `/` filtrable en français et en anglais (`/titre`, `/todo`, `/code`…).
- Transformation, déplacement, duplication et suppression des blocs.
- Glisser-déposer des blocs à la souris, avec aperçu, repère de dépôt et défilement automatique.
- Annulation et rétablissement au niveau du document pendant la session d’édition.
- Modèles de notes de réunion, projets, listes de tâches, suivi, dashboard et carnet à tiroirs.
- Icônes de pages, trois couvertures colorées, pleine largeur, sombre par défaut et thème clair.
- Duplication et déplacement des pages avec protection contre les cycles.
- Arborescence repliable, fil d’Ariane cliquable et navigation précédent/suivant.
- Sauvegarde automatique dans Turso et restauration à la prochaine ouverture.
- Bases de fiches, vues Tableau, Kanban, Calendrier et Dashboard partageant les mêmes pages.
- Statut, priorité, échéance et étiquettes modifiables pour chaque fiche.
- Filtres par statut, tri et recherche de fiches par nom, contenu ou étiquette.
- Liens vers les pages du carnet et liens retour automatiques.
- Colonnes texte, nombre, date, case à cocher, relation, agrégation et formule.
- Dashboards par base et dashboards globaux regroupant plusieurs bases.
- Indicateurs, graphiques et totaux recalculés depuis les fiches et les filtres.
- Vues nommées avec disposition, filtres et tri indépendants, enregistrés dans Turso.
- Export de notes en Markdown et de bases ou dashboards en CSV.
- Famille de 59 pictogrammes vectoriels Folio pour les pages et les commandes.
- Historique durable, points de restauration et sauvegarde complète du carnet.
- Sélection de plusieurs blocs et opérations groupées en une étape d’annulation.
- Déplacement à la souris des cartes Kanban, des échéances et des pages.
- Vues de bases intégrées aux notes, avec filtre sur les relations du projet.
- Relations inverses automatiques, agrégations et formules numériques.
- Images et documents locaux gérés dans le carnet, avec export des fichiers.
- Palette globale ⌘K, mentions `@`, liens de blocs et table des matières.
- Largeur, ordre et visibilité des colonnes par vue ; modifications de fiches en lot.
- Thème, largeur de la barre latérale et dernière page conservés au redémarrage.

## Sauvegardes et historique

Le menu « Plus → Votre carnet » en bas de la barre latérale regroupe sauvegarde,
restauration, historique et ajout de fichiers. Un fichier `.folio` contient les
pages, vues, colonnes, préférences, versions de pages et les fichiers joints.
La restauration présente le nombre de pages et fichiers, puis remplace le carnet
après confirmation ; une sauvegarde de l’état précédent est créée avant l’opération.
Les écritures SQL sont transactionnelles et les fichiers sont vérifiés avant restauration.

Les modifications conservent une version précédente par minute, jusqu’à 50 par
page. « Créer un point de restauration » capture explicitement l’état courant.
Restaurer une version conserve aussi cet état dans l’historique.

Des sauvegardes automatiques sont créées à l’ouverture et à la fermeture d’une
session modifiée ; les 14 dernières sont conservées dans `folio.backups`, à côté
de `folio.db`. Les fichiers joints sont dans `folio.assets`. Pour un chemin de
base personnalisé, les mêmes extensions `.backups` et `.assets` sont utilisées.
Une sauvegarde complète est limitée à 512 Mio ; les fichiers bruts qu’elle contient
sont limités ensemble à 256 Mio.

## Relations, agrégations et formules

Une colonne Relation choisit une base cible, puis ses cellules sélectionnent les
fiches liées. À la création, « Créer la relation inverse » est activé par défaut :
la base cible reçoit une colonne qui retrouve les liens automatiquement, sans
copie de valeurs à maintenir. On peut ajouter ou retirer un lien depuis les deux côtés.
Les fiches archivées sont exclues des relations affichées et des calculs.

Une Agrégation choisit une relation et calcule nombre de fiches, terminées,
progression en pourcentage, somme d’une propriété numérique ou nombre de fiches restantes.
Une Formule accepte les nombres, `+`, `-`, `*`, `/`, les parenthèses et
`prop("Nom de colonne")` ; `status_done` vaut 1 pour une fiche terminée et 0 sinon.
Par exemple : `prop("Charge totale (h)") * 80`. Les dépendances sont recalculées
à la lecture. Les cycles et divisions par zéro affichent une erreur ; les champs
calculés restent en lecture seule. Les exports CSV incluent ces résultats.

## Bases intégrées et calendrier

Tapez `/base` dans une note ou une fiche pour choisir une base existante et sa
vue enregistrée. Le bloc partage les fiches et propriétés de la source : modifier
un statut ou créer une fiche actualise également les autres vues et indicateurs.
Sur une fiche de projet, un filtre Relation peut limiter la vue aux tâches liées
à ce projet ; les nouvelles fiches créées depuis ce bloc reçoivent le lien.
La vue intégrée affiche jusqu’à 100 lignes ; « Ouvrir la base » donne accès au reste.

L’onglet Calendrier présente les échéances par mois, du lundi au dimanche.
Déplacer une fiche sur un jour change son échéance. Les fiches sans échéance
restent disponibles sous la grille. Cliquer sur un numéro de jour ouvre toutes
ses fiches et permet d’en créer une avec cette date. Les cellules Date du tableau
et les échéances utilisent aussi un sélecteur de date dans l’interface native Rune.

## Médias, navigation et opérations groupées

`/image` et `/fichier`, le menu du carnet ou le dépôt de fichiers dans une note
ajoutent des fichiers locaux. Folio copie les originaux dans son dossier géré :
déplacer le fichier source ne casse donc pas la note. Une importation accepte
jusqu’à 20 fichiers de 50 Mio chacun. Les images se redimensionnent entre 20 et
100 % de la largeur disponible. Ouvrir utilise l’application du système ; exporter
permet de récupérer l’original. L’export Markdown copie les médias dans un dossier
`folio-assets` à côté du document.

⌘K recherche globalement des pages et des commandes, avec navigation au clavier.
`@` pendant la saisie insère une mention de page ; le menu d’un bloc copie un lien
vers ce bloc. Les liens retour incluent les mentions. `/sommaire` ajoute une table
des matières qui suit les titres ; activer un lien ouvre les tiroirs nécessaires.

⌘+clic sur les poignées sélectionne plusieurs blocs, Maj+clic une plage.
La barre de sélection copie, colle, duplique, transforme, déplace ou supprime le
lot avec ses descendants, en une étape d’annulation. ⌘C/⌘V et ⌘D s’appliquent à la
sélection ; Échap la retire. Le collage de blocs internes recrée leurs identifiants.

Les cases du tableau sélectionnent plusieurs fiches pour modifier ensemble leur
statut, priorité ou échéance. Dans la barre latérale, déposer une page avant/après
une autre règle l’ordre, au centre l’imbrique, et dans la zone racine la détache.
Les cycles sont refusés ; Échap annule un déplacement en cours.

Le thème sombre reste le choix initial. Le thème choisi, la largeur de la barre
latérale et la dernière page ouverte sont conservés dans les préférences du carnet.

## Pictogrammes Folio

Les pages et commandes utilisent une famille SVG propre à Folio, avec une grille
de 24 unités, un trait de 1,5 et des extrémités arrondies. Leur couleur suit le
thème clair ou sombre. Le remplacement s’applique à la navigation, aux titres,
aux fiches, aux liens, aux sélecteurs, aux encadrés, aux menus et à l’éditeur.
Les 43 signes de commandes couvrent notamment recherche, favoris, filtres,
colonnes, export, types de blocs et poignées de déplacement. Les boutons sans
texte ont un libellé accessible et une infobulle. Le texte des notes reste
inchangé ; un emoji personnalisé hors du catalogue garde son rendu d’origine.

Cliquez sur l’icône d’une page pour ouvrir la grille des 15 pictogrammes. Les huit
choix initiaux conservent leurs identifiants et les pages existantes utilisent
automatiquement leur nouveau dessin, sans réécriture de la base. Calendrier,
Dossier, Repère, Lien et Archive utilisent un identifiant `folio:…` dans le même
champ ; le format de document actuel est la version 6.

Les [sources SVG et la planche claire/sombre](../assets/icons/README.md) sont
modifiables indépendamment des notes. La scène `icon-catalog` permet de contrôler
les dessins de pages dans Rune à trois tailles ; `icon-controls` présente ceux
des commandes.

Le logo global utilise un F plein évoquant des feuilles pliées, conçu dans
Blender. Son [fichier `.blend`, ses rendus et ses SVG](../assets/brand/README.md)
sont fournis avec le projet. La barre latérale affiche sa silhouette vectorielle
dans un badge de 32 px, adaptée aux deux thèmes.

Le [comparatif avec Notion](../NOTION-PARITY.md) distingue le périmètre couvert,
les versions simplifiées et les fonctions encore à construire.

## Bases de fiches et Kanban

Cliquez sur « Plus → Nouvelle base » en bas de la barre latérale, ou choisissez le modèle
« Suivi de projet ». Le bouton « Nouveau » crée une fiche qui s’ouvre comme une
page de notes. En Kanban, les boutons de chaque colonne créent une fiche avec
le statut de cette colonne. Le fil d’Ariane revient à la base.

Les vues Tableau et Kanban affichent les mêmes fiches. Modifiez le statut depuis
le tableau ou depuis les propriétés d’une fiche ; les chevrons d’une carte
Kanban la font passer au statut précédent ou suivant. On peut aussi déplacer une
carte à la souris entre les colonnes. Les fiches archivées sont masquées.

Les quatre propriétés intégrées sont le statut (À faire, En cours, Terminé), la
priorité (Basse, Moyenne, Haute), une échéance au format AAAA-MM-JJ et des étiquettes.
Cliquez sur l’échéance d’une ligne ou sur `•••` dans une carte pour modifier date
et étiquettes. Les dates sont validées, y compris les années bissextiles ; une
date vide efface l’échéance. Les étiquettes sont séparées par des virgules, avec
au plus 12 étiquettes de 40 caractères chacune. Annuler conserve les valeurs.

Les filtres de statut, priorité et échéance se combinent avec un ET logique.
Quatre tris sont proposés : nom, modification récente, échéance croissante
(fiches sans date à la fin) et priorité haute en premier. Ces réglages sont
enregistrés dans la vue nommée active. Les boutons Tableau, Kanban et Dashboard
changent sa disposition en conservant ses filtres et son tri.
La recherche de fiches est indépendante de celle de la barre latérale et inclut
les valeurs des colonnes personnalisées. « Effacer les filtres et la recherche »
revient à toutes les fiches actives de ce périmètre.

### Vues enregistrées

Le bouton « Vues enregistrées » au-dessus du tableau permet de nommer la
configuration actuelle. Folio garde une « Vue principale » avec les réglages
présents lors du premier enregistrement. Chaque vue mémorise sa disposition
(Tableau, Kanban, Calendrier ou Dashboard), ses filtres de statut/priorité/échéance,
son tri, la largeur, l’ordre et la visibilité des colonnes.
Les changements de ces réglages sont ensuite enregistrés dans la vue active.

Choisissez une autre vue dans ce menu pour retrouver ses réglages. Le nom de
la vue active apparaît sur le bouton. Vous pouvez renommer ou retirer une vue ;
retirer une vue conserve les fiches et les colonnes. La dernière vue est conservée.
Une base accepte jusqu’à 20 vues, avec des noms uniques de 1 à 40 caractères.
La vue active est retrouvée à la réouverture. La recherche est temporaire et
s’efface au changement de vue ; le schéma des colonnes reste commun à la base. Les dashboards globaux conservent leurs réglages propres.

### Exporter ses données

Le menu `•••` de la page propose « Exporter en Markdown » pour les notes et
les fiches. Les blocs, cases à cocher, indentation, code et propriétés d’une
fiche sont exportés. Les styles du texte utilisent du HTML inline dans le
Markdown pour conserver notamment le soulignement. Les liens internes conservent
leur titre et leur identifiant ; cet export de page seule ne
crée pas les fichiers des pages liées ni des sous-pages.

Pour une base ou un dashboard, « Exporter la vue en CSV » respecte les filtres,
la recherche et le tri courants. « Toutes les fiches en CSV » ignore les filtres
et la recherche. Sur un dashboard global, les deux commandes respectent le
périmètre des bases sélectionnées. Les bases et fiches archivées restent exclues.
Le CSV contient les identifiants, propriétés intégrées, notes en texte brut et
toutes les colonnes personnalisées, y compris masquées. Les nombres gardent
leur valeur complète. Les colonnes de bases distinctes restent séparées.

Une boîte native permet de choisir le fichier de destination et de confirmer
son remplacement s’il existe. L’écriture se fait en arrière-plan dans un fichier
temporaire, remplacé seulement une fois l’export complet. Une confirmation ou
une erreur apparaît dans l’application. Annuler le choix de fichier n’écrit rien.
Ces exports servent à récupérer le contenu ; ils ne remplacent pas une sauvegarde
complète du carnet, accessible depuis « Votre carnet ». La restauration importe
un fichier `.folio` ; l’import de Markdown, CSV ou d’un export Notion reste absent.

## Tableaux et colonnes personnalisées

Le bouton « Colonnes » d’une base permet d’ajouter jusqu’à 12 colonnes. Les types
proposés sont Texte, Nombre, Date, Case à cocher, Relation, Agrégation et Formule. Le nom peut être renommé ; le
type est conservé après création pour protéger les valeurs déjà enregistrées.
Chaque nom contient de 1 à 40 caractères et doit être unique parmi les colonnes
personnalisées de la base.

Cliquez sur une cellule pour modifier sa valeur ou directement sur une case
pour la cocher. Une valeur vide efface la cellule. Les nombres acceptent la
virgule ou le point décimal ; les valeurs non finies et dates invalides sont
refusées. Annuler conserve la valeur précédente. Les mêmes propriétés sont
accessibles dans la page de la fiche.

Vous pouvez masquer et réafficher les colonnes intégrées ou personnalisées.
Masquer une colonne conserve toutes ses valeurs. Les flèches du menu réordonnent
les colonnes ; les poignées du tableau règlent leur largeur. Ces réglages suivent
la vue enregistrée. Un tableau trop large se
parcourt horizontalement. La suppression et le changement de type d’une colonne
ne sont pas proposés dans cette version.

## Dashboards de suivi dynamique

Dans une base, l’onglet « Dashboard » affiche ses indicateurs. Le bouton « Vue
d’ensemble » de la barre latérale ouvre un dashboard global ; le modèle
« Dashboard de suivi » permet d’en créer d’autres. « Bases suivies » choisit les
bases à regrouper. Une sélection vide signifie qu’aucune base n’est suivie ;
« Toutes les bases » inclut aussi celles qui seront créées plus tard.

Chaque dashboard affiche :

- Le nombre de fiches, de fiches terminées et de fiches en cours.
- L’avancement, égal au nombre de fiches terminées divisé par le total, affiché
  en pourcentage entier. Un périmètre vide affiche 0 %.
- Les fiches en retard et celles arrivant à échéance dans les sept jours.
- La répartition par statut et priorité, sous forme de barres cliquables.
- Les sommes des colonnes numériques visibles et les décomptes des cases cochées.
- L’avancement par base dans un dashboard global, les six premières échéances
  ouvertes à surveiller et le tableau des fiches derrière les chiffres.

Cliquer sur un indicateur de statut ou d’échéance applique ce filtre ; cliquer de
nouveau le retire. Les graphiques appliquent le même principe pour statut et
priorité. La recherche et les filtres s’appliquent aussi aux totaux, graphiques
et détails. « Réinitialiser » efface ces filtres et la recherche en conservant
les bases choisies. Les tableaux de détails ouvrent les fiches et permettent de
modifier leur statut, ce qui recalcule immédiatement les indicateurs.

Un dashboard global possède ses propres filtres et ne reprend pas les filtres
individuels des bases sources. Les bases et fiches archivées sont exclues.
Les échéances utilisent la propriété intégrée Échéance : une fiche est en retard
si sa date précède le jour local et si elle n’est pas terminée. La période des
sept jours va d’aujourd’hui à aujourd’hui + 7 jours inclus, sans les fiches
terminées. La date locale est revérifiée périodiquement, avec un réveil par minute.
Le mode démo fixe cette date au 10 novembre 2026 pour rester reproductible.

Les totaux numériques restent séparés par base et identifiant de colonne, même
si deux colonnes portent le même nom. Ils s’affichent avec au plus deux décimales,
en notation scientifique pour les très grands nombres ;
un dépassement de capacité affiche « Hors limite ». Les cases non renseignées
comptent comme non cochées. Les nombres invalides ne participent pas aux sommes.
Les changements locaux de fiches et de configuration recalculent les widgets,
sans compteurs saisis manuellement. Les connexions à des sources externes et la
réorganisation libre des widgets restent à construire.

## Relier ses notes

Dans un bloc vide, tapez `/link` ou `/lien`, validez puis recherchez une page du
carnet. Le bloc devient un lien cliquable. Depuis le sélecteur de type, créer un
lien à partir d’un texte existant insère le lien après ce texte et conserve son
contenu. Vous pouvez transformer de nouveau un lien pour changer sa destination.

La cible affiche automatiquement les pages qui la référencent dans la section
« Liens retour ». Renommer une cible met à jour les libellés des références sans
changer leur destination. Les sources archivées sont masquées dans les liens
retour ; une cible archivée peut toujours être ouverte et restaurée. Une cible
manquante affiche « Page introuvable ». Les mentions `@` dans le texte et les liens
vers un bloc précis complètent ces références.

## Écrire et organiser

Tapez `/` au début d’un bloc, filtrez les commandes puis choisissez avec ↑/↓ et
Entrée. Échap ferme le menu en conservant le texte saisi. Le sélecteur de type
de la barre d’outils transforme le bloc actif sans supprimer son contenu.

Le bouton `+` d’un bloc ajoute un bloc juste après lui. Son bouton `⠿` ouvre
le menu pour monter, descendre, dupliquer ou supprimer ce bloc. Maintenez aussi
le bouton gauche sur cette poignée, déplacez le bloc et relâchez au-dessus ou
au-dessous d’un autre bloc : une ligne verte indique sa future position. La zone
« Ajouter un bloc » accepte un dépôt en fin de liste. Un aperçu suit la souris ;
l’ordre n’est modifié et sauvegardé qu’au relâchement.

Cela fonctionne pour les listes à puces, numérotées, les tâches et les autres
blocs, même de hauteurs différentes. Les identifiants, styles, cases cochées et
indentations sont conservés ; la numérotation est recalculée dans le nouvel ordre.
Chaque geste déplace un bloc avec tous ses sous-éléments, même masqués. Le groupe
adopte le niveau de sa cible et conserve les retraits relatifs de ses enfants.
Un dépôt qui dépasserait les six niveaux de retrait est ignoré.
La sélection et l’édition de texte restent disponibles dans le contenu.

Approchez le pointeur des bords haut ou bas de la zone du document pour faire
défiler une longue page pendant le geste. Échap, une perte de focus de la fenêtre
ou un relâchement hors de l’éditeur annulent le déplacement sans écrire de données.
Un simple clic sur `⠿` garde son menu ; au clavier, Espace ouvre le menu de la
poignée sélectionnée. Le déplacement constitue une seule étape d’annulation,
accessible par le bouton de la barre d’outils ou ⌘Z après le dépôt.

Entrée sépare le bloc à la position du curseur, en conservant les styles. Les
titres continuent en texte ; listes et tâches continuent avec le même type.
Entrée dans une sous-liste vide la désindente ; à la racine, elle revient au texte normal.
Maj+Entrée insère une ligne
dans le même bloc. Dans le code, Entrée insère directement une ligne.
Tab et Maj+Tab modifient l’indentation d’un élément avec ses enfants.

### Tiroirs et listes imbriquées

Tapez `/tiroir` ou `/toggle` pour une liste à tiroirs. Le menu `/` propose aussi
trois titres repliables. Leur chevron masque ou affiche tous les sous-éléments ;
l’état des tiroirs imbriqués reste indépendant et est conservé à la réouverture.
Entrée dans leur titre ajoute un premier bloc de contenu. Un tiroir vide ouvert
propose aussi « Ajouter du contenu au tiroir ». Le menu de bloc permet d’ajouter
un sous-élément à n’importe quel bloc, jusqu’à six niveaux de retrait.

Tab place le groupe sous son frère précédent ; Maj+Tab le sort de son parent,
après les autres enfants de celui-ci. Les deux boutons à droite de la barre
d’édition proposent les mêmes gestes. Retour arrière au début d’un sous-élément
le désindente ; au début d’une liste racine, il la transforme en texte sans
supprimer son contenu. Le menu du bloc et Alt+Maj+↑/↓ déplacent un groupe parmi
ses frères. Dupliquer et supprimer portent également sur tout son sous-arbre.
Toutes ces opérations se restaurent avec Annuler.
↑ au début d’un bloc et ↓ à sa fin passent au bloc visible précédent ou suivant,
en sautant le contenu des tiroirs fermés.

La numérotation continue après les sous-listes et recommence pour chaque nouveau
parent. Le menu d’un élément numéroté propose « Recommencer à 1 » ou « Continuer
la numérotation ». Saisir `5.` suivi d’un espace crée une séquence démarrant à 5.

Au début d’un bloc de texte vide, saisir le préfixe puis l’espace transforme le bloc :

| Préfixe | Résultat |
| --- | --- |
| `-`, `*` ou `+` | Liste à puces |
| `1.` | Liste numérotée |
| `[]`, `[ ]` ou `[x]` | Tâche vide ou cochée |
| `>` | Tiroir |
| `>#`, `>##` ou `>###` | Titre repliable |
| `#`, `##` ou `###` | Titre |
| `"` | Citation |
| Trois accents graves | Code |
| `---` | Séparateur |

Coller plusieurs lignes crée des blocs séparés, avec une seule étape d’annulation.
Les listes HTML conservent leurs styles et retraits ; les listes en texte brut
reconnaissent les préfixes et quatre espaces ou une tabulation par niveau.
Le code conserve ses retours à la ligne dans un seul bloc. Maj+Entrée reste un
retour dans le même élément pendant la saisie.

Les tiroirs affichent la progression de leurs tâches, même fermés. Le menu de
bloc permet de cocher ou décocher toute la liste courante, y compris ses tâches
masquées, sans modifier les listes séparées par un autre bloc racine. ⌘Entrée
coche une tâche ou replie/déplie un tiroir. Le menu de page permet de tout replier
ou déplier. Le modèle « Carnet à tiroirs » donne un exemple utilisable immédiatement.

Les éléments masqués restent recherchés et exportés. Les tiroirs sont exportés
en HTML `<details>` avec leur contenu et leur état dans le fichier Markdown ;
leur interactivité dépend du lecteur utilisé.

Cliquez sur l’icône d’une page pour la changer. Le bouton de couverture fait
alterner trois couleurs puis retire la couverture. Le menu `•••` contient la
duplication, le déplacement, la pleine largeur et l’archivage. Dupliquer une
page copie son contenu et son apparence ; ses sous-pages restent rattachées à
l’original. Archiver une page conserve les données : ses enfants actifs restent
accessibles à la racine jusqu’à sa restauration. Aucune suppression définitive
de page n’est proposée.

| Raccourci | Action |
| --- | --- |
| ⌘N | Nouvelle page |
| ⌘K | Ouvrir la palette globale de pages et commandes |
| ⌘S | Relancer les sauvegardes en erreur |
| ⌘B / ⌘I / ⌘U | Gras, italique, souligné |
| ⌘D dans un bloc | Dupliquer ce bloc et ses enfants |
| Tab / Maj+Tab | Indenter / désindenter le groupe |
| Alt+Maj+↑ / ↓ | Monter / descendre le groupe |
| ⌘Entrée | Cocher une tâche ou ouvrir/fermer un tiroir |
| ⌘Z / ⌘⇧Z dans un bloc | Annuler / rétablir dans le document |

Sous Windows et Linux, utiliser Ctrl à la place de ⌘. L’annulation immédiate
est limitée à la page actuellement ouverte. L’historique durable du carnet
permet également de restaurer des versions après un redémarrage. Dans le titre, Entrée passe au premier bloc.

## Données et migration

Par défaut, la base est créée dans le dossier de données utilisateur :

- macOS : `~/Library/Application Support/Folio/folio.db`
- Linux : `$XDG_DATA_HOME/Folio/folio.db`, sinon `~/.local/share/Folio/folio.db`
- Windows : `%LOCALAPPDATA%/Folio/folio.db`

Pour isoler un carnet de développement :

```sh
FOLIO_DB_PATH="$PWD/data/dev.db" cargo run --locked
```

Les bases Folio 0.1 sont converties automatiquement à l’ouverture. La migration
ajoute les documents de blocs sans remplacer les lignes de pages initiales :
titres, texte enrichi, parenté, favoris et indentation des listes sont conservés.
Les identifiants des blocs convertis restent identiques aux ouvertures suivantes.
Une version de document inconnue produit une erreur explicite.

Les documents 0.2, 0.3 et 0.4 restent lisibles avec des valeurs par défaut pour les
nouveaux champs. Folio 0.5 à 0.8 enregistrent la version de document 4, ajoutant les vues
nommées au schéma et aux cellules personnalisés, aux filtres combinés et aux
configurations de dashboard. Les anciens Folio refusent cette version plutôt
que de supprimer les nouvelles données. Les identifiants des pages et blocs
sont conservés. Les bases existantes gardent leurs réglages et n’obtiennent pas
de nouvelles vues avant un premier enregistrement explicite.

Folio 0.9 enregistrait le format 5 : tiroirs, état replié et points de redémarrage
de numérotation. Les versions 1 à 4 restent lisibles, avec des tiroirs ouverts
et une numérotation automatique par défaut. Les anciens clients refusent le
format 5. Les données existantes restent intactes jusqu’à leur prochaine modification.

Folio 0.10 écrit le format 6 et lit les formats 1 à 5 : vues intégrées, médias,
relations et calculs, disposition des tableaux et ordre des pages. Les anciens
clients refusent le format 6.

La source du document est un JSON versionné dans `page_content`. Chaque bloc
possède un identifiant stable, un type, du texte enrichi produit par
`TextEdit::to_html`, une indentation et l’état de sa case à cocher. Les propriétés
d’apparence de la page sont conservées avec le document. Un HTML dérivé et le
texte brut restent dans `pages` pour la compatibilité et la recherche. Rune
interprète le texte enrichi dans ses éditeurs natifs ; l’application n’embarque
ni navigateur ni moteur JavaScript.

La base s’ouvre sur un thread dédié. Chaque modification envoie une copie de la
page à ce thread. La page et son document de blocs sont enregistrés dans la même
transaction. « Tout est enregistré » attend l’accusé de réception de toutes les
écritures. Une erreur reste visible et peut être relancée en cliquant sur le
statut. La fermeture normale attend les écritures en attente ; un arrêt forcé
peut perdre les changements qui n’ont pas encore été confirmés.

Utiliser « Plus → Votre carnet → Sauvegarder le carnet complet… » pour créer une
sauvegarde pendant la session. Pour copier manuellement la base et ses dossiers,
fermer d’abord l’application.
Aucun compte, serveur ou jeton n’est nécessaire. La synchronisation Turso Cloud
et la collaboration ne sont pas implémentées.

## Organisation du code

| Fichier | Responsabilité |
| --- | --- |
| `src/model.rs` | Pages, recherche et contenu initial |
| `src/document.rs` | Blocs, format versionné et conversion du texte enrichi |
| `src/outline.rs`, `src/lists.rs` | Hiérarchie, tiroirs, numérotation, saisie et commandes de listes |
| `src/templates.rs` | Modèles de réunion, projet et tâches |
| `src/collection.rs` | Propriétés, vues, tris et validation des valeurs |
| `src/collections.rs` | Création et édition des fiches, filtres et tris |
| `src/saved_views.rs` | Création, sélection, renommage et retrait des vues |
| `src/export.rs`, `src/exporting.rs` | Markdown/CSV, boîte native et écriture atomique |
| `src/fields.rs`, `src/custom_fields.rs` | Colonnes, cellules et validation des valeurs |
| `src/dashboard.rs`, `src/tracking.rs` | Périmètres, filtres, indicateurs et agrégations |
| `src/linking.rs` | Références de pages et liens retour |
| `src/storage.rs`, `migrations/` | Schéma, migrations et transactions Turso |
| `src/worker.rs` | File d’écritures, erreurs et arrêt du thread |
| `src/workspace.rs`, `src/workspace_worker.rs`, `src/notebook.rs` | Sauvegardes, fichiers, préférences et historique |
| `src/calculations.rs` | Relations inverses, agrégations et formules |
| `src/selection.rs`, `src/entity_drag.rs` | Sélections groupées et déplacement des pages/fiches |
| `src/calendar.rs`, `src/table_layout.rs` | Dates et disposition des tableaux |
| `src/command_palette.rs` | Palette globale, mentions, liens de blocs et vues intégrées |
| `src/state.rs` | Session et révisions non enregistrées |
| `src/editor.rs` | Commandes, sélection, annulation et rétablissement |
| `src/block_drag.rs` | Geste souris, cible de dépôt, défilement et réorganisation |
| `src/icons.rs`, `assets/icons/` | Catalogue vectoriel, anciens identifiants et sources SVG |
| `src/navigation.rs` | Arborescence, déplacements et historique |
| `src/ui.rs`, `src/ui/` | Interface Rune et composants Rune Kit |
| `src/main.rs` | Démarrage, réveils de l’interface et raccourcis |

## Vérifier et prévisualiser

```sh
cargo test --locked --all-targets
cargo clippy --locked --all-targets --no-deps -- -D warnings
cargo fmt --package folio -- --check
```

Les tests couvrent la saisie réelle dans l’interface, les commandes `/`, les
tâches, les opérations sur les blocs, l’annulation, les caractères UTF-8 et les
styles, les modèles, la navigation, la prévention des cycles, la persistance,
la migration des anciens formats et le rollback d’une écriture partiellement échouée.
Ils couvrent aussi les vues indépendantes, leur persistance, le menu sur petite
fenêtre et les exports Markdown/CSV (contenu, annulation, erreur et reprise).
Le glisser-déposer est testé avec de vrais événements souris : déplacements
avant/après, gestes rapides, blocs de tailles différentes, annulation, clavier,
défilement continu, sauvegarde Turso et conservation du contenu enrichi.

La version 0.10 compte 95 tests : les nouveaux parcours couvrent notamment les
relations inverses, calculs, sélections multiples, déplacement des pages et
fiches, calendrier, médias, historique et restauration complète avec ses fichiers.
Le rendu natif a aussi été contrôlé sur la page projet et le calendrier, ainsi
que les formulaires et la palette à 920 × 620.

Les commandes de démonstration ci-dessous génèrent les captures dans `artifacts/`.
Ces fichiers temporaires restent locaux ; les rendus du logo livrés avec le
projet sont disponibles dans [assets/brand](../assets/brand/README.md).

Le mode démo conserve les modifications en mémoire et ne crée aucune base :

```sh
FOLIO_DEMO=1 cargo run --locked
FOLIO_DEMO=1 cargo run --locked -- --scene workspace
FOLIO_DEMO=1 cargo run --locked -- --scene calendar
FOLIO_DEMO=1 cargo run --locked -- --script scripts/light.json --snap artifacts/folio-v2-light.png --tree artifacts/folio-v2-light.json
FOLIO_DEMO=1 cargo run --locked -- --script scripts/dark.json --snap artifacts/folio-v2-dark.png --at 0.8
FOLIO_DEMO=1 cargo run --locked -- --script scripts/slash.json --snap artifacts/folio-v2-slash.png --at 1.2
FOLIO_DEMO=1 cargo run --locked -- --script scripts/project.json --snap artifacts/folio-v2-project.png --at 0.8
FOLIO_DEMO=1 cargo run --locked -- --script scripts/board.json --snap artifacts/folio-v3-board.png --at 0.8
FOLIO_DEMO=1 cargo run --locked -- --script scripts/table.json --snap artifacts/folio-v3-table.png --at 0.8
FOLIO_DEMO=1 cargo run --locked -- --script scripts/link-picker.json --snap artifacts/folio-v3-links.png --at 1.6
FOLIO_DEMO=1 cargo run --locked -- --scene dashboard --snap artifacts/folio-v4-dashboard.png --size 1200x1120 --at 0.6
FOLIO_DEMO=1 cargo run --locked -- --scene table-custom --snap artifacts/folio-v4-table.png --at 0.6
FOLIO_DEMO=1 cargo run --locked -- --scene saved-views --script scripts/views.json --snap artifacts/folio-v5-views.png --at 0.8
FOLIO_DEMO=1 cargo run --locked -- --scene late-view --snap artifacts/folio-v5-late.png --size 1200x1120 --at 0.6
FOLIO_DEMO=1 cargo run --locked -- --scene drag-list
FOLIO_DEMO=1 cargo run --locked -- --scene drag-list --script scripts/drag-list.json --snap artifacts/folio-v6-drag.png --at 0.8
FOLIO_DEMO=1 cargo run --locked -- --scene drag-list --script scripts/drag-list.json --snap artifacts/folio-v6-drop.png --at 1.3
FOLIO_DEMO=1 cargo run --locked -- --scene icon-catalog
FOLIO_DEMO=1 cargo run --locked -- --scene icon-controls
FOLIO_DEMO=1 cargo run --locked -- --scene lists
FOLIO_DEMO=1 cargo run --locked -- --scene lists --snap artifacts/folio-v9-lists-dark.png --size 1200x1060 --at 0.6
FOLIO_DEMO=1 cargo run --locked -- --scene lists --script scripts/light.json --snap artifacts/folio-v9-lists-light.png --size 1200x1060 --at 0.6
FOLIO_DEMO=1 cargo run --locked -- --scene icon-catalog --script scripts/light.json --snap artifacts/folio-v7-icons-light.png --size 1200x1000 --at 0.6
```

## Limites actuelles

Folio reste un carnet personnel local. La synchronisation, la collaboration,
les permissions, la disposition libre des widgets, les colonnes de blocs,
les vues galerie/timeline et l’import d’exports Notion restent à construire.
Les formules couvrent les calculs numériques simples, pas le langage complet de Notion.

L’ensemble du carnet est chargé en mémoire et la recherche est textuelle simple.
Pour de gros carnets, il faudra paginer les pages, virtualiser la navigation et
indexer la recherche dans Turso. Aucune mesure comparative de performances
avec Notion n’a été réalisée.

Références : [Rune](https://github.com/anomalyco/rune),
[SDK Rust Turso](https://docs.turso.tech/sdk/rust/quickstart).
