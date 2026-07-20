<img align="left" width="80" height="80" src="apps/hachimi/assets/icon.png">

# HachimiRedux

[English](README.md) | [简体中文](README-zh_cn.md) | [繁體中文](README-zh_tw.md) | [Español (España)](README-es_es.md) | [Español (Latinoamérica)](README-es_419.md) | Français | [Português (Brasil)](README-pt_br.md) | [Português (Portugal)](README-pt_pt.md)

Mod d'amélioration et de traduction du jeu pour UM:PD. HachimiRedux est un fork de Hachimi doté d'un plugin de suivi d'entraînement intégré au jeu et d'un SDK de plugins natif retravaillé.

<img height="400" src="apps/hachimi/assets/screenshot-2.png">

## Table des matières

- [Merci de ne pas créer de lien vers ce dépôt ni vers le site de Hachimi](#️-merci-de-ne-pas-créer-de-lien-vers-ce-dépôt-ni-vers-le-site-de-hachimi)
- [Incompatible avec les plugins de Hachimi d'origine](#️-incompatible-avec-les-plugins-de-hachimi-dorigine)
- [Fonctionnalités](#fonctionnalités)
- [Installation](#installation)
  - [Installer avec l'installateur (recommandé)](#installer-avec-linstallateur-recommandé)
  - [Compiler depuis les sources (avancé)](#compiler-depuis-les-sources-avancé)
- [Dépannage](#dépannage)
- [Remerciements particuliers](#remerciements-particuliers)
- [Licence](#licence)

# ⚠️ Merci de ne pas créer de lien vers ce dépôt ni vers le site de Hachimi
Nous comprenons que vous souhaitiez aider les gens à installer Hachimi et à profiter d'une meilleure expérience de jeu. Cependant, ce projet va, par nature, à l'encontre des conditions d'utilisation du jeu, et les développeurs du jeu voudraient très certainement le voir disparaître s'ils venaient à en avoir connaissance.

Le partager dans vos services de discussion privés et par messages privés ne pose pas de problème, mais nous vous demandons humblement d'éviter de partager des liens vers ce projet sur des sites publics, ou vers l'un des outils impliqués.

Ou bien partagez-les et gâchez tout pour la douzaine d'utilisateurs de Hachimi. C'est à vous de voir.

### Si vous comptez le partager malgré tout
Faites ce que vous devez, mais nous vous demandons respectueusement d'essayer de désigner le jeu par « UM:PD » ou « The Honse Game » plutôt que par son vrai nom, afin d'éviter l'indexation par les moteurs de recherche.

# ⚠️ Incompatible avec les plugins de Hachimi d'origine
Ce fork est livré avec sa propre API native de plugins (host API v16). **Les plugins conçus pour le Hachimi d'origine ne sont pas compatibles avec HachimiRedux**, et le plugin de suivi d'entraînement distribué ici ne se chargera pas sur le Hachimi d'origine. Privilégiez les DLL compilées à partir de ce dépôt, et utilisez-les ensemble. Mélanger des builds peut empêcher le chargement ou faire planter le jeu.

## Compatibilité avec les plugins hérités (optionnelle, limitée)
Les plugins sans manifeste et à ABI héritée (par ex. les data-dumpers du Hachimi d'origine) peuvent être chargés via une **voie de compatibilité optionnelle**. Ajoutez la DLL à une liste d'autorisation `legacy_libraries` dans `config.json`, en plus de `load_libraries` :

```json
{
  "windows": {
    "load_libraries": ["some_legacy_plugin.dll"],
    "legacy_libraries": ["some_legacy_plugin.dll"]
  }
}
```

Un plugin hérité n'a besoin que d'exporter `hachimi_init` ; l'hôte ignore son contrôle habituel de manifeste/ABI et le charge sur confiance. Cette prise en charge est **limitée et non supportée** :

- Le plugin ne doit s'appuyer **que sur le préfixe stable de la vtable** de l'API de l'hôte. Tout ce qui va au-delà relève d'un comportement indéfini et peut faire planter le jeu.
- L'hôte **ne peut pas valider, suivre ni décharger** un plugin hérité ni ses hooks IL2CPP. La DLL reste mappée pendant toute la durée de vie du processus.
- Un avertissement est consigné chaque fois qu'un plugin se charge par cette voie.
- Les entrées de `legacy_libraries` doivent également figurer dans `load_libraries`.

En cas de doute, recompilez le plugin contre ce dépôt (host API v16) plutôt que de vous reposer sur la voie héritée.

# Fonctionnalités
- **Traductions de haute qualité :** Hachimi est doté de fonctionnalités de traduction avancées qui aident les traductions à paraître plus naturelles (formes plurielles, nombres ordinaux, etc.) et évitent d'introduire des défauts dans l'interface. Il prend aussi en charge la traduction de la plupart des composants du jeu ; aucun patch manuel des assets nécessaire !

    Composants pris en charge :
    - Texte de l'interface
    - master.mdb (noms de compétences, descriptions de compétences, etc.)
    - Histoires de courses
    - Histoire principale / dialogues du Home
    - Paroles de chansons
    - Remplacement de textures
    - Remplacement d'atlas de sprites

    De plus, Hachimi ne fournit pas de fonctionnalités de traduction pour une seule langue ; il a été conçu pour être entièrement configurable pour n'importe quelle langue.

- **Installation facile :** Il suffit de brancher et de jouer. Toute la configuration se fait au sein même du jeu, aucune application externe nécessaire.
- **Mise à jour automatique des traductions :** L'outil de mise à jour des traductions intégré vous permet de jouer normalement pendant la mise à jour, et les recharge en jeu une fois terminée, sans redémarrage nécessaire !
- **Interface graphique intégrée :** Livré avec un éditeur de configuration pour que vous puissiez modifier les paramètres sans même quitter le jeu !
- **Paramètres graphiques :** Vous pouvez ajuster les paramètres graphiques du jeu pour tirer pleinement parti des caractéristiques de votre appareil, comme le déverrouillage des FPS et la mise à l'échelle de la résolution.
- **Windows uniquement :** Conçu spécifiquement pour la version Windows (Steam) du jeu. **HachimiRedux ne prend pas en charge Android, par choix** : il se concentre uniquement sur le client Windows, et il n'est pas prévu d'ajouter ni de maintenir un build Android.

# Installation

Le moyen le plus simple d'installer HachimiRedux est l'**installateur** disponible sur la [page des Releases](https://github.com/jalbarrang/hachimi-redux/releases) : il configure pour vous le mod principal et le plugin optionnel Training Tracker, sans copie manuelle de fichiers ni édition de JSON. Si vous préférez le compiler vous-même, voir [Compiler depuis les sources](#compiler-depuis-les-sources-avancé).

HachimiRedux est le mod principal (chargé en tant que `cri_mana_vpx.dll`) ; le **Training Tracker** est un plugin DLL optionnel chargé par le mod principal. Les deux proviennent du même build.

Le répertoire du jeu est le dossier d'installation Steam, par ex.
`C:\Program Files (x86)\Steam\steamapps\common\UmamusumePrettyDerby`.

## Installer avec l'installateur (recommandé)

1. Téléchargez le `hachimi_installer.exe` le plus récent depuis la [page des Releases](https://github.com/jalbarrang/hachimi-redux/releases).
2. Lancez-le. L'installateur détecte automatiquement votre répertoire de jeu Steam ; s'il n'y parvient pas, sélectionnez-le manuellement (le chemin par défaut est indiqué ci-dessus).
3. Choisissez votre langue. Pour obtenir le Training Tracker en jeu, laissez la case **« Install Training Tracker plugin »** cochée (activée par défaut).
4. Cliquez sur **Install**. L'installateur sauvegarde le `cri_mana_vpx.dll` d'origine, installe le mod et crée `config.json` pour vous.
5. Lancez le jeu. Appuyez sur la touche de menu — par défaut la **flèche droite** — pour ouvrir l'interface en jeu.

Pour mettre à jour ou supprimer HachimiRedux plus tard, relancez simplement l'installateur (il propose une option de désinstallation).

## Compiler depuis les sources (avancé)

Ce dépôt est un workspace Cargo. Depuis la racine du dépôt :

```sh
# Mod principal
cargo build --release -p hachimi                    # -> target/release/hachimi.dll
# Plugin Training Tracker
cargo build --release -p hachimi-training-tracker   # -> target/release/hachimi_training_tracker.dll
```

## Installer HachimiRedux (cœur)

Le jeu charge le mod via la DLL de rendu `cri_mana_vpx.dll`.

1. Dans le répertoire du jeu, sauvegardez le `cri_mana_vpx.dll` d'origine sous `cri_mana_vpx.dll.backup` (faites-le une seule fois — n'écrasez jamais la sauvegarde par la suite).
2. Copiez `target/release/hachimi.dll` dans le répertoire du jeu et renommez-le en `cri_mana_vpx.dll`.
3. Lancez le jeu. Appuyez sur la touche de menu — par défaut la **flèche droite** — pour ouvrir l'interface en jeu. L'écran de démarrage affiche la touche actuelle, et vous pouvez la réassigner depuis l'interface graphique en jeu.

Les paramètres du mod se trouvent dans `config.json`, à l'intérieur du répertoire de données du jeu, qui est le **sous-dossier `hachimi` du répertoire du jeu** (par ex. `…\UmamusumePrettyDerby\hachimi\config.json`). Il est créé automatiquement par l'installateur / au premier lancement ; tout le reste se configure depuis l'interface graphique en jeu.

## Installer le plugin Training Tracker

Les plugins sont des DLL natives que le mod principal charge au démarrage depuis la racine du répertoire du jeu.

1. Installez d'abord le cœur de HachimiRedux (ci-dessus).
2. Copiez `target/release/hachimi_training_tracker.dll` à la racine du répertoire du jeu (le même dossier que `cri_mana_vpx.dll`). Remarque : la DLL du plugin va à la **racine** du jeu, tandis que `config.json` se trouve dans le sous-dossier `hachimi`.
3. Ajoutez la DLL à la liste `load_libraries` dans `config.json` (`<game_dir>\hachimi\config.json`) :

   ```json
   {
     "windows": {
       "load_libraries": ["hachimi_training_tracker.dll"]
     }
   }
   ```
4. Lancez le jeu. Le tracker apparaît sous forme de page dans l'onglet Plugins et de panneau de superposition flottant. Consultez [docs/plugin-sdk.md](docs/plugin-sdk.md) pour comprendre le fonctionnement des plugins.

## Déploiement automatisé (Windows, depuis les sources)

Depuis la racine du dépôt, le script utilitaire compile et copie les deux DLL dans le répertoire du jeu :

```powershell
.\scripts\deploy-windows.ps1 -Build
```

Remplacez le dossier du jeu s'il ne se trouve pas au chemin Steam par défaut :

```powershell
$env:HACHIMI_GAME_DIR = "D:\path\to\UmamusumePrettyDerby"
.\scripts\deploy-windows.ps1 -Build
```

Le script copie `hachimi.dll` → `cri_mana_vpx.dll` et la DLL du training tracker dans le répertoire du jeu, et ne modifie jamais `cri_mana_vpx.dll.backup`.

# Données de jeu hébergées

Le Training Tracker télécharge ses instantanés de données de jeu (catalogues GameTora, ressources du tracker, icônes de carrière) à l'exécution depuis ce dépôt, via des URLs raw GitHub sous `main/data/…`. Les instantanés sont régénérés par le workflow quotidien [Data Refresh](.github/workflows/data_refresh.yml) ; la séquence manuelle est documentée dans [docs/updating-game-data.md](docs/updating-game-data.md), avec des notes de maintenance dans [MAINTENANCE.md](MAINTENANCE.md).

**Ne renommez pas ce dépôt, la branche `main` ni le chemin `data/`** — les builds déployées codent ces URLs en dur et cesseraient de recevoir les mises à jour de données.

# Dépannage

## Le jeu plante au lancement / se comporte bizarrement

La cause de loin la plus fréquente est l'**empilement de plusieurs mods de jeu ou injecteurs de DLL** dans le dossier du jeu. Chacun s'accroche au rendu/runtime du jeu, et ils se gênent mutuellement. HachimiRedux avertit de cela en jeu (une notification + le `hachimi.log`) et l'installateur avertit avant d'installer, mais c'est à vous de supprimer les autres :

- Conservez **uniquement** HachimiRedux : `cri_mana_vpx.dll` et tout plugin compilé avec HachimiRedux (par ex. `hachimi_training_tracker.dll`).
- Supprimez du dossier du jeu les autres superpositions/injecteurs, comme les DLL de proxy-loader qui ne devraient pas s'y trouver (`version.dll`, `winhttp.dll`, `dxgi.dll`, `d3d11.dll`, `dinput8.dll`, …) et les superpositions nommées (`horseACT.dll`, `heaven_overlay.dll`, …).
- **Seuls les plugins compilés avec HachimiRedux** ont leur place dans `load_libraries`. N'y ajoutez pas de superpositions tierces — ce ne sont pas des plugins HachimiRedux et elles seront rejetées (avec un avis en jeu) ou peuvent faire planter le jeu.

## Où se trouve chaque chose

- `cri_mana_vpx.dll` et les DLL de plugins : le répertoire **racine** du jeu.
- `config.json` et les autres données du mod : le **sous-dossier `hachimi`** du répertoire du jeu (`<game_dir>\hachimi\config.json`).
- Journal du mod : `hachimi.log` à la racine du jeu (activez `enable_file_logging` dans `config.json`).
- Journal du jeu : `%USERPROFILE%\AppData\LocalLow\Cygames\Umamusume\Player.log`.

## Collecter des diagnostics

- En jeu : ouvrez le menu (flèche droite par défaut) → **Config** → **Save diagnostics report**. Cela écrit `hachimi_diagnostics.txt` dans le dossier du jeu.
- Installateur : exécutez `installer collect-logs` pour rassembler `config.json`, `hachimi.log` et un rapport de conflits dans `%TEMP%\hachimi_diagnostics`.

# Remerciements particuliers

HachimiRedux est un fork bâti sur le travail de :

- [Hachimi](https://github.com/Hachimi-Hachimi/Hachimi) — le projet d'origine dont il est issu. Si le projet d'origine vous intéresse, rejoignez [son serveur Discord](https://discord.gg/YjBgmuqqYr).
- [Hachimi Edge](https://github.com/kairusds/Hachimi-Edge) — le fork axé Windows/Steam dont HachimiRedux est la continuité.

Ces projets ont à leur tour servi de base au développement de Hachimi ; sans eux, Hachimi n'aurait jamais existé sous sa forme actuelle :

- [Trainers' Legend G](https://github.com/MinamiChiwa/Trainers-Legend-G)
- [umamusume-localify-android](https://github.com/Kimjio/umamusume-localify-android)
- [umamusume-localify](https://github.com/GEEKiDoS/umamusume-localify)
- [Carotenify](https://github.com/KevinVG207/Uma-Carotenify)
- [umamusu-translate](https://github.com/noccu/umamusu-translate)
- [frida-il2cpp-bridge](https://github.com/vfsfitvnm/frida-il2cpp-bridge)

# Licence
[GNU GPLv3](LICENSE)
