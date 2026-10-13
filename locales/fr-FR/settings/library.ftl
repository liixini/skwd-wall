settings-library-watch-section-desc = Détecte les fichiers ajoutés hors de skwd-wall et garde la bibliothèque à jour.
settings-library-watch-fallback-label = Repli par sondage
settings-library-watch-fallback-desc = Vérifie uniquement les dossiers de la bibliothèque que la surveillance native des fichiers ne peut pas suivre. Activez ceci pour les montages réseau ou FUSE qui manquent des changements, puis redémarrez skwd-walld.
settings-library-watch-interval-label = Intervalle de sondage
settings-library-watch-interval-desc = Attend ce nombre de secondes entre deux vérifications bornées. Des valeurs plus basses trouvent les changements plus tôt mais lisent le système de fichiers plus souvent. Redémarrez skwd-walld après modification.
settings-library-watch-unknown-label = État de la surveillance indisponible
settings-library-watch-unknown-desc = Ce daemon ne signale pas l’état de surveillance de la bibliothèque. Mettez à jour ou redémarrez skwd-walld.
settings-library-watch-poll-failed-label = Le sondage ne peut pas lire un dossier de la bibliothèque
settings-library-watch-poll-failed-desc = Vérifiez que chaque dossier de bibliothèque configuré est monté et lisible. Le sondage réessaiera dans { $interval } secondes.
settings-library-watch-polling-label = Repli par sondage actif
settings-library-watch-polling-desc = La surveillance native a échoué pour { $count ->
    [one] { $count } dossier de la bibliothèque
   *[other] { $count } dossiers de la bibliothèque
    }. Jusqu’à { $budget } entrées sont vérifiées toutes les { $interval } secondes. Dernière convergence réussie : { $convergence }.
settings-library-watch-recovering-label = Surveillance native rétablie
settings-library-watch-recovering-desc = Le surveillant natif est de nouveau actif. Une analyse complète de transfert est encore en cours avant que la bibliothèque soit déclarée à jour.
settings-library-watch-unavailable-label = Surveillance de la bibliothèque indisponible
settings-library-watch-unavailable-desc = La surveillance native des fichiers a échoué et le repli par sondage est désactivé. Activez Repli par sondage, puis redémarrez skwd-walld.
settings-library-watch-recovered-label = Surveillance native restaurée
settings-library-watch-recovered-desc = Le surveillant natif et son analyse de transfert sont à jour. Dernière convergence réussie : { $convergence }.
settings-library-watch-native-label = Surveillance native des fichiers
settings-library-watch-native-desc = Les événements du système de fichiers sont actifs pour chaque dossier de la bibliothèque. Le sondage est inactif.
settings-library-watch-convergence-never = Pas encore terminée
settings-library-watch-convergence-seconds = { $value ->
    [one] il y a { $value } seconde
   *[other] il y a { $value } secondes
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] il y a { $value } minute
   *[other] il y a { $value } minutes
    }
settings-library-watch-convergence-hours = { $value ->
    [one] il y a { $value } heure
   *[other] il y a { $value } heures
    }
settings-profiles-card = Profils
settings-profiles-card-desc = Choisissez les dossiers utilisés par le sélecteur, le filtre de dossiers, les fonds aléatoires et les planifications. Les dossiers masqués reviennent avec le bouton des éléments masqués de la barre ou son raccourci clavier, et chaque lancement les masque de nouveau.
settings-profiles-everything = Tout
settings-profiles-name-label = Nom
settings-profiles-name-placeholder = Nom du profil
settings-profiles-state-show = Afficher
settings-profiles-state-hide = Masquer
settings-profiles-state-exclude = Exclure
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = Les éléments du Workshop n’ont pas de dossier, ce réglage s’applique donc à tous. Masquez un fond précis depuis le dos de sa carte.
settings-profiles-summary = { $hidden } masqués, { $excluded } exclus
settings-profiles-summary-none = Affiche tous les dossiers
settings-profiles-delete-action = Supprimer le profil
settings-profiles-add-label = Nouveau profil
settings-profiles-new-name = Nouveau profil
settings-profiles-tree-label = Dossiers de ce profil
settings-profiles-tree-desc = Afficher garde le dossier dans le sélecteur et l’aléatoire. Masquer le garde hors de vue jusqu’au bouton des éléments masqués de la barre ou son raccourci. Exclure le retire du profil, le bouton l’ignore donc. Un dossier règle aussi ses sous-dossiers ; modifiez un sous-dossier ensuite pour faire une exception.
settings-profiles-never-scanned = Jamais analysé
settings-profiles-mixed = Mélangé à l’intérieur
settings-profiles-hidden-inside = { $count ->
    [one] { $count } masqué à l’intérieur
   *[other] { $count } masqués à l’intérieur
    }
settings-profiles-copy-action = Copier
settings-profiles-helm-desc = En fait le profil actif, la rotation et les planifications le suivent donc.
settings-profiles-flag-desc = Ouvre un sélecteur dans ce profil et laisse le profil actif tel quel.
settings-profiles-make-active = Activer
settings-profiles-active-tag = Actif
settings-profiles-builtin-tag = Intégré
settings-profiles-filter-placeholder = Filtrer les dossiers
settings-profiles-no-match = Aucun dossier ne correspond.
settings-profiles-switch-label = Passer à ce profil
settings-profiles-switch-desc = skwd-helm change le profil enregistré, donc la rotation et les planifications suivent. L’option de lancement ouvre un sélecteur dans ce profil sans toucher au profil enregistré.
settings-profiles-hidden-note-label = Fonds masqués
settings-profiles-hidden-note-desc = Le bouton de masquage de la barre (Ctrl+H) affiche les dossiers masqués du profil actif et chaque fond masqué depuis sa carte. Chaque lancement les masque à nouveau.
settings-profiles-tree-note = Jamais analysé vient des dossiers exclus dans Bibliothèque › Dossiers, aucun profil ne peut donc l’afficher. Exclure ici retire seulement un dossier de ce profil. Les fonds masqués depuis leur carte restent masqués dans chaque profil, Tout compris, jusqu’à Afficher les masqués. Les dossiers à point comme .private sont des dossiers ordinaires ici.
settings-profiles-legend-show = dans le sélecteur et l’aléatoire.
settings-profiles-legend-hide = jusqu’au bouton de masquage de la barre ou Ctrl+H. Jamais en aléatoire.
settings-profiles-legend-exclude = de ce profil. Afficher les masqués l’ignore.
settings-profiles-everything-desc = Tout affiche chaque dossier analysé et chaque élément Workshop. Créez un nouveau profil pour en masquer ou en exclure certains.
settings-profiles-delete-hint = Cliquez encore pour supprimer. Les fonds et les dossiers restent en place.
