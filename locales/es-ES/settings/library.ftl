settings-library-watch-section-desc = Detecta archivos añadidos fuera de skwd-wall y mantiene la biblioteca actualizada.
settings-library-watch-fallback-label = Respaldo por sondeo
settings-library-watch-fallback-desc = Comprueba solo las carpetas de la biblioteca que la vigilancia nativa no puede observar. Actívalo para montajes de red o FUSE que pierden cambios y luego reinicia skwd-walld.
settings-library-watch-interval-label = Intervalo de sondeo
settings-library-watch-interval-desc = Espera estos segundos entre comprobaciones acotadas. Valores más bajos encuentran cambios antes pero leen el sistema de archivos con más frecuencia. Reinicia skwd-walld tras cambiarlo.
settings-library-watch-unknown-label = Estado de vigilancia no disponible
settings-library-watch-unknown-desc = Este daemon no informa del estado de vigilancia de la biblioteca. Actualiza o reinicia skwd-walld.
settings-library-watch-poll-failed-label = El sondeo no puede leer una carpeta de la biblioteca
settings-library-watch-poll-failed-desc = Comprueba que cada carpeta configurada esté montada y sea legible. El sondeo reintentará en { $interval } segundos.
settings-library-watch-polling-label = Respaldo por sondeo activo
settings-library-watch-polling-desc = La vigilancia nativa falló para { $count ->
    [one] una carpeta de la biblioteca
   *[other] { $count } carpetas de la biblioteca
    }. Se comprueban hasta { $budget } entradas cada { $interval } segundos. Última convergencia exitosa: { $convergence }.
settings-library-watch-recovering-label = Vigilancia nativa recuperada
settings-library-watch-recovering-desc = El vigilante nativo está activo de nuevo. Aún se está ejecutando un escaneo completo de traspaso antes de declarar la biblioteca actualizada.
settings-library-watch-unavailable-label = Vigilancia de biblioteca no disponible
settings-library-watch-unavailable-desc = La vigilancia nativa falló y el respaldo por sondeo está desactivado. Activa el respaldo por sondeo y reinicia skwd-walld.
settings-library-watch-recovered-label = Vigilancia nativa restaurada
settings-library-watch-recovered-desc = El vigilante nativo y su escaneo de traspaso están actualizados. Última convergencia exitosa: { $convergence }.
settings-library-watch-native-label = Vigilancia nativa de archivos
settings-library-watch-native-desc = Los eventos del sistema de archivos están activos para cada carpeta de la biblioteca. El sondeo está inactivo.
settings-library-watch-convergence-never = Aún no completado
settings-library-watch-convergence-seconds = { $value ->
    [one] hace 1 segundo
   *[other] hace { $value } segundos
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] hace 1 minuto
   *[other] hace { $value } minutos
    }
settings-library-watch-convergence-hours = { $value ->
    [one] hace 1 hora
   *[other] hace { $value } horas
    }
settings-profiles-card = Perfiles
settings-profiles-card-desc = Decide qué carpetas usan el selector, el filtro de carpetas, los fondos aleatorios y las programaciones. Las carpetas ocultas vuelven con el interruptor de ocultos de la barra o su atajo de teclado, y cada inicio vuelve a ocultarlas.
settings-profiles-everything = Todo
settings-profiles-name-label = Nombre
settings-profiles-name-placeholder = Nombre del perfil
settings-profiles-state-show = Mostrar
settings-profiles-state-hide = Ocultar
settings-profiles-state-exclude = Excluir
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = Los elementos de Workshop no tienen carpetas, así que esto se aplica a todos. Oculta un fondo concreto desde el reverso de su tarjeta.
settings-profiles-summary = { $hidden } ocultas, { $excluded } excluidas
settings-profiles-summary-none = Muestra todas las carpetas
settings-profiles-delete-action = Eliminar perfil
settings-profiles-add-label = Nuevo perfil
settings-profiles-new-name = Nuevo perfil
settings-profiles-tree-label = Carpetas de este perfil
settings-profiles-tree-desc = Mostrar mantiene la carpeta en el selector y en el aleatorio. Ocultar la mantiene fuera de la vista hasta que pulses el interruptor de ocultos de la barra o su atajo. Excluir la deja fuera del perfil, así que el interruptor la omite. Una carpeta también fija sus subcarpetas; cambia una subcarpeta después para hacer una excepción.
settings-profiles-never-scanned = Nunca se analiza
settings-profiles-mixed = Mezcla dentro
settings-profiles-hidden-inside = { $count ->
    [one] { $count } oculta dentro
   *[other] { $count } ocultas dentro
    }
settings-profiles-copy-action = Copiar
settings-profiles-helm-desc = Lo convierte en el perfil activo, así que la rotación y las programaciones lo siguen.
settings-profiles-flag-desc = Abre un selector en este perfil y deja el activo como está.
settings-profiles-make-active = Activar
settings-profiles-active-tag = Activo
settings-profiles-builtin-tag = Integrado
settings-profiles-filter-placeholder = Filtrar carpetas
settings-profiles-no-match = Ninguna carpeta coincide.
settings-profiles-switch-label = Cambiar a este perfil
settings-profiles-switch-desc = skwd-helm cambia el perfil guardado, así que la rotación y las programaciones lo siguen. La opción de lanzamiento abre un selector en este perfil y deja el guardado intacto.
settings-profiles-hidden-note-label = Fondos ocultos
settings-profiles-hidden-note-desc = El botón de ocultos de la barra (Ctrl+H) muestra las carpetas ocultas del perfil activo y cada fondo ocultado desde su tarjeta. Cada inicio vuelve a ocultarlos.
settings-profiles-tree-note = Nunca analizado viene de las carpetas excluidas en Biblioteca › Carpetas, así que ningún perfil puede mostrarlo. Excluir aquí solo deja una carpeta fuera de este perfil. Los fondos ocultados desde su tarjeta siguen ocultos en todos los perfiles, Todo incluido, hasta Mostrar ocultos. Las carpetas con punto como .private son carpetas normales aquí.
settings-profiles-legend-show = en el selector y el aleatorio.
settings-profiles-legend-hide = hasta el botón de ocultos de la barra o Ctrl+H. Nunca aleatorio.
settings-profiles-legend-exclude = de este perfil. Mostrar ocultos lo omite.
settings-profiles-everything-desc = Todo muestra cada carpeta analizada y cada elemento del Workshop. Crea un perfil nuevo para ocultar o excluir algunos.
settings-profiles-delete-hint = Haz clic otra vez para eliminar. Los fondos y las carpetas se quedan donde están.
