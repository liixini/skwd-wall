settings-library-watch-section-desc = Обнаруживать файлы, добавленные вне skwd-wall, и поддерживать библиотеку в актуальном состоянии.
settings-library-watch-fallback-label = Резервный опрос
settings-library-watch-fallback-desc = Проверять только те папки библиотеки, за которыми не может следить встроенное отслеживание файлов. Включите для сетевых или FUSE-монтирований, которые пропускают изменения, затем перезапустите skwd-walld.
settings-library-watch-interval-label = Интервал опроса
settings-library-watch-interval-desc = Столько секунд ждать между ограниченными проверками. Меньшие значения быстрее находят изменения, но чаще читают файловую систему. После изменения перезапустите skwd-walld.
settings-library-watch-unknown-label = Состояние отслеживания недоступно
settings-library-watch-unknown-desc = Эта служба не сообщает о состоянии отслеживания библиотеки. Обновите или перезапустите skwd-walld.
settings-library-watch-poll-failed-label = Опрос не может прочитать папку библиотеки
settings-library-watch-poll-failed-desc = Проверьте, что все настроенные папки библиотеки смонтированы и доступны для чтения. Повторный опрос через { $interval } с.
settings-library-watch-polling-label = Резервный опрос активен
settings-library-watch-polling-desc = Встроенное отслеживание не работает для { $count ->
    [one] { $count } папки библиотеки
    [few] { $count } папок библиотеки
    [many] { $count } папок библиотеки
   *[other] { $count } папки библиотеки
    }. Каждые { $interval } с проверяется до { $budget } записей. Последняя успешная синхронизация: { $convergence }.
settings-library-watch-recovering-label = Встроенное отслеживание восстановлено
settings-library-watch-recovering-desc = Встроенное отслеживание снова активно. Полное переходное сканирование ещё идёт, прежде чем библиотека будет считаться актуальной.
settings-library-watch-unavailable-label = Отслеживание библиотеки недоступно
settings-library-watch-unavailable-desc = Встроенное отслеживание файлов не работает, а резервный опрос выключен. Включите резервный опрос, затем перезапустите skwd-walld.
settings-library-watch-recovered-label = Встроенное отслеживание восстановлено
settings-library-watch-recovered-desc = Встроенное отслеживание и переходное сканирование актуальны. Последняя успешная синхронизация: { $convergence }.
settings-library-watch-native-label = Встроенное отслеживание файлов
settings-library-watch-native-desc = События файловой системы активны для всех папок библиотеки. Опрос простаивает.
settings-library-watch-convergence-never = Ещё не завершено
settings-library-watch-convergence-seconds = { $value ->
    [one] { $value } секунду назад
    [few] { $value } секунды назад
    [many] { $value } секунд назад
   *[other] { $value } секунды назад
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] { $value } минуту назад
    [few] { $value } минуты назад
    [many] { $value } минут назад
   *[other] { $value } минуты назад
    }
settings-library-watch-convergence-hours = { $value ->
    [one] { $value } час назад
    [few] { $value } часа назад
    [many] { $value } часов назад
   *[other] { $value } часа назад
    }
settings-profiles-card = Профили
settings-profiles-card-desc = Определяют, какие папки используют выбор обоев, фильтр папок, случайные обои и расписания. Скрытые папки возвращаются переключателем скрытого на панели или его сочетанием клавиш, и при каждом запуске они снова скрыты.
settings-profiles-everything = Всё
settings-profiles-name-label = Название
settings-profiles-name-placeholder = Название профиля
settings-profiles-state-show = Показать
settings-profiles-state-hide = Скрыть
settings-profiles-state-exclude = Исключить
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = У элементов Workshop нет папок, поэтому это действует на все. Скройте отдельные обои на обороте их карточки.
settings-profiles-summary = скрыто: { $hidden }, исключено: { $excluded }
settings-profiles-summary-none = Показываются все папки
settings-profiles-delete-action = Удалить профиль
settings-profiles-add-label = Новый профиль
settings-profiles-new-name = Новый профиль
settings-profiles-tree-label = Папки в этом профиле
settings-profiles-tree-desc = «Показать» оставляет папку в выборе и в случайных обоях. «Скрыть» убирает её из виду, пока вы не нажмёте переключатель скрытого на панели или его сочетание клавиш. «Исключить» убирает её из профиля, и переключатель её пропускает. Папка задаёт состояние и своим подпапкам; измените подпапку потом, чтобы сделать исключение.
settings-profiles-never-scanned = Не сканируется
settings-profiles-mixed = Смешанное содержимое
settings-profiles-hidden-inside = { $count ->
    [one] { $count } скрытая внутри
    [few] { $count } скрытые внутри
   *[many] { $count } скрытых внутри
    }
settings-profiles-copy-action = Копировать
settings-profiles-helm-desc = Делает этот профиль активным, и ротация с расписаниями следуют ему.
settings-profiles-flag-desc = Открывает выбор в этом профиле, не меняя активный.
settings-profiles-make-active = Сделать активным
settings-profiles-active-tag = Активный
settings-profiles-builtin-tag = Встроенный
settings-profiles-filter-placeholder = Фильтр папок
settings-profiles-no-match = Ни одна папка не подходит.
settings-profiles-switch-label = Переключиться на этот профиль
settings-profiles-switch-desc = skwd-helm меняет сохранённый профиль, поэтому ротация и расписания следуют за ним. Флаг запуска открывает один выбор в этом профиле и не трогает сохранённый.
settings-profiles-hidden-note-label = Скрытые обои
settings-profiles-hidden-note-desc = Кнопка скрытых в панели (Ctrl+H) показывает скрытые папки активного профиля и каждые обои, скрытые с обратной стороны карточки. Каждый запуск снова их скрывает.
settings-profiles-tree-note = «Не сканируется» берётся из исключённых папок в Библиотека › Папки, поэтому ни один профиль не может их показать. Исключить здесь лишь убирает папку из этого профиля. Обои, скрытые с карточки, остаются скрытыми во всех профилях, включая «Всё», пока не нажать «Показать скрытые». Папки с точкой, такие как .private, здесь обычные папки.
settings-profiles-legend-show = в выборе и случайных обоях.
settings-profiles-legend-hide = до кнопки скрытых в панели или Ctrl+H. Никогда случайно.
settings-profiles-legend-exclude = из этого профиля. «Показать скрытые» их пропускает.
settings-profiles-everything-desc = «Всё» показывает каждую отсканированную папку и каждый элемент Workshop. Создайте новый профиль, чтобы скрыть или исключить часть из них.
settings-profiles-delete-hint = Нажмите ещё раз, чтобы удалить. Обои и папки остаются на месте.
