settings-library-folders-desc = Analyser ces dossiers et leurs sous-dossiers à la recherche d’images et de vidéos.
settings-library-folders-say = Analyse de { $val }{ $folders }{ $end }.{ $state ->
        [polling] { " " }Le sondage couvre ce que la surveillance native rate, toutes les { $every }.{ $unread }
        [native] { " " }La surveillance native est active ; le sondage est au repos.
        [down] { " " }La surveillance native a échoué et le sondage est désactivé.
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }{ $count } dossier est illisible.
       *[other] { " " }{ $count } dossiers sont illisibles.
    }
settings-library-watching = Surveillance
settings-library-poll-hint = Pour les dossiers réseau ou FUSE
settings-library-interval-hint = Redémarrez skwd-walld après modification
settings-library-excluded-desc = Ignorer ces dossiers et leurs sous-dossiers. Les métadonnées existantes restent enregistrées.
settings-library-excluded-say = Jamais d’analyse pour { $val }{ $folders }{ $end }.{ $outside }
settings-library-outside = { $count ->
        [one] { " " }{ $count } est hors de tout dossier de fonds d’écran et n’ignore rien.
       *[other] { " " }{ $count } sont hors de tout dossier de fonds d’écran et n’ignorent rien.
    }
settings-library-excluded-hint = Les dossiers exclus ne sont jamais analysés, aucun profil ne peut donc les afficher. Pour garder un dossier mais le masquer en public, utilisez { $link }.
settings-library-profiles-desc = Choisissez les dossiers utilisés par le sélecteur, le filtre de dossiers, les fonds aléatoires et les planifications.
settings-library-profiles-say = Profil actif { $state ->
        [none] { $menu }, qui montre tous les dossiers.
       *[some] { $menu } : { $summary }.
    } Ctrl+H ou le bouton masqués de la barre affiche les fonds masqués, y compris ceux masqués depuis leur carte, jusqu’au prochain lancement.
settings-library-active = Profil actif
settings-library-terminal = Changer depuis un terminal
settings-library-tree-note = Masquer reste hors de l’aléatoire jusqu’à Ctrl+H. Exclure reste dehors même avec Ctrl+H. Un dossier règle ses sous-dossiers.
settings-library-show-tip = Dans le sélecteur et l’aléatoire
settings-library-hide-tip = Jusqu’à Ctrl+H, jamais aléatoire
settings-library-exclude-tip = Hors de ce profil, Ctrl+H l’ignore
settings-library-from = Depuis { $link }
settings-library-excluded-inside = { $count } exclus à l’intérieur
settings-library-both-inside = { $hidden } masqués, { $excluded } exclus à l’intérieur
settings-library-storage = Stockage et corbeille
settings-library-storage-desc = Copies WebP plus légères, stockage de récupération et copies recolorées.
settings-library-storage-say = { $optimise ->
        [on] Les nouvelles images passent en WebP en qualité { $quality }, plafonnées à { $size }.
       *[off] Les nouvelles images restent telles quelles. Tout optimiser utilise la qualité { $quality }, plafonnée à { $size }.
    } { $trash ->
        [on] Les originaux mis à la corbeille restent { $days }, puis sont supprimés.
       *[off] Les originaux mis à la corbeille restent jusqu’à leur suppression.
    } { $recolour ->
        [on] Les nouveaux fonds reçoivent aussi une copie recolorée.
       *[off] Les nouveaux fonds ne sont pas recolorés.
    }
settings-library-retention-hint = Jours de conservation des images à la corbeille
settings-library-optimise-hint = Optimiser toutes les images de la bibliothèque
settings-library-recolour = Recoloration
settings-library-thumbnails = Miniatures et réparation
settings-library-thumbnails-desc = Reconstruire les aperçus et les couleurs quand quelque chose semble faux.
settings-library-thumbnails-say = Jusqu’à { $jobs } miniatures sont créées à la fois pendant les reconstructions.
settings-library-rebuild = Reconstruire
settings-library-start-over = Repartir de zéro
settings-library-recompute-hint = À partir des miniatures existantes
settings-library-clear-hint = Les miniatures se recréent à la prochaine analyse
