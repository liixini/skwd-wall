settings-system-power-desc = Le sélecteur et les fonds d’écran ont des limites de batterie séparées.
settings-system-power-say = Sur batterie, { $cap ->
        [none] le sélecteur n’a pas de limite supplémentaire ({ $fps })
       *[cap] le sélecteur est limité à { $fps }
    }{ $idle ->
        [off] , seule la pause d’inactivité normale s’applique ({ $seconds })
       *[on] , la vidéo s’arrête après { $seconds } sans activité
    } et { $detail ->
        [full] les fonds d’écran gardent tous leurs détails
       *[performance] les fonds d’écran passent en mode performance
    }.
settings-system-power-off = Désactivé. Sur batterie, le sélecteur et les fonds d’écran fonctionnent avec les mêmes limites que sur secteur.
settings-system-power-source = Source : { $source }
settings-system-always = Toujours
settings-system-on-battery = Sur batterie · quand l’économie est active
settings-system-max-fps-hint = Fréquence de l’écran, ou 60 pour un portable plus frais
settings-system-idle-hint = 0 désactive
settings-system-battery-fps-hint = Le plus bas avec FPS max · 0 désactive
settings-system-battery-idle-hint = Le plus court gagne · 0 désactive
settings-system-battery-wallpaper-hint = Vidéo directe, scènes à 30 fps
settings-system-gpu-desc = Rouvrez le sélecteur après l’avoir changé.
settings-system-gpu-say = { $mode ->
        [low] Le sélecteur utilise { $gpu } : il demande un GPU intégré.
        [high] Le sélecteur utilise { $gpu } : il demande un GPU dédié.
        [none] Le sélecteur utilise { $gpu } : le pilote décide.
        [auto] Le sélecteur utilise { $gpu } : il préfère un GPU économe quand l’économie de batterie est active.
       *[custom] Le sélecteur utilise { $gpu }, qu’il ne reconnaît pas : il agit comme Auto.
    }
settings-system-language-desc = Les changements s’appliquent tout de suite et sont enregistrés.
settings-system-language-say = { $follow ->
        [desktop] L’interface utilise { $language }, la langue du bureau. Noms et étiquettes des fonds d’écran inchangés.
       *[chosen] L’interface utilise { $language }. Noms et étiquettes des fonds d’écran inchangés.
    }
settings-system-bug-title = Rapport de bug
settings-system-bug-desc = Un fichier à joindre à un ticket.
settings-system-bug-say = Regroupe la version, l’environnement et les journaux récents dans un seul fichier.
