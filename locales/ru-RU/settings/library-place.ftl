settings-library-folders-desc = Искать изображения и видео в этих папках и их подпапках.
settings-library-folders-say = Сканируется: { $val }{ $folders }{ $end }.{ $state ->
        [polling] { " " }Опрос ловит то, что пропускает встроенное слежение, каждые { $every }.{ $unread }
        [native] { " " }Встроенное слежение работает; опрос простаивает.
        [down] { " " }Встроенное слежение не работает, опрос выключен.
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }Папок нельзя прочитать: { $count }.
       *[other] { " " }Папок нельзя прочитать: { $count }.
    }
settings-library-watching = Слежение
settings-library-poll-hint = Для сетевых и FUSE-папок
settings-library-interval-hint = После изменения перезапустите skwd-walld
settings-library-excluded-desc = Пропускать эти папки и подпапки. Метаданные сохраняются.
settings-library-excluded-say = Никогда не сканируется: { $val }{ $folders }{ $end }.{ $outside }
settings-library-outside = { $count ->
        [one] { " " }Вне всех папок обоев и ничего не пропускают: { $count }.
       *[other] { " " }Вне всех папок обоев и ничего не пропускают: { $count }.
    }
settings-library-excluded-hint = Исключённые папки не сканируются, и профиль не может их показать. Чтобы оставить папку, но скрыть на людях, откройте { $link }.
settings-library-profiles-desc = Выберите папки для окна выбора, фильтра папок, случайных обоев и расписаний.
settings-library-profiles-say = Активный профиль { $state ->
        [none] { $menu }, показывает все папки.
       *[some] { $menu }: { $summary }.
    } Ctrl+H или кнопка скрытых на панели показывает скрытые обои, в том числе скрытые с карточки, до следующего запуска.
settings-library-active = Активный профиль
settings-library-terminal = Сменить из терминала
settings-library-tree-note = Скрыть: вне случайного выбора до Ctrl+H. Исключить: вне выбора даже с Ctrl+H. Папка задаёт и подпапки.
settings-library-show-tip = В окне выбора и случайных
settings-library-hide-tip = До Ctrl+H, не случайные
settings-library-exclude-tip = Вне этого профиля, Ctrl+H пропускает
settings-library-from = Из раздела { $link }
settings-library-excluded-inside = Внутри исключено: { $count }
settings-library-both-inside = Внутри скрыто: { $hidden }, исключено: { $excluded }
settings-library-storage = Хранилище и корзина
settings-library-storage-desc = Лёгкие копии WebP, хранилище для восстановления и перекрашенные копии.
settings-library-storage-say = { $optimise ->
        [on] Новые изображения становятся WebP, качество { $quality }, не больше { $size }.
       *[off] Новые изображения не меняются. Оптимизация всех: качество { $quality }, не больше { $size }.
    } { $trash ->
        [on] Оригиналы в корзине хранятся { $days }, затем удаляются.
       *[off] Оригиналы в корзине хранятся, пока вы их не удалите.
    } { $recolour ->
        [on] Новые обои также получают перекрашенную копию.
       *[off] Новые обои не перекрашиваются.
    }
settings-library-retention-hint = Сколько дней хранить изображения в корзине
settings-library-optimise-hint = Оптимизировать все изображения библиотеки
settings-library-recolour = Перекраска
settings-library-thumbnails = Миниатюры и ремонт
settings-library-thumbnails-desc = Пересоздайте превью и цвета, если что-то выглядит неверно.
settings-library-thumbnails-say = При пересборке создаётся до { $jobs } миниатюр одновременно.
settings-library-rebuild = Пересборка
settings-library-start-over = С нуля
settings-library-recompute-hint = Из имеющихся миниатюр
settings-library-clear-hint = Миниатюры создадутся при следующем сканировании
