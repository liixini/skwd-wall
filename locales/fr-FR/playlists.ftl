playlists-unnamed = (sans nom)
playlists-generated-name = Liste de lecture { $number }
playlists-new-name = Nom de la nouvelle liste de lecture
playlists-name = Nom de la liste de lecture
playlists-play-now = Lire maintenant
playlists-delete = Supprimer
playlists-type-curated = Manuelle
playlists-type-smart = Filtrée
playlists-order-shuffle = Aléatoire
playlists-order-sequential = Séquentiel
playlists-every = Toutes les
playlists-seconds-minimum = secondes (min. 5)
playlists-all = Tout
playlists-favourites = Favoris
playlists-filter-example = ex. type:video color:blue tag:cat,-anime
playlists-empty-smart = Aucun fond d'écran ne correspond encore à ce filtre.
playlists-empty-curated = Cette liste de lecture est vide. Ouvrez la carte d'un fond d'écran et choisissez Liste de lecture pour l'ajouter.
playlists-stop-all = ■ Tout arrêter
playlists-card-add-title = Ajouter à une liste de lecture
playlists-card-none = Aucune liste de lecture pour l'instant. Nommez-en une pour la créer avec ce fond d'écran déjà ajouté :
playlists-card-new = Ou créez une autre liste de lecture avec ce fond d'écran :
playlists-masthead = Listes de lecture  /  { $selected }
playlists-masthead-fallback = Bibliothèque
playlists-index-title = Listes de lecture
playlists-index-subtitle = Une liste manuelle contient les fonds d'écran que vous choisissez. Une liste filtrée reste synchronisée avec ses règles.
playlists-index-live-detail = { $kind } · active sur { $outputs }
playlists-index-count-detail =
    { $kind }  ·  { $count } { $count ->
        [one] fond d'écran
       *[other] fonds d'écran
    }
playlists-index-empty-title = Aucune liste de lecture pour l'instant
playlists-index-empty-hint = Créez-en une ci-dessous pour commencer.
playlists-new-playlist = Nouvelle liste de lecture
playlists-live-routing = Écrans actifs
playlists-active-assignments =
    { $count } { $count ->
        [one] affectation active
       *[other] affectations actives
    }
playlists-library = Listes de lecture enregistrées
playlists-library-stats = { $count } enregistrées · aucune active
playlists-live-results = Fonds d'écran correspondants
playlists-the-sequence = Ordre de lecture
playlists-sequence-smart-hint = Fonds d'écran correspondant au filtre actuel, dans l'ordre affiché.
playlists-sequence-curated-hint = Les fonds d'écran sont lus de haut en bas. Faites-les glisser pour changer l'ordre.
playlists-empty-smart-hint = Aucun fond d'écran ne correspond à ces règles. Modifiez le filtre pour en inclure davantage.
playlists-empty-curated-hint = Ouvrez la carte d'un fond d'écran et choisissez Liste de lecture pour ajouter le premier.
playlists-live-badge = Active
playlists-empty-eyebrow = Listes de lecture / bibliothèque
playlists-empty-title = Créer une rotation
playlists-empty-choose = Choisissez une liste de lecture dans l'index
playlists-empty-desc = Ou créez une nouvelle liste manuelle et commencez à ajouter des fonds d'écran.
playlists-kind-smart = Liste filtrée
playlists-kind-curated = Liste manuelle
playlists-state-live = Active / { $outputs }
playlists-state-query-ready = Filtre prêt
playlists-state-ready =
    { $padded } { $count ->
        [one] fond d'écran prêt
       *[other] fonds d'écran prêts
    }
playlists-edition-eyebrow = Listes de lecture / édition
playlists-kind-tagline = { $kind } · prête à être affectée aux écrans
playlists-stop-edition = Arrêter la liste de lecture
playlists-colour-notes = Couleurs
playlists-folder-notes = Dossiers
playlists-live-collection-title = Liste filtrée
playlists-live-collection-desc = Cette liste se met à jour d'elle-même selon la source, les tags, la résolution et les filtres de couleur sélectionnés.
playlists-definition-title = Détails de la liste de lecture
playlists-definition-desc = Définissez le nom et choisissez si les fonds d'écran sont ajoutés manuellement ou sélectionnés par un filtre.
playlists-playback-title = Lecture
playlists-playback-desc = Choisissez l'ordre et la durée d'affichage de chaque fond d'écran.
playlists-routing-title = Appliquer aux écrans
playlists-routing-desc = Choisissez où cette liste de lecture s'exécute. Sélectionner tous les écrans crée une affectation partagée.
playlists-routing-all = Tous les écrans connectés
playlists-routing-live = Affectation active
playlists-routing-none = Aucune affectation active
playlists-demo-name = The baddest cut

playlists-id = ID { $id }
playlists-picker-entry = { $name } · ID { $id }
playlists-choose = Choose a playlist
playlists-browse-all = All wallpapers
playlists-edit = Edit playlists
playlists-picker-search = Search by name or ID
playlists-picker-no-match = No playlists match this search.
playlists-picker-hint = Choose which wallpapers to browse. Playback stays as it is.
playlists-filter-help = Filter reference
playlists-help-combine = Séparez les filtres par des espaces. Chaque filtre doit correspondre, sauf les règles de dossier : un élément dans l’un des dossiers listés compte. Mettez entre guillemets les chemins contenant des espaces.
playlists-help-source = all / favourites
    Start with all wallpapers or only favourites.
playlists-help-folder = folder:nature/forest
    Correspond à ce dossier et à ses sous-dossiers. Plusieurs règles de dossier correspondent à l’une ou l’autre ; mettez entre guillemets les chemins contenant des espaces.
playlists-help-type = type:image / type:video / type:we
    Images, videos, or Wallpaper Engine wallpapers. Also accepts static, picture, pic, img; vid, mp4; scene, wallpaperengine, wallpaper-engine.
playlists-help-tag = tag:cat,night / tag:cat|dog,-anime
    Commas require every tag; | accepts either tag; - excludes a tag. Include at least one positive tag. Tags ignore letter case.
playlists-help-colour = color:blue,cyan / colour:gray
    Commas accept any listed colour. Colours: red, orange, yellow, lime, green, spring, cyan, azure, blue, violet, magenta, pink, gray. Numeric buckets 0-11 and 99 also work.
    Aliases: chartreuse; mint, emerald; teal, aqua; sky; purple, indigo; fuchsia; rose; grey, mono, monochrome, grayscale, greyscale, black, white.
playlists-help-ratio = ratio:portrait / ratio:landscape / ratio:square
    Tall, wide, or nearly square (within 0.05 of 1:1). Use decimal ratios for comparisons, such as ratio:>1.7.
playlists-help-width = width:>=1920
    Compare the wallpaper width in pixels.
playlists-help-height = height:>=1080
    Compare the wallpaper height in pixels.
playlists-help-resolution = res:>=1920x1080
    Both dimensions must satisfy the comparison.
playlists-help-comparison = Comparisons: >=, <=, >, <, =
    With no operator, numbers mean at least. Numeric = allows a difference below 0.5, including ratios. Use lowercase filter keywords.
playlists-help-example = Example: favourites type:video ratio:landscape color:blue
    Blue, wide videos from your favourites.
