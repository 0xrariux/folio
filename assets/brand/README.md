# Identité Folio — feuilles pliées

Un F plein avec deux terminaisons diagonales rappelant des feuilles pliées.
La tranche gauche et le pied sont arrondis ; les ouvertures restent larges pour
que le signe soit lisible dans la barre latérale. La couleur sauge et le graphite
reprennent la palette de Folio.

- [`folio-logo.blend`](folio-logo.blend) : scène Blender éditable, badge et signe
  séparés, matériaux, caméra orthographique et éclairage. Créée avec Blender 5.2.3.
- [`folio-logo.svg`](folio-logo.svg) : badge vectoriel complet en sauge/graphite.
- [`folio-logo.png`](folio-logo.png) : rendu Blender frontal 1 024 px avec transparence extérieure.
- [`folio-logo-studio.png`](folio-logo-studio.png) : présentation en léger relief sur fond graphite.
- [`../icons/folio.svg`](../icons/folio.svg) : monogramme monochrome embarqué par Rune.

Les contours du modèle Blender et des SVG viennent de la même définition dans
[`scripts/build-brand.py`](../../scripts/build-brand.py). Aucun modèle, texture
ou fichier de police externe n’est nécessaire. Le modèle comprend une très légère
épaisseur et des chanfreins ; l’interface emploie la silhouette vectorielle avec
la couleur du thème pour préserver sa lisibilité à toutes les résolutions.

Le badge natif mesure 32 px avec un rayon de 9 px et un signe de 28 px. Ces
proportions sont reprises par le badge SVG et la scène Blender.

Pour régénérer les fichiers depuis la racine du projet :

```sh
/Applications/Blender.app/Contents/MacOS/Blender \
  --background --factory-startup --threads 4 --python scripts/build-brand.py
python3 scripts/build-icon-sheet.py
```

Sous Windows ou Linux, remplacer le chemin de l’exécutable par `blender`.
La régénération remplace les exports et la scène ; conserver les modifications
manuelles éventuelles de la scène avant de relancer le script.

Les fichiers sont fournis sous la licence MIT du projet.
