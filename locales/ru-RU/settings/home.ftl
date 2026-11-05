settings-home-search = Поиск по всем настройкам: частота кадров, звук, экран блокировки, размытие
settings-home-displays = { $count ->
    [one] { $count } дисплей
    [few] { $count } дисплея
    [many] { $count } дисплеев
   *[other] { $count } дисплея
 }
settings-home-displays-say = { $link } { $count ->
    [one] показывает обои.
   *[other] показывают обои.
 }
settings-home-watching = { " " }Отслеживание библиотеки: { $val }{ $state }{ $end }.
settings-home-model-ready = готова
settings-home-model-missing = нет модели
settings-home-model-off = выключена
settings-home-colours-from = { " " }{ $link }: источник { $val }{ $engine }{ $end }.
settings-home-sound-desc = Звук видеообоев и обоев Wallpaper Engine.
settings-home-sound-say = Обои звучат на { $volume }{ $duck ->
    [yes] { " " }и ставятся на паузу, когда звук воспроизводят другие приложения.
   *[no] , даже когда звук воспроизводят другие приложения.
 }
settings-home-colours-desc = Как цвета обоев попадают на остальной рабочий стол.
settings-home-colours-say = { settings-theme-policy-label }: { $policy }.{ " " }Подробнее: { $place }.
settings-home-rotation-desc = Ставить случайные обои по истечении интервала.
settings-home-rotation-say = Случайные обои каждые { $interval }.
settings-home-rotation-off = Смена выключена. Обои меняются, только когда вы их выбираете.
settings-home-transitions-desc = Как одни обои сменяются следующими.
settings-home-transitions-say = Одни обои сменяются следующими за { $duration }.
settings-home-transitions-off = Переходы выключены. Обои сменяются сразу.
settings-home-changed = Отличия от значений по умолчанию
settings-home-changed-desc = Настройки, которые вы изменили, в одном месте, у каждой есть «Сбросить».
settings-home-changed-none = Среди сравниваемых здесь настроек ничего не отличается от значений по умолчанию.
settings-home-changed-say = { $count ->
    [one] { $count } настройка отличается от значения по умолчанию.
    [few] { $count } настройки отличаются от значений по умолчанию.
    [many] { $count } настроек отличаются от значений по умолчанию.
   *[other] { $count } настройки отличаются от значений по умолчанию.
 }
settings-home-review = Просмотреть
settings-home-reset = Сбросить
settings-home-reset-all = Сбросить всё
settings-home-badge = изменено: { $count }
settings-home-automatic = Автоматически
settings-home-displays-none = Ни один дисплей
settings-home-displays-none-say = { $link } не показывает обои.
settings-home-reset-scope = Сбрасывает все настройки выше, включая сочетания клавиш. Ключи API, имена пользователей, местоположения и папки сохраняются.
settings-home-items = { $count ->
    [one] { $count } элемент
    [few] { $count } элемента
    [many] { $count } элементов
   *[other] { $count } элемента
 }
