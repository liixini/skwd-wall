settings-sources-other = Другие источники
settings-sources-other-desc = Unsplash, Pexels, Bing: фото дня и YouTube.
settings-sources-wallhaven-say = { $top ->
        [yes] Новые сеансы сортируются по { $sort } за { $range }, соотношение { $ratio }, показ: { $categories }, уровень { $purity }.
       *[no] Новые сеансы сортируются по { $sort }, соотношение { $ratio }, показ: { $categories }, уровень { $purity }.
    }
settings-sources-wallhaven-off = Выкл. Обозреватель Wallhaven скрыт.
settings-sources-workshop-say = { $trend ->
        [yes] Новые сеансы сортируются по { $sort } за { $days }, тип { $type }, разрешение { $resolution }, категория { $category }.
       *[no] Новые сеансы сортируются по { $sort }, тип { $type }, разрешение { $resolution }, категория { $category }.
    }
settings-sources-workshop-off = Выкл. Обозреватель Мастерской Steam и настройки сцен Wallpaper Engine скрыты.
settings-sources-no-category = без категорий
settings-sources-no-content = без уровня содержимого
settings-sources-content = Содержимое
settings-sources-account = Аккаунт
settings-sources-browser = Обозреватель
settings-sources-browser-folders = Обозреватель и папки
settings-sources-photos = Фото
settings-sources-daily = Фото дня и видео
settings-sources-not-set = не задан
settings-sources-detected = Определяется через Steam
settings-sources-nsfw-hint = Нужен для результатов NSFW
settings-sources-grid-hint = Столбцы × строки
settings-sources-thumb-hint = Ширина × высота
settings-sources-steamcmd-hint = Для steamcmd
settings-sources-workshop-only = Только Мастерская
settings-sources-method-steam = Клиент Steam
settings-sources-method-steamcmd = steamcmd
settings-sources-method-steam-hint = Нужны skwd-deck-steamworks и запущенный Steam
settings-sources-method-steamcmd-hint = Нужны steamcmd и однократный вход
settings-sources-unsplash-hint = Бесплатно на unsplash.com/developers
settings-sources-pexels-hint = Бесплатно на pexels.com/api
settings-sources-bing-hint = Последние 8 изображений главной страницы
settings-sources-youtube-hint = Нужен yt-dlp в PATH
settings-sources-clip-hint = 0 скачивает целиком
settings-sources-searching = Поиск: { $sources }.
settings-sources-searching-none = Ни один источник не ищет.
settings-sources-needs-key = { $key ->
        [access] { " " }{ $source }: нужен ключ доступа.
       *[api] { " " }{ $source }: нужен ключ API.
    }
settings-sources-off = { $count ->
        [one] { " " }{ $sources }: выкл.
       *[some] { " " }{ $sources }: выкл.
    }
