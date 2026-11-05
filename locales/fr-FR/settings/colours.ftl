settings-colours-source = D’où vient la couleur
settings-colours-source-desc = Suivre chaque fond, garder une palette ou laisser les couleurs telles quelles.
settings-colours-source-follow = suivent chaque fond
settings-colours-source-fixed = viennent d’une palette fixe
settings-colours-source-off = restent telles quelles
settings-colours-source-say = Les couleurs { $policy }.
settings-colours-source-made = Les couleurs { $policy }, créées par { $authority }.
settings-colours-end4 = end-4
settings-colours-control-menu = Contrôle des couleurs · shells détectés
settings-colours-generator-menu = Générateur · installés seulement
settings-colours-theme-menu = Thème · préréglages, vôtres, germes
settings-colours-all-presets = Les { $count } préréglages…
settings-colours-generator = Générateur
settings-colours-generator-desc = Le programme qui transforme le fond en palette.
settings-colours-generator-say = { $engine } crée la palette.
settings-colours-generator-seeded = { $engine } crée la palette, germe de l’emplacement { $index }, contraste { $contrast }.
settings-colours-generator-shell = { $shell } crée la palette à partir du fond.
settings-colours-palette = Palette
settings-colours-palette-desc = Une palette pour tous les fonds.
settings-colours-palette-say = { $theme } pour tous les fonds.
settings-colours-idle = Inutilisé tant que les couleurs restent telles quelles.
settings-colours-index-hint = Emplacement de palette utilisé comme germe
settings-colours-contrast-hint = -1 à 1
settings-colours-wallust-hint = Force une palette wallust, p. ex. dark16. Vide : selon le mode
settings-colours-seeds-hint = Utilisé par Germes personnalisés
settings-colours-config-hint = Matugen externe
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = Auto
settings-colours-look = Apparence
settings-colours-look-desc = Force de diffusion de la couleur, finition, clair ou sombre.
settings-colours-look-say = Caractère { $scheme }, finition { $finish }, variante { $variant }.
settings-colours-look-imported = Palette de { $shell }, importée en variante { $variant }.
settings-colours-look-shell = { $shell } définit sa propre apparence.
settings-colours-look-fixed = La palette fixe définit sa propre apparence.
settings-colours-no-designer = Le concepteur de thèmes est disponible avec Suivre le fond ou Palette fixe.
settings-colours-designer-hint = Ajuster ou créer une palette à la main
settings-colours-go = Où vont les couleurs
settings-colours-go-desc = Thèmes d’applis qui suivent la palette quand elle change.
settings-colours-find-hint = Revérifier après avoir installé une appli
settings-colours-matugen-hint = Interrupteur des sorties Matugen
settings-colours-reload-hint = Exécuté après l’écriture du fichier
settings-colours-output-heading = Sortie { $count }
settings-colours-shells = Shells
settings-colours-shells-desc = Noctalia et Dank Material Shell peuvent prendre les couleurs de Skwd.
settings-colours-dms-hint = DMS doit utiliser Auto (fond)
settings-colours-mode-hint = Appliqué avec chaque schéma de couleurs
settings-colours-bin-hint = S’il n’est pas dans le PATH
settings-colours-lock = Écrans de verrouillage
settings-colours-lock-desc = Écrans de verrouillage qui affichent votre fond.
settings-colours-lock-follow = le dernier fond appliqué
settings-colours-lock-still = une image fixe
settings-colours-sddm-hint = Approbation administrateur requise
settings-colours-go-say = { $val }{ $on } applis sur { $all }{ $end } { $followers ->
        [one] suit
       *[some] suivent
    } vos couleurs{ $changed ->
        [one] ; { $val }la config de { $name } a changé hors de Skwd{ $end }.
        [some] ; { $val }{ $count } applis ont changé hors de Skwd{ $end }.
       *[none] .
    }
settings-colours-outputs-say = { $state ->
        [none] Aucune intégration ne génère encore de modèle.
        [single] { $val }1 intégration{ $end } génère un modèle dans un fichier après chaque changement de couleur.
       *[many] { $val }{ $count } intégrations{ $end } génèrent des modèles dans des fichiers après chaque changement de couleur.
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] suit les couleurs de Skwd
       *[keeps] garde ses couleurs
    }{ $end } ; DMS { $val }{ $dms ->
        [follows] suit les couleurs de Skwd
       *[keeps] garde ses couleurs
    }{ $end }{ $caelestia ->
        [yes]  ; les fonds de Caelestia sont synchronisés.
       *[no] .
    }
settings-colours-lock-say = { $state ->
        [none] Aucun écran de verrouillage configuré. Les nouveaux utilisent { $mode }.
        [one] { $names } utilise { $mode }.
       *[some] { $names } utilisent { $mode }.
    }
settings-colours-lock-say-static = { $state ->
        [none] Aucun écran de verrouillage configuré. Les nouveaux utilisent { $mode } : { $image }.
        [one] { $names } utilise { $mode } : { $image }.
       *[some] { $names } utilisent { $mode } : { $image }.
    }
settings-menu-more = Plus
