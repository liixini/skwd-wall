settings-wallpapers-displays-desc = Что на каждом дисплее и как размещено.
settings-wallpapers-play = Как воспроизводятся обои
settings-wallpapers-play-desc = Движки, их слой и сцены.
settings-wallpapers-transitions-desc = Как обои сменяют друг друга.
settings-wallpapers-awww-desc = Переход awww для изображений.
settings-wallpapers-drawing = Рисование
settings-wallpapers-drawing-desc = Готовые обои рисуют себя сами.
settings-wallpapers-sound = Звук
settings-wallpapers-sound-desc = Громкость и когда звук уступает.
settings-wallpapers-pause-desc = Пауза анимации по правилу.
settings-wallpapers-plasma = Экран блокировки Plasma
settings-wallpapers-plasma-desc = Обои Plasma при блокировке.
settings-wallpapers-niri = Обзор Niri
settings-wallpapers-niri-desc = На niri. Фон за обзором.
settings-wallpapers-display-more = Ещё
settings-wallpapers-lock = Заблокировать { $name }
settings-wallpapers-lock-hint = Обновлять только через мультивыбор
settings-wallpapers-layer-bottom = Снизу
settings-wallpapers-gpu-hint = Откат на «Автоматически»
settings-wallpapers-timeout-hint = Для очень больших изображений
settings-wallpapers-animated = Анимированные обои
settings-wallpapers-scenes = Сцены
settings-wallpapers-scenes-hint = Предел FPS · масштаб
settings-wallpapers-particles-hint = Только сцены
settings-wallpapers-preview-hint = Частота кадров
settings-wallpapers-quality-short = Песку нужно не менее { $ms } мс.
settings-wallpapers-quality-hint = Full без предела, Low до 30 fps
settings-wallpapers-sharpen-hint = Пока качество снижает разрешение
settings-wallpapers-monitors = Мониторы
settings-wallpapers-primary-hint = Анимация только для основного
settings-wallpapers-primary-blank = первый доступный
settings-wallpapers-primary-only-hint = Остальные используют все мониторы
settings-wallpapers-wave = Волна
settings-wallpapers-wave-hint = Ширина × высота
settings-wallpapers-position-hint = center, top-left или 0.5,0.5
settings-wallpapers-bezier-hint = x1,y1,x2,y2
settings-wallpapers-bezier-blank = по умолчанию awww
settings-wallpapers-invert-hint = Отразить ось Y от начала
settings-wallpapers-colours-files = Цвета и файлы
settings-wallpapers-preparing = Подготовка
settings-wallpapers-paper-ink = Бумага и чернила
settings-wallpapers-prepare = Подготовить
settings-wallpapers-prepare-all = Все обои
settings-wallpapers-scope-view = Текущий вид · { $count }
settings-wallpapers-pack-ready = Файлы обработки готовы.
settings-wallpapers-pack-none = Файлов обработки пока нет.
settings-wallpapers-pack-blank = Пока нет
settings-wallpapers-drawing-blocked = Рисование недоступно. Сначала выберите файлы обработки.
settings-wallpapers-volume = Громкость
settings-wallpapers-other-apps = Другие приложения
settings-wallpapers-pause-apps = Пауза, пока работают приложения из списка.
settings-wallpapers-pause-none = Правила паузы выключены.
settings-wallpapers-process-hint = Можно * и ?
settings-wallpapers-process-action = Выбрать
settings-wallpapers-process-blank = пока нет
settings-wallpapers-resume-hint = После отмены последнего правила
settings-wallpapers-scope-all = всех дисплеях
settings-wallpapers-scope-display = дисплее окна
settings-wallpapers-rule-fullscreen = полноэкранных окнах
settings-wallpapers-rule-maximised = развёрнутых окнах
settings-wallpapers-rule-focused = окнах в фокусе
settings-wallpapers-rule-full-width = колонках на всю ширину
settings-wallpapers-plasma-left = управляется Plasma
settings-wallpapers-plasma-static = показывает статичное изображение
settings-wallpapers-plasma-follow = следует за последними обоями
settings-wallpapers-plasma-off-row = Без управления: Plasma не трогаем.
settings-wallpapers-image-hint = Абсолютный путь или путь с ~/
settings-wallpapers-image-blank = изображение не задано
settings-wallpapers-dynamic-hint = Постер это одно беззвучное изображение
settings-wallpapers-niri-off = Выкл. Обзор сохраняет собственный фон niri.
settings-wallpapers-source = Источник
settings-wallpapers-look = Вид
settings-wallpapers-source-blank = последние обои
settings-wallpapers-blur = Размыть фон
settings-wallpapers-radii = Радиус размытия
settings-wallpapers-radii-hint = Изображения · видео · сцены
settings-wallpapers-dim-hint = 0 нет, 100 чёрный
settings-wallpapers-transitions-off = Выкл. Обои сменяются сразу, без перехода.
settings-wallpapers-transitions-off-foot = Включите переходы, чтобы выбрать предпросмотр, мониторы и качество песка.
settings-wallpapers-awww-none = Изображения сменяются так: { $type }.
settings-wallpapers-awww-no-type = без перехода
settings-wallpapers-sound-muted = Звук обоев выключен.
settings-wallpapers-displays-say = { $state ->
        [none] Заблокированных дисплеев нет.
        [one] { $names } заблокирован.
        [some] { $names } заблокированы.
       *[detecting] Поиск дисплеев.
    } Дисплеи без своего размещения используют { $fill }.
settings-wallpapers-play-say = Изображения: { $backend }, видео: { $val }Vulkan{ $end }, на слое { $layer }.
settings-wallpapers-transitions-say = Семейство { $family }{ $kind ->
        [random] , каждый раз другой, за { $duration }.
        [preset] , эффект { $shader }, за { $ms } мс.
       *[shader] , эффект { $shader }, за { $duration }.
    } Частота { $rate }.
settings-wallpapers-primary-count = { $state ->
        [none] ни один из { $all }
        [all] все { $all }
       *[some] { $shown } из { $all }
    }
settings-wallpapers-awww-say = Изображения: { $type }, фильтр { $filter }, за { $duration }.
settings-wallpapers-drawing-say = { $state ->
        [on] Подготовленные обои рисуются за { $duration }.
       *[off] Выкл. Карточки с рисованием рисуются за { $duration }.
    }
settings-wallpapers-sound-say = Звук обоев на { $volume }{ $duck ->
        [yes] { " " }и пауза, когда звучат другие приложения.
       *[no] .
    }
settings-wallpapers-pause-say = Пауза при { $rules } на { $scope }{ $apps ->
        [yes]  и для приложений из списка.
       *[no] .
    }
settings-wallpapers-pause-rules = правилах окон ({ $count })
settings-wallpapers-plasma-say = { $state ->
        [off] Экран блокировки { $mode }.
        [static] Экран блокировки { $mode }: { $image }.
       *[follow] Экран блокировки { $mode }, сцены как { $scenes ->
        [live] живой рендер.
       *[poster] постер.
    }
    }
settings-wallpapers-niri-say = Показывает { $source ->
       *[current] текущие обои
        [own] свой источник
        [stationary] одни постоянные обои
    }{ $blur ->
        [yes] , с размытием
       *[no] { "" }
    }, затемнение { $dim }{ $auto ->
        [yes] , перекрашено
       *[no] { "" }
    }.
settings-wallpapers-device-hint = «Автоматически»: CUDA, WebGPU, затем CPU
settings-wallpapers-memory-hint = При превышении «Автоматически» перейдёт на CPU
