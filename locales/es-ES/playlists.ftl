playlists-unnamed = (sin nombre)
playlists-generated-name = Lista { $number }
playlists-new-name = Nombre de la nueva lista
playlists-name = Nombre de la lista
playlists-play-now = Reproducir ahora
playlists-delete = Eliminar
playlists-type-curated = Manual
playlists-type-smart = Filtrada
playlists-order-shuffle = Aleatorio
playlists-order-sequential = Secuencial
playlists-every = Cada
playlists-seconds-minimum = segundos (mín. 5)
playlists-all = Todo
playlists-favourites = Favoritos
playlists-filter-example = ej. type:video color:blue tag:cat,-anime
playlists-empty-smart = Ningún fondo coincide con este filtro todavía.
playlists-empty-curated = Esta lista está vacía. Abre la tarjeta de un fondo y elige Lista para añadirlo.
playlists-stop-all = ■ Detener todo
playlists-card-add-title = Añadir a la lista
playlists-card-none = Aún no hay listas. Ponle un nombre para crearla con este fondo ya añadido:
playlists-card-new = O crea otra lista con este fondo:
playlists-masthead = Listas  /  { $selected }
playlists-masthead-fallback = Biblioteca
playlists-index-title = Listas
playlists-index-subtitle = Una lista manual contiene los fondos que elijas. Una lista filtrada se mantiene sincronizada con sus reglas.
playlists-index-live-detail = { $kind } · activa en { $outputs }
playlists-index-count-detail =
    { $kind }  ·  { $count } { $count ->
        [one] fondo
       *[other] fondos
    }
playlists-index-empty-title = Aún no hay listas
playlists-index-empty-hint = Crea una abajo para empezar.
playlists-new-playlist = Nueva lista
playlists-live-routing = Pantallas activas
playlists-active-assignments =
    { $count } { $count ->
        [one] asignación activa
       *[other] asignaciones activas
    }
playlists-library = Listas guardadas
playlists-library-stats = { $count } guardadas · ninguna activa
playlists-live-results = Fondos coincidentes
playlists-the-sequence = Orden de reproducción
playlists-sequence-smart-hint = Fondos que coinciden con el filtro actual, en el orden mostrado.
playlists-sequence-curated-hint = Los fondos se reproducen de arriba abajo. Arrástralos para cambiar el orden.
playlists-empty-smart-hint = Ningún fondo coincide con estas reglas. Cambia el filtro para incluir más.
playlists-empty-curated-hint = Abre la tarjeta de un fondo y elige Lista para añadir el primero.
playlists-live-badge = Activa
playlists-empty-eyebrow = Listas / biblioteca
playlists-empty-title = Crea una rotación
playlists-empty-choose = Elige una lista del índice
playlists-empty-desc = O crea una nueva lista manual y empieza a añadir fondos.
playlists-kind-smart = Lista filtrada
playlists-kind-curated = Lista manual
playlists-state-live = Activa / { $outputs }
playlists-state-query-ready = Filtro listo
playlists-state-ready =
    { $padded } { $count ->
        [one] fondo
       *[other] fondos
    } listos
playlists-edition-eyebrow = Listas / editar
playlists-kind-tagline = { $kind } · lista para asignar a pantallas
playlists-stop-edition = Detener lista
playlists-colour-notes = Colores
playlists-folder-notes = Carpetas
playlists-live-collection-title = Lista filtrada
playlists-live-collection-desc = Esta lista se actualiza sola según la fuente, etiquetas, resolución y filtros de color seleccionados.
playlists-definition-title = Detalles de la lista
playlists-definition-desc = Ponle un nombre y elige si los fondos se añaden manualmente o mediante un filtro.
playlists-playback-title = Reproducción
playlists-playback-desc = Elige el orden y cuánto tiempo permanece cada fondo en pantalla.
playlists-routing-title = Aplicar a pantallas
playlists-routing-desc = Elige dónde se ejecuta esta lista. Seleccionar todas las pantallas crea una asignación compartida.
playlists-routing-all = Todas las pantallas conectadas
playlists-routing-live = Asignación activa
playlists-routing-none = Sin asignación activa
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
playlists-help-combine = Separa los filtros con espacios. Cada filtro debe coincidir, salvo las reglas de carpeta: cuenta un elemento de cualquiera de las carpetas indicadas. Pon entre comillas las rutas con espacios.
playlists-help-source = all / favourites
    Start with all wallpapers or only favourites.
playlists-help-folder = folder:nature/forest
    Coincide con esta carpeta y sus subcarpetas. Varias reglas de carpeta coinciden con cualquiera de ellas; pon entre comillas las rutas con espacios.
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
