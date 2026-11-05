settings-colours-source = Откуда берётся цвет
settings-colours-source-desc = Следовать за каждыми обоями, оставить одну палитру или не менять цвета.
settings-colours-source-follow = следуют за каждыми обоями
settings-colours-source-fixed = берутся из одной палитры
settings-colours-source-off = остаются как есть
settings-colours-source-say = Цвета { $policy }.
settings-colours-source-made = Цвета { $policy }, создаёт { $authority }.
settings-colours-end4 = end-4
settings-colours-control-menu = Управление цветом · найденные оболочки
settings-colours-generator-menu = Генератор · только установленные
settings-colours-theme-menu = Тема · готовые, ваши, свои цвета
settings-colours-all-presets = Все готовые ({ $count })…
settings-colours-generator = Генератор
settings-colours-generator-desc = Программа, которая превращает обои в палитру.
settings-colours-generator-say = { $engine } создаёт палитру.
settings-colours-generator-seeded = { $engine } создаёт палитру из ячейки { $index } с контрастом { $contrast }.
settings-colours-generator-shell = { $shell } создаёт палитру из обоев.
settings-colours-palette = Палитра
settings-colours-palette-desc = Одна палитра для всех обоев.
settings-colours-palette-say = { $theme } для всех обоев.
settings-colours-idle = Не используется, пока цвета остаются как есть.
settings-colours-index-hint = Какая ячейка палитры станет исходным цветом
settings-colours-contrast-hint = от -1 до 1
settings-colours-wallust-hint = Задать палитру wallust, напр. dark16. Пусто: по режиму
settings-colours-seeds-hint = Для пункта «Свои цвета»
settings-colours-config-hint = Внешний Matugen
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = Авто
settings-colours-look = Вид
settings-colours-look-desc = Насколько сильно расходится цвет, отделка, светлый или тёмный.
settings-colours-look-say = Характер { $scheme }, отделка { $finish }, вариант { $variant }.
settings-colours-look-imported = Палитра { $shell }, импортирована как вариант { $variant }.
settings-colours-look-shell = { $shell } сам задаёт вид.
settings-colours-look-fixed = Фиксированная палитра сама задаёт вид.
settings-colours-no-designer = Дизайнер тем доступен с режимами «Следовать за обоями» и «Фиксированная палитра».
settings-colours-designer-hint = Настроить или собрать палитру вручную
settings-colours-go = Куда идут цвета
settings-colours-go-desc = Темы приложений, которые следуют за палитрой.
settings-colours-find-hint = Проверить снова после установки приложения
settings-colours-matugen-hint = Переключатель выводов Matugen
settings-colours-reload-hint = Запускается после записи файла
settings-colours-output-heading = Вывод { $count }
settings-colours-shells = Оболочки
settings-colours-shells-desc = Noctalia и Dank Material Shell могут брать цвета Skwd.
settings-colours-dms-hint = DMS должен использовать Auto (обои)
settings-colours-mode-hint = Применяется с каждой цветовой схемой
settings-colours-bin-hint = Если его нет в PATH
settings-colours-lock = Экраны блокировки
settings-colours-lock-desc = Экраны блокировки с вашими обоями.
settings-colours-lock-follow = последние применённые обои
settings-colours-lock-still = статичное изображение
settings-colours-sddm-hint = Нужно подтверждение администратора
settings-colours-go-say = { $val }{ $on } из { $all } приложений{ $end } { $followers ->
        [one] следует за
       *[some] следуют за
    } вашими цветами{ $changed ->
        [one] ; { $val }конфиг { $name } изменён вне Skwd{ $end }.
        [some] ; { $val }изменено вне Skwd: { $count }{ $end }.
       *[none] .
    }
settings-colours-outputs-say = { $state ->
        [none] Пока ни одна интеграция не рендерит шаблон.
        [single] { $val }1 интеграция{ $end } рендерит шаблон в файл после смены цвета.
       *[many] { $val }Интеграций: { $count }{ $end }, рендерят шаблоны в файлы после смены цвета.
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] следует за цветами Skwd
       *[keeps] сохраняет свои цвета
    }{ $end }; DMS { $val }{ $dms ->
        [follows] следует за цветами Skwd
       *[keeps] сохраняет свои цвета
    }{ $end }{ $caelestia ->
        [yes] ; обои Caelestia синхронизируются.
       *[no] .
    }
settings-colours-lock-say = { $state ->
        [none] Экраны блокировки не настроены. Новые используют { $mode }.
        [one] { $names } использует { $mode }.
       *[some] { $names } используют { $mode }.
    }
settings-colours-lock-say-static = { $state ->
        [none] Экраны блокировки не настроены. Новые используют { $mode }: { $image }.
        [one] { $names } использует { $mode }: { $image }.
       *[some] { $names } используют { $mode }: { $image }.
    }
settings-menu-more = Ещё
