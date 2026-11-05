settings-wallpapers-displays-desc = Ce que chaque écran affiche et son placement.
settings-wallpapers-play = Lecture des fonds d’écran
settings-wallpapers-play-desc = Moteurs, couche d’affichage et scènes.
settings-wallpapers-transitions-desc = Comment un fond d’écran passe au suivant.
settings-wallpapers-awww-desc = La transition d’awww pour les images.
settings-wallpapers-drawing = Dessin
settings-wallpapers-drawing-desc = Les fonds préparés se dessinent à l’écran.
settings-wallpapers-sound = Son
settings-wallpapers-sound-desc = Volume, et quand l’audio du fond s’efface.
settings-wallpapers-pause-desc = Mettre en pause les fonds animés selon une règle.
settings-wallpapers-plasma = Écran de verrouillage Plasma
settings-wallpapers-plasma-desc = Le fond affiché par Plasma une fois verrouillé.
settings-wallpapers-niri = Vue d’ensemble Niri
settings-wallpapers-niri-desc = Sur niri. Un fond derrière la vue d’ensemble.
settings-wallpapers-display-more = Plus
settings-wallpapers-lock = Verrouiller { $name }
settings-wallpapers-lock-hint = Mise à jour par le multisélecteur uniquement
settings-wallpapers-layer-bottom = Bas
settings-wallpapers-gpu-hint = Revient à Automatique
settings-wallpapers-timeout-hint = Pour les très grandes images
settings-wallpapers-animated = Fonds animés
settings-wallpapers-scenes = Scènes
settings-wallpapers-scenes-hint = Limite FPS · mise à l’échelle
settings-wallpapers-particles-hint = Scènes uniquement
settings-wallpapers-preview-hint = Fréquence d’images
settings-wallpapers-quality-short = Le sable demande au moins { $ms } ms.
settings-wallpapers-quality-hint = Full sans limite, Low limité à 30 fps
settings-wallpapers-sharpen-hint = Quand la qualité réduit la résolution
settings-wallpapers-monitors = Écrans
settings-wallpapers-primary-hint = Anime seulement pour Principal
settings-wallpapers-primary-blank = premier disponible
settings-wallpapers-primary-only-hint = Les autres utilisent tous les écrans
settings-wallpapers-wave = Vague
settings-wallpapers-wave-hint = Largeur × hauteur
settings-wallpapers-position-hint = center, top-left ou 0.5,0.5
settings-wallpapers-bezier-hint = x1,y1,x2,y2
settings-wallpapers-bezier-blank = défaut d’awww
settings-wallpapers-invert-hint = Inverse l’axe Y de l’origine
settings-wallpapers-colours-files = Couleurs et fichiers
settings-wallpapers-preparing = Préparation
settings-wallpapers-paper-ink = Papier et encre
settings-wallpapers-prepare = Préparer
settings-wallpapers-prepare-all = Tous les fonds
settings-wallpapers-scope-view = Vue actuelle · { $count }
settings-wallpapers-pack-ready = Fichiers de traitement prêts.
settings-wallpapers-pack-none = Pas encore de fichiers de traitement.
settings-wallpapers-pack-blank = Aucun pour l’instant
settings-wallpapers-drawing-blocked = Dessin indisponible. Choisissez d’abord les fichiers de traitement.
settings-wallpapers-volume = Volume
settings-wallpapers-other-apps = Autres applications
settings-wallpapers-pause-apps = Pause tant que les applications listées tournent.
settings-wallpapers-pause-none = Aucune règle de pause active.
settings-wallpapers-process-hint = * et ? acceptés
settings-wallpapers-process-action = Choisir
settings-wallpapers-process-blank = aucun pour l’instant
settings-wallpapers-resume-hint = Après la fin de la dernière règle
settings-wallpapers-scope-all = tous les écrans
settings-wallpapers-scope-display = l’écran de la fenêtre
settings-wallpapers-rule-fullscreen = les fenêtres plein écran
settings-wallpapers-rule-maximised = les fenêtres maximisées
settings-wallpapers-rule-focused = les fenêtres actives
settings-wallpapers-rule-full-width = les colonnes pleine largeur
settings-wallpapers-plasma-left = reste géré par Plasma
settings-wallpapers-plasma-static = affiche une image fixe
settings-wallpapers-plasma-follow = suit le dernier fond appliqué
settings-wallpapers-plasma-off-row = Non géré laisse Plasma tranquille.
settings-wallpapers-image-hint = Chemin absolu, ou commençant par ~/
settings-wallpapers-image-blank = aucune image
settings-wallpapers-dynamic-hint = Image d’affiche : une image muette
settings-wallpapers-niri-off = Désactivé. La vue d’ensemble garde le fond de niri.
settings-wallpapers-source = Source
settings-wallpapers-look = Aspect
settings-wallpapers-source-blank = dernier fond
settings-wallpapers-blur = Flouter le fond
settings-wallpapers-radii = Rayon du flou
settings-wallpapers-radii-hint = Images · vidéos · scènes
settings-wallpapers-dim-hint = 0 aucun, 100 noir
settings-wallpapers-transitions-off = Désactivé. Chaque fond passe directement au suivant.
settings-wallpapers-transitions-off-foot = Activez les transitions pour choisir l’aperçu, les écrans et la qualité du sable.
settings-wallpapers-awww-none = Les images changent avec { $type }.
settings-wallpapers-awww-no-type = aucune transition
settings-wallpapers-sound-muted = L’audio des fonds est coupé.
settings-wallpapers-displays-say = { $state ->
        [none] Aucun écran verrouillé.
        [one] { $names } est verrouillé.
        [some] { $names } sont verrouillés.
       *[detecting] Détection des écrans.
    } Sans placement propre, les écrans utilisent { $fill }.
