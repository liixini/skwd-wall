playlists-unnamed = (sem nome)
playlists-generated-name = Playlist { $number }
playlists-new-name = Nome da nova playlist
playlists-name = Nome da playlist
playlists-play-now = Tocar agora
playlists-delete = Excluir
playlists-type-curated = Manual
playlists-type-smart = Filtrada
playlists-order-shuffle = Aleatória
playlists-order-sequential = Sequencial
playlists-every = A cada
playlists-seconds-minimum = segundos (mín. 5)
playlists-all = Todos
playlists-favourites = Favoritos
playlists-filter-example = ex.: type:video color:blue tag:cat,-anime
playlists-empty-smart = Nenhum papel de parede corresponde a este filtro ainda.
playlists-empty-curated = Esta playlist está vazia. Abra o cartão de um papel de parede e escolha Playlist para adicioná-lo.
playlists-stop-all = ■ Parar todas
playlists-card-add-title = Adicionar à playlist
playlists-card-none = Ainda não existem playlists. Dê um nome para criar uma já com este papel de parede:
playlists-card-new = Ou crie outra playlist com este papel de parede:
playlists-masthead = Playlists  /  { $selected }
playlists-masthead-fallback = Biblioteca
playlists-index-title = Playlists
playlists-index-subtitle = Uma playlist manual contém os papéis de parede que você escolhe. Uma playlist filtrada fica sincronizada com as regras dela.
playlists-index-live-detail = { $kind } · ativa em { $outputs }
playlists-index-count-detail =
    { $kind }  ·  { $count } { $count ->
        [one] papel de parede
       *[other] papéis de parede
    }
playlists-index-empty-title = Nenhuma playlist ainda
playlists-index-empty-hint = Crie uma abaixo para começar.
playlists-new-playlist = Nova playlist
playlists-live-routing = Telas ativas
playlists-active-assignments =
    { $count } { $count ->
        [one] atribuição ativa
       *[other] atribuições ativas
    }
playlists-library = Playlists salvas
playlists-library-stats = { $count } salvas · nenhuma ativa
playlists-live-results = Papéis de parede correspondentes
playlists-the-sequence = Ordem de reprodução
playlists-sequence-smart-hint = Papéis de parede que correspondem ao filtro atual, na ordem mostrada.
playlists-sequence-curated-hint = Os papéis de parede tocam de cima para baixo. Arraste para mudar a ordem.
playlists-empty-smart-hint = Nenhum papel de parede corresponde a estas regras. Altere o filtro para incluir mais.
playlists-empty-curated-hint = Abra o cartão de um papel de parede e escolha Playlist para adicionar o primeiro.
playlists-live-badge = Ativa
playlists-empty-eyebrow = Playlists / biblioteca
playlists-empty-title = Monte uma rotação
playlists-empty-choose = Escolha uma playlist no índice
playlists-empty-desc = Ou crie uma nova playlist manual e comece a adicionar papéis de parede.
playlists-kind-smart = Playlist filtrada
playlists-kind-curated = Playlist manual
playlists-state-live = Ativa / { $outputs }
playlists-state-query-ready = Filtro pronto
playlists-state-ready =
    { $padded } { $count ->
        [one] papel de parede pronto
       *[other] papéis de parede prontos
    }
playlists-edition-eyebrow = Playlists / editar
playlists-kind-tagline = { $kind } · pronta para atribuir a telas
playlists-stop-edition = Parar playlist
playlists-colour-notes = Cores
playlists-folder-notes = Pastas
playlists-live-collection-title = Playlist filtrada
playlists-live-collection-desc = Esta lista se atualiza sozinha a partir dos filtros de fonte, tags, resolução e cor selecionados.
playlists-definition-title = Detalhes da playlist
playlists-definition-desc = Defina o nome e escolha se os papéis de parede são adicionados manualmente ou selecionados por um filtro.
playlists-playback-title = Reprodução
playlists-playback-desc = Escolha a ordem e quanto tempo cada papel de parede fica na tela.
playlists-routing-title = Aplicar às telas
playlists-routing-desc = Escolha onde esta playlist roda. Selecionar todas as telas cria uma única atribuição compartilhada.
playlists-routing-all = Todas as telas conectadas
playlists-routing-live = Atribuição ativa
playlists-routing-none = Nenhuma atribuição ativa
playlists-demo-name = O corte mais brabo

playlists-id = ID { $id }
playlists-picker-entry = { $name } · ID { $id }
playlists-choose = Choose a playlist
playlists-browse-all = All wallpapers
playlists-edit = Edit playlists
playlists-picker-search = Search by name or ID
playlists-picker-no-match = No playlists match this search.
playlists-picker-hint = Choose which wallpapers to browse. Playback stays as it is.
playlists-filter-help = Filter reference
playlists-help-combine = Separe os filtros com espaços. Todo filtro precisa corresponder, exceto as regras de pasta: conta um item em qualquer uma das pastas listadas. Coloque entre aspas caminhos com espaços.
playlists-help-source = all / favourites
    Start with all wallpapers or only favourites.
playlists-help-folder = folder:nature/forest
    Corresponde a esta pasta e às subpastas. Várias regras de pasta correspondem a qualquer uma delas; coloque entre aspas caminhos com espaços.
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
