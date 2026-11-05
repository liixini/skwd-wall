settings-wallpapers-displays-desc = Qué muestra cada pantalla y su colocación.
settings-wallpapers-play = Cómo se reproducen los fondos
settings-wallpapers-play-desc = Motores, su capa y escenas.
settings-wallpapers-transitions-desc = Cómo cambia un fondo al siguiente.
settings-wallpapers-awww-desc = La transición de awww para imágenes.
settings-wallpapers-drawing = Dibujo
settings-wallpapers-drawing-desc = Los fondos preparados se dibujan solos.
settings-wallpapers-sound = Sonido
settings-wallpapers-sound-desc = Volumen, y cuándo se aparta el audio del fondo.
settings-wallpapers-pause-desc = Pausa los fondos animados según reglas.
settings-wallpapers-plasma = Pantalla de bloqueo de Plasma
settings-wallpapers-plasma-desc = Qué fondo muestra Plasma al bloquear.
settings-wallpapers-niri = Vista general de Niri
settings-wallpapers-niri-desc = En niri. Un fondo detrás de la vista general.
settings-wallpapers-display-more = Más
settings-wallpapers-lock = Bloquear { $name }
settings-wallpapers-lock-hint = Solo se actualiza con el multiselector
settings-wallpapers-layer-bottom = Abajo
settings-wallpapers-gpu-hint = Vuelve a Automático
settings-wallpapers-timeout-hint = Para imágenes muy grandes
settings-wallpapers-animated = Fondos animados
settings-wallpapers-scenes = Escenas
settings-wallpapers-scenes-hint = Límite FPS · escala
settings-wallpapers-particles-hint = Solo escenas
settings-wallpapers-preview-hint = Frecuencia
settings-wallpapers-quality-short = La arena necesita al menos { $ms } ms.
settings-wallpapers-quality-hint = Full sin límite, Low limitado a 30 fps
settings-wallpapers-sharpen-hint = Cuando la calidad reduce la resolución
settings-wallpapers-monitors = Pantallas
settings-wallpapers-primary-hint = Anima solo con Principal
settings-wallpapers-primary-blank = la primera disponible
settings-wallpapers-primary-only-hint = El resto usa todas las pantallas
settings-wallpapers-wave = Onda
settings-wallpapers-wave-hint = Ancho × alto
settings-wallpapers-position-hint = center, top-left o 0.5,0.5
settings-wallpapers-bezier-hint = x1,y1,x2,y2
settings-wallpapers-bezier-blank = predeterminado de awww
settings-wallpapers-invert-hint = Invierte el eje Y del origen
settings-wallpapers-colours-files = Colores y archivos
settings-wallpapers-preparing = Preparación
settings-wallpapers-paper-ink = Papel y tinta
settings-wallpapers-prepare = Preparar
settings-wallpapers-prepare-all = Todos los fondos
settings-wallpapers-scope-view = Vista actual · { $count }
settings-wallpapers-pack-ready = Archivos de procesado listos.
settings-wallpapers-pack-none = Aún no hay archivos de procesado.
settings-wallpapers-pack-blank = Ninguno aún
settings-wallpapers-drawing-blocked = Dibujo no disponible. Elige primero los archivos de procesado.
settings-wallpapers-volume = Volumen
settings-wallpapers-other-apps = Otras apps
settings-wallpapers-pause-apps = Se pausa mientras se ejecutan las apps de la lista.
settings-wallpapers-pause-none = No hay ninguna regla de pausa activa.
settings-wallpapers-process-hint = Admite * y ?
settings-wallpapers-process-action = Elegir
settings-wallpapers-process-blank = ninguno aún
settings-wallpapers-resume-hint = Tras terminar la última regla
settings-wallpapers-scope-all = todas las pantallas
settings-wallpapers-scope-display = la pantalla de la ventana
settings-wallpapers-rule-fullscreen = ventanas a pantalla completa
settings-wallpapers-rule-maximised = ventanas maximizadas
settings-wallpapers-rule-focused = ventanas enfocadas
settings-wallpapers-rule-full-width = columnas de ancho completo
settings-wallpapers-plasma-left = queda en manos de Plasma
settings-wallpapers-plasma-static = muestra una imagen fija
settings-wallpapers-plasma-follow = sigue el último fondo aplicado
settings-wallpapers-plasma-off-row = Sin gestionar deja Plasma como está.
settings-wallpapers-image-hint = Ruta absoluta, o que empiece por ~/
settings-wallpapers-image-blank = sin imagen
settings-wallpapers-dynamic-hint = El fotograma es una imagen muda
settings-wallpapers-niri-off = Desactivada. La vista general mantiene el fondo de niri.
settings-wallpapers-source = Origen
settings-wallpapers-look = Aspecto
settings-wallpapers-source-blank = último fondo
settings-wallpapers-blur = Desenfocar el fondo
settings-wallpapers-radii = Radio de desenfoque
settings-wallpapers-radii-hint = Imágenes · vídeos · escenas
settings-wallpapers-dim-hint = 0 nada, 100 negro
settings-wallpapers-transitions-off = Desactivadas. Cada fondo pasa directamente al siguiente.
settings-wallpapers-transitions-off-foot = Activa las transiciones para elegir la vista previa, las pantallas y la calidad de arena.
settings-wallpapers-awww-none = Las imágenes cambian con { $type }.
settings-wallpapers-awww-no-type = ninguna transición
settings-wallpapers-sound-muted = El audio de los fondos está silenciado.
settings-wallpapers-displays-say = { $state ->
        [none] Ninguna pantalla bloqueada.
        [one] { $names } está bloqueada.
        [some] { $names } están bloqueadas.
       *[detecting] Detectando pantallas.
    } Sin colocación propia, las pantallas usan { $fill }.
