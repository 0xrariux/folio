# Pictogrammes Folio

Une famille de 59 dessins SVG créés pour Folio : 15 pictogrammes de pages,
un monogramme et 43 signes de commandes, navigation, blocs et propriétés.
Les fichiers sont fournis sous la licence MIT du projet.

- Grille `24 × 24`, trait `1.5`, extrémités et jointures arrondies pour les pictogrammes.
- Couleur `currentColor` : les mêmes fichiers servent en clair et en sombre.
- Formes SVG simples, sans polices ni ressources externes ; aucune dépendance à Blender à l’exécution.
- Le monogramme utilise un F plein avec deux coupes diagonales évoquant des feuilles
  pliées. Son [fichier Blender et ses exports](../brand/README.md) sont éditables ;
  le SVG embarqué partage exactement le même contour et reste net à petite taille.

[Planche des pages et du monogramme](catalog.svg) ·
[Planche des commandes](controls.svg). Chaque planche comprend les deux thèmes.

Les noms et correspondances avec les anciens emojis vivent dans
[`src/icons.rs`](../../src/icons.rs). Les commandes et correspondances avec les
types de blocs vivent dans [`src/icons/controls.rs`](../../src/icons/controls.rs).
Rune charge les SVG embarqués avec son type
`Icon` ; les fichiers exportés sont les sources effectivement utilisées par
l’interface. Les notes gardent leurs identifiants et leur texte.

Après modification d’un SVG ou du catalogue, régénérer les deux planches avec :

```sh
python3 scripts/build-icon-sheet.py
```

Contrôler le rendu réel aux trois tailles dans la scène de démonstration :

```sh
FOLIO_DEMO=1 cargo run --locked -- --scene icon-catalog
FOLIO_DEMO=1 cargo run --locked -- --scene icon-controls
```

Les noms de fichier correspondent aux identifiants du catalogue. Les anciens
emojis restent reconnus ; le catalogue central définit les nouveaux identifiants.
