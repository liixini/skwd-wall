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
