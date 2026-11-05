settings-automation-rotation-desc = Applique un fond d’écran aléatoire du lot à chaque fin d’intervalle.
settings-automation-rotation-say = Intervalle de { $interval } : un choix au hasard parmi { $val }{ $pool }{ $end }.
settings-automation-rotation-off = Désactivé. Le fond d’écran ne change que lorsque vous en choisissez un.
settings-automation-pool = { $count ->
        [none] un lot vide
        [one] { $first }
        [two] { $first } et { $second }
       *[three] { $first }, { $second } et { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] { $pool } (favoris uniquement)
       *[all] { $pool }
    }
settings-automation-pool-images = les images
settings-automation-pool-videos = les vidéos
settings-automation-pool-engine = les éléments Wallpaper Engine
settings-automation-pool-heading = Le lot contient
settings-automation-timing = Rythme
settings-automation-interval-hint = 10 s minimum
settings-automation-schedule-desc = Des règles choisissent le fond d’écran et le thème selon l’heure, le jour, la météo, l’alimentation et les écrans. La première règle qui correspond l’emporte.
settings-automation-schedule-say = { $start ->
        [applies] Les règles sont actives ; la première règle qui correspond { $val }s’applique au démarrage{ $end }. Un fond d’écran choisi par vous l’emporte toujours.
       *[waits] Les règles sont actives ; la première règle qui correspond { $val }attend le prochain changement{ $end }. Un fond d’écran choisi par vous l’emporte toujours.
    }
settings-automation-schedule-off = Désactivé. Vos règles sont conservées jusqu’à ce que vous l’activiez.
settings-automation-start-hint = première règle qui correspond au démarrage du service
settings-automation-editor-hint = créer des règles à partir de blocs de conditions
settings-automation-location = Lieu et météo
settings-automation-location-desc = Sert aux heures de lever et de coucher du soleil et aux conditions météo.
settings-automation-location-say = Météo pour { $city }, heures du soleil à { $lat }, { $lon }. Changements selon la météo : { $mode }.
settings-automation-no-city = aucune ville
settings-automation-no-latitude = aucune latitude
settings-automation-no-longitude = aucune longitude
settings-automation-location-hint = Laissez la ville vide pour utiliser les coordonnées. La latitude est positive au nord, la longitude à l’est.
settings-automation-startup = Démarrage et notifications
settings-automation-startup-desc = Ce qui se passe au démarrage du service et quand le fond d’écran change.
settings-automation-startup-say = { $restore ->
        [restores] Au démarrage, Skwd { $val }réapplique le dernier fond d’écran{ $end }.
       *[leaves] Au démarrage, Skwd { $val }laisse le bureau tel quel{ $end }.
    } { $notify ->
        [notifies] Les changements de fond d’écran { $val }envoient une notification{ $end }.
       *[silent] Les changements de fond d’écran { $val }restent silencieux{ $end }.
    }
settings-automation-commands = Après l’application d’un fond d’écran
settings-automation-commands-desc = Exécutez vos propres commandes après chaque changement, par exemple pour piloter un autre outil de fond d’écran.
settings-automation-command-count = { $count ->
        [one] { $val }{ $count } commande{ $end } s’exécute après chaque changement.
       *[other] { $val }{ $count } commandes{ $end } s’exécutent après chaque changement.
    }
settings-automation-commands-say = { $listed ->
        [none] { $val }Aucune commande{ $end } ne s’exécute après chaque changement.
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }laisse l’application à vos commandes{ $end }, et les commandes { $val }s’exécutent aussi lors de la restauration au démarrage{ $end }.
           *[skip] Skwd { $val }laisse l’application à vos commandes{ $end }, et les commandes { $val }ignorent la restauration au démarrage{ $end }.
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }applique lui-même le fond d’écran{ $end }, et les commandes { $val }s’exécutent aussi lors de la restauration au démarrage{ $end }.
           *[skip] Skwd { $val }applique lui-même le fond d’écran{ $end }, et les commandes { $val }ignorent la restauration au démarrage{ $end }.
        }
    }
settings-automation-pick-only-hint = vos commandes l’appliquent
settings-automation-commands-heading = Commandes
settings-automation-placeholders = Utilisez %path%, %thumb%, %type% et %name% dans une commande.
settings-automation-minutes = { $count } min
settings-automation-mode-off = Désactivé
settings-automation-mode-outdoors = Suivre le temps dehors
settings-automation-mode-playlists = Choisir dans vos listes
settings-automation-mode-semantic = Chercher dans votre bibliothèque
settings-automation-condition-clear = Dégagé
settings-automation-condition-cloudy = Nuageux
settings-automation-condition-rainy = Pluvieux
settings-automation-condition-snowy = Neigeux
settings-automation-condition-stormy = Orageux
settings-automation-condition-foggy = Brumeux
settings-automation-condition-windy = Venteux
settings-automation-display-another = Un autre
settings-automation-display-exclude = Exclure
settings-automation-display-keep = Garder
settings-automation-playlists-heading = Liste par météo
settings-automation-hours = { $count } h
