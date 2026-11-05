settings-automation-rotation-desc = Aplica un fondo aleatorio del grupo cada vez que vence el intervalo.
settings-automation-rotation-say = Cada { $interval }, una elección al azar entre { $val }{ $pool }{ $end }.
settings-automation-rotation-off = Desactivado. El fondo solo cambia cuando eliges uno.
settings-automation-pool = { $count ->
        [none] un grupo vacío
        [one] { $first }
        [two] { $first } y { $second }
       *[three] { $first }, { $second } y { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] { $pool } favoritos
       *[all] { $pool }
    }
settings-automation-pool-images = imágenes
settings-automation-pool-videos = vídeos
settings-automation-pool-engine = elementos de Wallpaper Engine
settings-automation-pool-heading = El grupo incluye
settings-automation-timing = Tiempo
settings-automation-interval-hint = mínimo 10 s
settings-automation-schedule-desc = Las reglas eligen el fondo y el tema según la hora, el día, el tiempo, la energía y las pantallas. Gana la primera regla que coincide.
settings-automation-schedule-say = { $start ->
        [applies] Las reglas están activas; la primera regla que coincide { $val }se aplica al iniciar{ $end }. Un fondo que elijas tú sigue ganando.
       *[waits] Las reglas están activas; la primera regla que coincide { $val }espera al siguiente cambio{ $end }. Un fondo que elijas tú sigue ganando.
    }
settings-automation-schedule-off = Desactivado. Tus reglas se conservan hasta que lo actives.
settings-automation-start-hint = primera regla que coincide al iniciar el servicio
settings-automation-editor-hint = crea reglas con bloques de condiciones
settings-automation-location = Ubicación y tiempo
settings-automation-location-desc = Se usa para las horas de salida y puesta del sol y para las condiciones del tiempo.
settings-automation-location-say = Tiempo para { $city }, horas del sol en { $lat }, { $lon }. Cambios según el tiempo: { $mode }.
settings-automation-no-city = sin ciudad
settings-automation-no-latitude = sin latitud
settings-automation-no-longitude = sin longitud
settings-automation-location-hint = Deja la ciudad vacía para usar las coordenadas. La latitud es positiva al norte y la longitud, al este.
settings-automation-startup = Inicio y avisos
settings-automation-startup-desc = Qué ocurre cuando se inicia el servicio y cuando cambia el fondo.
settings-automation-startup-say = { $restore ->
        [restores] Al iniciar, Skwd { $val }vuelve a aplicar el último fondo{ $end }.
       *[leaves] Al iniciar, Skwd { $val }deja el escritorio como está{ $end }.
    } { $notify ->
        [notifies] Los cambios de fondo { $val }envían una notificación{ $end }.
       *[silent] Los cambios de fondo { $val }son silenciosos{ $end }.
    }
settings-automation-commands = Después de aplicar un fondo
settings-automation-commands-desc = Ejecuta tus propios comandos tras cada cambio, por ejemplo para controlar otro gestor de fondos.
settings-automation-command-count = { $count ->
        [one] { $val }{ $count } comando{ $end } se ejecuta tras cada cambio.
       *[other] { $val }{ $count } comandos{ $end } se ejecutan tras cada cambio.
    }
settings-automation-commands-say = { $listed ->
        [none] { $val }Ningún comando{ $end } se ejecuta tras cada cambio.
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }deja la aplicación a tus comandos{ $end } y los comandos { $val }también se ejecutan en la restauración al iniciar{ $end }.
           *[skip] Skwd { $val }deja la aplicación a tus comandos{ $end } y los comandos { $val }omiten la restauración al iniciar{ $end }.
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }aplica el fondo por sí mismo{ $end } y los comandos { $val }también se ejecutan en la restauración al iniciar{ $end }.
           *[skip] Skwd { $val }aplica el fondo por sí mismo{ $end } y los comandos { $val }omiten la restauración al iniciar{ $end }.
        }
    }
settings-automation-pick-only-hint = tus comandos lo aplican
settings-automation-commands-heading = Comandos
settings-automation-placeholders = Usa %path%, %thumb%, %type% y %name% en un comando.
settings-automation-minutes = { $count } min
settings-automation-mode-off = Desactivado
settings-automation-mode-outdoors = Seguir el tiempo de fuera
settings-automation-mode-playlists = Elegir de tus listas
settings-automation-mode-semantic = Buscar en tu biblioteca
settings-automation-condition-clear = Despejado
settings-automation-condition-cloudy = Nublado
settings-automation-condition-rainy = Lluvioso
settings-automation-condition-snowy = Nevado
settings-automation-condition-stormy = Tormentoso
settings-automation-condition-foggy = Neblinoso
settings-automation-condition-windy = Ventoso
settings-automation-display-another = Otro
settings-automation-display-exclude = Excluir
settings-automation-display-keep = Mantener
settings-automation-playlists-heading = Lista por tiempo
settings-automation-hours = { $count } h