settings-wallpapers-play-say = Images avec { $backend }, vidéos avec { $val }Vulkan{ $end }, sur la couche { $layer }.
settings-wallpapers-transitions-say = Famille { $family }{ $kind ->
        [random] , un différent à chaque fois, sur { $duration }.
        [preset] , avec { $shader }, en { $ms } ms.
       *[shader] , avec { $shader }, sur { $duration }.
    } Fréquence { $rate }.
settings-wallpapers-primary-count = { $state ->
        [none] aucun des { $all }
        [all] les { $all }
       *[some] { $shown } sur { $all }
    }
settings-wallpapers-awww-say = Images en { $type } avec le filtre { $filter }, sur { $duration }.
settings-wallpapers-drawing-say = { $state ->
        [on] Les fonds préparés se dessinent en { $duration }.
       *[off] Désactivé. Les cartes réglées sur Dessin durent { $duration }.
    }
settings-wallpapers-sound-say = L’audio des fonds joue à { $volume }{ $duck ->
        [yes] { " " }et s’arrête quand une autre app émet du son.
       *[no] .
    }
settings-wallpapers-pause-say = Pause sur { $rules } sur { $scope }{ $apps ->
        [yes] , et pour les applis listées.
       *[no] .
    }
settings-wallpapers-pause-rules = { $count } règles de fenêtre
settings-wallpapers-plasma-say = { $state ->
        [off] L’écran de verrouillage { $mode }.
        [static] L’écran de verrouillage { $mode } : { $image }.
       *[follow] L’écran de verrouillage { $mode }, scènes en { $scenes ->
        [live] rendu direct.
       *[poster] image d’affiche.
    }
    }
settings-wallpapers-niri-say = Affiche { $source ->
       *[current] le fond actuel
        [own] sa propre source
        [stationary] un fond fixe
    }{ $blur ->
        [yes] , flouté
       *[no] { "" }
    }, assombri à { $dim }{ $auto ->
        [yes] , recoloré
       *[no] { "" }
    }.
settings-wallpapers-device-hint = Automatique essaie CUDA, WebGPU, puis CPU
settings-wallpapers-memory-hint = Au-delà, Automatique réessaie sur CPU