settings-wallpapers-play-say = Imágenes con { $backend }, vídeos con { $val }Vulkan{ $end }, en la capa { $layer }.
settings-wallpapers-transitions-say = Familia { $family }{ $kind ->
        [random] , distinta cada vez, en { $duration }.
        [preset] , con { $shader }, en { $ms } ms.
       *[shader] , con { $shader }, en { $duration }.
    } Frecuencia { $rate }.
settings-wallpapers-primary-count = { $state ->
        [none] ninguna de { $all }
        [all] las { $all }
       *[some] { $shown } de { $all }
    }
settings-wallpapers-awww-say = Imágenes con { $type } y filtro { $filter }, en { $duration }.
settings-wallpapers-drawing-say = { $state ->
        [on] Los fondos preparados se dibujan en { $duration }.
       *[off] Desactivado. Las tarjetas con Dibujo tardan { $duration }.
    }
settings-wallpapers-sound-say = El audio de los fondos suena a { $volume }{ $duck ->
        [yes] { " " }y se pausa si otra app reproduce sonido.
       *[no] .
    }
settings-wallpapers-pause-say = Pausa con { $rules } en { $scope }{ $apps ->
        [yes] , y con apps de la lista.
       *[no] .
    }
settings-wallpapers-pause-rules = { $count } reglas
settings-wallpapers-plasma-say = { $state ->
        [off] La pantalla de bloqueo { $mode }.
        [static] La pantalla de bloqueo { $mode }: { $image }.
       *[follow] La pantalla de bloqueo { $mode }, escenas como { $scenes ->
        [live] render en vivo.
       *[poster] fotograma fijo.
    }
    }
settings-wallpapers-niri-say = Muestra { $source ->
       *[current] el fondo actual
        [own] su propio origen
        [stationary] un fondo fijo
    }{ $blur ->
        [yes] , desenfocado
       *[no] { "" }
    }, atenuado { $dim }{ $auto ->
        [yes] , recoloreado
       *[no] { "" }
    }.
settings-wallpapers-device-hint = Automático prueba CUDA, WebGPU y luego CPU
settings-wallpapers-memory-hint = Si se supera, Automático reintenta en CPU
