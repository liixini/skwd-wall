settings-library-folders-desc = Analiza estas carpetas y sus subcarpetas en busca de imágenes y vídeos.
settings-library-folders-say = Analizando { $val }{ $folders }{ $end }.{ $state ->
        [polling] { " " }El sondeo cubre lo que la vigilancia nativa no ve, cada { $every }.{ $unread }
        [native] { " " }La vigilancia nativa está activa; el sondeo está en reposo.
        [down] { " " }La vigilancia nativa falló y el sondeo está desactivado.
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }{ $count } carpeta no se puede leer.
       *[other] { " " }{ $count } carpetas no se pueden leer.
    }
settings-library-watching = Vigilancia
settings-library-poll-hint = Para carpetas de red o FUSE
settings-library-interval-hint = Reinicia skwd-walld tras cambiarlo
settings-library-excluded-desc = Omite estas carpetas y sus subcarpetas. Los metadatos existentes se conservan.
settings-library-excluded-say = Nunca se analizan { $val }{ $folders }{ $end }.{ $outside }
settings-library-outside = { $count ->
        [one] { " " }{ $count } está fuera de toda carpeta de fondos y no omite nada.
       *[other] { " " }{ $count } están fuera de toda carpeta de fondos y no omiten nada.
    }
settings-library-excluded-hint = Las carpetas excluidas nunca se analizan, así que ningún perfil puede mostrarlas. Para conservar una carpeta pero ocultarla en público, usa { $link }.
settings-library-profiles-desc = Decide qué carpetas usan el selector, el filtro de carpetas, los fondos aleatorios y las programaciones.
settings-library-profiles-say = Perfil activo { $state ->
        [none] { $menu }, que muestra todas las carpetas.
       *[some] { $menu }: { $summary }.
    } Ctrl+H o el botón de ocultos de la barra muestra los fondos ocultos, incluidos los ocultados desde su tarjeta, hasta el próximo inicio.
settings-library-active = Perfil activo
settings-library-terminal = Cambiar desde una terminal
settings-library-tree-note = Ocultar queda fuera del azar hasta Ctrl+H. Excluir queda fuera incluso con Ctrl+H. Una carpeta fija sus subcarpetas.
settings-library-show-tip = En el selector y el azar
settings-library-hide-tip = Hasta Ctrl+H, nunca al azar
settings-library-exclude-tip = Fuera de este perfil, Ctrl+H lo omite
settings-library-from = Desde { $link }
settings-library-excluded-inside = { $count } excluidas dentro
settings-library-both-inside = { $hidden } ocultas, { $excluded } excluidas dentro
settings-library-storage = Almacenamiento y papelera
settings-library-storage-desc = Copias WebP más ligeras, almacenamiento de recuperación y copias recoloreadas.
settings-library-storage-say = { $optimise ->
        [on] Las imágenes nuevas pasan a WebP con calidad { $quality }, hasta { $size }.
       *[off] Las imágenes nuevas se quedan como están. Optimizar todo usa calidad { $quality }, hasta { $size }.
    } { $trash ->
        [on] Los originales en la papelera siguen { $days } y luego se eliminan.
       *[off] Los originales en la papelera siguen hasta que los elimines.
    } { $recolour ->
        [on] Los fondos nuevos también reciben una copia recoloreada.
       *[off] Los fondos nuevos no se recolorean.
    }
settings-library-retention-hint = Días para conservar imágenes en la papelera
settings-library-optimise-hint = Optimiza todas las imágenes de la biblioteca
settings-library-recolour = Recoloreado
settings-library-thumbnails = Miniaturas y reparación
settings-library-thumbnails-desc = Reconstruye vistas previas y colores cuando algo se vea mal.
settings-library-thumbnails-say = Se crean hasta { $jobs } miniaturas a la vez durante las reconstrucciones.
settings-library-rebuild = Reconstruir
settings-library-start-over = Empezar de nuevo
settings-library-recompute-hint = A partir de las miniaturas existentes
settings-library-clear-hint = Las miniaturas se regeneran en el próximo análisis
