settings-automation-rotation-desc = Ставит случайные обои из набора каждый раз, когда истекает интервал.
settings-automation-rotation-say = Раз в { $interval }: случайный выбор из { $val }{ $pool }{ $end }.
settings-automation-rotation-off = Выключено. Обои меняются, только когда вы выбираете их сами.
settings-automation-pool = { $count ->
        [none] пустого набора
        [one] { $first }
        [two] { $first } и { $second }
       *[three] { $first }, { $second } и { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] избранных { $pool }
       *[all] { $pool }
    }
settings-automation-pool-images = изображений
settings-automation-pool-videos = видео
settings-automation-pool-engine = обоев Wallpaper Engine
settings-automation-pool-heading = В набор входят
settings-automation-timing = Время
settings-automation-interval-hint = минимум 10 с
settings-automation-schedule-desc = Правила выбирают обои и тему по времени, дню недели, погоде, питанию и экранам. Срабатывает первое подходящее правило.
settings-automation-schedule-say = { $start ->
        [applies] Правила включены; первое подходящее правило { $val }применяется при запуске{ $end }. Обои, выбранные вами, всё равно важнее.
       *[waits] Правила включены; первое подходящее правило { $val }ждёт следующей смены{ $end }. Обои, выбранные вами, всё равно важнее.
    }
settings-automation-schedule-off = Выключено. Ваши правила сохранятся, пока вы это не включите.
settings-automation-start-hint = первое подходящее правило при запуске службы
settings-automation-editor-hint = собирайте правила из блоков условий
settings-automation-location = Место и погода
settings-automation-location-desc = Нужно для времени восхода и заката и для погодных условий.
settings-automation-location-say = Погода для { $city }, время солнца по { $lat }, { $lon }. Смена по погоде: { $mode }.
settings-automation-no-city = город не задан
settings-automation-no-latitude = широта не задана
settings-automation-no-longitude = долгота не задана
settings-automation-location-hint = Оставьте город пустым, чтобы использовать координаты. Широта положительна к северу, долгота к востоку.
settings-automation-startup = Запуск и уведомления
settings-automation-startup-desc = Что происходит при запуске службы и при смене обоев.
settings-automation-startup-say = { $restore ->
        [restores] При запуске Skwd { $val }снова ставит последние обои{ $end }.
       *[leaves] При запуске Skwd { $val }оставляет рабочий стол как есть{ $end }.
    } { $notify ->
        [notifies] При смене обоев { $val }приходит уведомление{ $end }.
       *[silent] Смена обоев { $val }проходит тихо{ $end }.
    }
settings-automation-commands = После применения обоев
settings-automation-commands-desc = Запускайте свои команды после каждой смены, например чтобы управлять другой программой для обоев.
settings-automation-command-count = { $count ->
        [one] { $val }{ $count } команда{ $end } выполняется после каждой смены.
        [few] { $val }{ $count } команды{ $end } выполняются после каждой смены.
        [many] { $val }{ $count } команд{ $end } выполняются после каждой смены.
       *[other] { $val }{ $count } команды{ $end } выполняются после каждой смены.
    }
settings-automation-commands-say = { $listed ->
        [none] { $val }Никакие команды{ $end } не выполняются после смены.
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }оставляет применение вашим командам{ $end }, а команды { $val }выполняются и при восстановлении на запуске{ $end }.
           *[skip] Skwd { $val }оставляет применение вашим командам{ $end }, а команды { $val }пропускают восстановление на запуске{ $end }.
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }сам применяет обои{ $end }, а команды { $val }выполняются и при восстановлении на запуске{ $end }.
           *[skip] Skwd { $val }сам применяет обои{ $end }, а команды { $val }пропускают восстановление на запуске{ $end }.
        }
    }
settings-automation-pick-only-hint = обои применяют ваши команды
settings-automation-commands-heading = Команды
settings-automation-placeholders = Используйте в команде %path%, %thumb%, %type% и %name%.
settings-automation-minutes = { $count } мин
settings-automation-mode-off = Выключено
settings-automation-mode-outdoors = Следовать за погодой на улице
settings-automation-mode-playlists = Брать из плейлистов
settings-automation-mode-semantic = Искать в библиотеке
settings-automation-condition-clear = Ясно
settings-automation-condition-cloudy = Облачно
settings-automation-condition-rainy = Дождь
settings-automation-condition-snowy = Снег
settings-automation-condition-stormy = Гроза
settings-automation-condition-foggy = Туман
settings-automation-condition-windy = Ветер
settings-automation-display-another = Ещё
settings-automation-display-exclude = Исключить
settings-automation-display-keep = Оставить
settings-automation-playlists-heading = Плейлист для погоды
settings-automation-hours = { $count } ч
