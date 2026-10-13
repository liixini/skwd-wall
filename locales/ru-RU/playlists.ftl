playlists-unnamed = (без названия)
playlists-generated-name = Плейлист { $number }
playlists-new-name = Название нового плейлиста
playlists-name = Название плейлиста
playlists-play-now = Воспроизвести
playlists-delete = Удалить
playlists-type-curated = Ручной
playlists-type-smart = По фильтру
playlists-order-shuffle = Случайно
playlists-order-sequential = По порядку
playlists-every = Каждые
playlists-seconds-minimum = секунд (мин. 5)
playlists-all = Все
playlists-favourites = Избранное
playlists-filter-example = например type:video color:blue tag:cat,-anime
playlists-empty-smart = Этому фильтру пока не соответствуют никакие обои.
playlists-empty-curated = Плейлист пуст. Откройте карточку обоев и выберите «Плейлист», чтобы добавить их.
playlists-stop-all = ■ Остановить все
playlists-card-add-title = Добавить в плейлист
playlists-card-none = Плейлистов пока нет. Введите название, чтобы создать плейлист с этими обоями:
playlists-card-new = Или создайте ещё один плейлист с этими обоями:
playlists-masthead = Плейлисты  /  { $selected }
playlists-masthead-fallback = Библиотека
playlists-index-title = Плейлисты
playlists-index-subtitle = Ручной плейлист содержит выбранные вами обои. Плейлист по фильтру синхронизируется со своими правилами.
playlists-index-live-detail = { $kind } · активен на { $outputs }
playlists-index-count-detail =
    { $kind }  ·  { $count } { $count ->
        [one] фон
        [few] фона
        [many] фонов
       *[other] фона
    }
playlists-index-empty-title = Плейлистов пока нет
playlists-index-empty-hint = Создайте первый ниже.
playlists-new-playlist = Новый плейлист
playlists-live-routing = Активные дисплеи
playlists-active-assignments =
    { $count } { $count ->
        [one] активное назначение
        [few] активных назначения
        [many] активных назначений
       *[other] активного назначения
    }
playlists-library = Сохранённые плейлисты
playlists-library-stats = Сохранено: { $count } · нет активных
playlists-live-results = Подходящие обои
playlists-the-sequence = Порядок воспроизведения
playlists-sequence-smart-hint = Обои, соответствующие текущему фильтру, в показанном порядке.
playlists-sequence-curated-hint = Обои воспроизводятся сверху вниз. Перетащите их, чтобы изменить порядок.
playlists-empty-smart-hint = Этим правилам не соответствуют никакие обои. Измените фильтр, чтобы охватить больше.
playlists-empty-curated-hint = Откройте карточку обоев и выберите «Плейлист», чтобы добавить первые обои.
playlists-live-badge = Активен
playlists-empty-eyebrow = Плейлисты / библиотека
playlists-empty-title = Соберите ротацию
playlists-empty-choose = Выберите плейлист в списке
playlists-empty-desc = Или создайте новый ручной плейлист и начните добавлять обои.
playlists-kind-smart = Плейлист по фильтру
playlists-kind-curated = Ручной плейлист
playlists-state-live = Активен / { $outputs }
playlists-state-query-ready = Фильтр готов
playlists-state-ready =
    { $padded } { $count ->
        [one] фон готов
        [few] фона готовы
        [many] фонов готовы
       *[other] фона готовы
    }
playlists-edition-eyebrow = Плейлисты / редактирование
playlists-kind-tagline = { $kind } · можно назначить дисплеям
playlists-stop-edition = Остановить плейлист
playlists-colour-notes = Цвета
playlists-folder-notes = Папки
playlists-live-collection-title = Плейлист по фильтру
playlists-live-collection-desc = Этот список обновляется сам по выбранным фильтрам источника, тегов, разрешения и цвета.
playlists-definition-title = Параметры плейлиста
playlists-definition-desc = Задайте название и выберите, добавляются ли обои вручную или отбираются фильтром.
playlists-playback-title = Воспроизведение
playlists-playback-desc = Выберите порядок и время показа каждых обоев.
playlists-routing-title = Применить к дисплеям
playlists-routing-desc = Выберите, где работает этот плейлист. Если выбрать все дисплеи, создаётся одно общее назначение.
playlists-routing-all = Все подключённые дисплеи
playlists-routing-live = Активное назначение
playlists-routing-none = Нет активного назначения
playlists-demo-name = Самая дерзкая подборка

playlists-id = ID { $id }
playlists-picker-entry = { $name } · ID { $id }
playlists-choose = Choose a playlist
playlists-browse-all = All wallpapers
playlists-edit = Edit playlists
playlists-picker-search = Search by name or ID
playlists-picker-no-match = No playlists match this search.
playlists-picker-hint = Choose which wallpapers to browse. Playback stays as it is.
playlists-filter-help = Filter reference
playlists-help-combine = Разделяйте фильтры пробелами. Должен совпасть каждый фильтр, кроме правил папок: подходит элемент из любой указанной папки. Заключайте пути с пробелами в кавычки.
playlists-help-source = all / favourites
    Start with all wallpapers or only favourites.
playlists-help-folder = folder:nature/forest
    Совпадает с этой папкой и её подпапками. Несколько правил папок совпадают с любой из них; заключайте пути с пробелами в кавычки.
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
