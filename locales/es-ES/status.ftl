status-daemon-version-mismatch = Versión del daemon no coincide: skwd-walld es { $daemon }, esta app es { $gui }. Reinicia el daemon (pkill -x skwd-walld).
status-daemon-connecting = Conectando con skwd-walld… Si no desaparece, revisa ~/.cache/skwd-wall-v2/skwd-walld.log.
status-daemon-lost = Se perdió la conexión con skwd-walld. Reintentando; revisa ~/.cache/skwd-wall-v2/skwd-walld.log si no se reconecta.
status-library-empty = Aún no hay fondos. Añade imágenes a { $directory } o abre el navegador en línea.
status-diagnostics-passed =
    { $count ->
        [one] Diagnóstico: { $count } comprobación superada
       *[other] Diagnóstico: las { $count } comprobaciones superadas
    }
status-diagnostics-more = (+{ $count } más)
status-diagnostics-issues =
    Diagnóstico: { $count } { $count ->
        [one] problema
       *[other] problemas
    } - { $issues }{ $more }
status-apply-file-missing = Error al aplicar: falta el archivo del fondo
status-apply-renderer-failed = Error al aplicar: el renderizador no pudo iniciarse
status-apply-decode-failed = Error al aplicar: no se pudo decodificar el fondo
status-apply-no-outputs = Error al aplicar: ninguna salida coincide
status-apply-invalid-request = Error al aplicar: solicitud no válida
status-apply-failed = Error al aplicar
status-apply-detail = { $heading } ({ $detail })
status-unsubscribe-failed = Eliminado localmente, pero Steam no pudo cancelar la suscripción. El elemento podría descargarse de nuevo mientras siga suscrito.
status-disk-full = Disco lleno. Se detuvo la importación de miniaturas; libera espacio y vuelve a escanear.
status-theme-failed = La actualización del tema falló. Se mantuvieron los colores actuales.
status-theme-backend-missing = { $requested } no está instalado. El tema se generó con { $effective } en su lugar.
status-download-wallpaper = Descargando fondo
status-effects-static-only = Los efectos solo funcionan con imágenes.
status-random-rotation-on = La rotación aleatoria está activada.
status-random-rotation-off = La rotación aleatoria está desactivada.
status-demo-playback-missing = Falta el fondo de demostración de reproducción: { $key }
status-demo-missing = Falta el fondo de demostración: { $key }
status-demo-filtered = El fondo de demostración fue filtrado: { $key }
status-bug-report-saved = Informe de error guardado: { $path }
status-bug-report-failed = No se pudo guardar el informe de error. Revisa el registro para ver el error real.
status-keybinds-reset = Controles restaurados a los valores por defecto.
