settings-system-power-desc = У окна выбора и обоев отдельные ограничения для батареи.
settings-system-power-say = От батареи { $cap ->
        [none] у окна выбора нет дополнительного ограничения ({ $fps })
       *[cap] окно выбора ограничено до { $fps }
    }{ $idle ->
        [off] , действует только обычная пауза при бездействии ({ $seconds })
       *[on] , видео останавливается через { $seconds } без ввода
    }, а { $detail ->
        [full] обои сохраняют полную детализацию
       *[performance] обои переходят в режим производительности
    }.
settings-system-power-off = Выкл. От батареи окно выбора и обои работают с теми же ограничениями, что и от сети.
settings-system-power-source = Питание: { $source }
settings-system-always = Всегда
settings-system-on-battery = От батареи · пока экономия включена
settings-system-max-fps-hint = Частота обновления экрана или 60 для более прохладного ноутбука
settings-system-idle-hint = 0 отключает
settings-system-battery-fps-hint = Меньшее из этого и макс. FPS · 0 отключает
settings-system-battery-idle-hint = Действует более короткое · 0 отключает
settings-system-battery-wallpaper-hint = Прямое видео, сцены в 30 fps
settings-system-gpu-desc = После изменения откройте окно выбора заново.
settings-system-gpu-say = { $mode ->
        [low] Окно выбора использует { $gpu }: оно запрашивает встроенный GPU.
        [high] Окно выбора использует { $gpu }: оно запрашивает дискретный GPU.
        [none] Окно выбора использует { $gpu }: решает драйвер.
        [auto] Окно выбора использует { $gpu }: предпочитает экономичный GPU при экономии батареи.
       *[custom] Окно выбора использует { $gpu }, но не знает его и ведёт себя как «Авто».
    }
settings-system-language-desc = Изменения применяются сразу и сохраняются.
settings-system-language-say = { $follow ->
        [desktop] Язык интерфейса: { $language }, он следует языку рабочего стола. Названия и теги обоев не меняются.
       *[chosen] Язык интерфейса: { $language }. Названия и теги обоев не меняются.
    }
settings-system-bug-title = Отчёт об ошибке
settings-system-bug-desc = Один файл, который можно приложить к задаче.
settings-system-bug-say = Соберите версию, окружение и свежие журналы в один файл.
