settings-home-search = Busca cualquier ajuste: velocidad de fotogramas, sonido, pantalla de bloqueo, desenfoque
settings-home-displays = { $count ->
    [one] { $count } pantalla
   *[other] { $count } pantallas
 }
settings-home-displays-say = { $link } { $count ->
    [one] muestra fondos.
   *[other] muestran fondos.
 }
settings-home-watching = { " " }Vigilancia de la biblioteca: { $val }{ $state }{ $end }.
settings-home-model-ready = listo
settings-home-model-missing = sin modelo
settings-home-model-off = desactivado
settings-home-colours-from = { " " }{ $link } vienen de { $val }{ $engine }{ $end }.
settings-home-sound-desc = Audio de fondos de vídeo y de Wallpaper Engine.
settings-home-sound-say = Los fondos suenan al { $volume }{ $duck ->
    [yes] { " " }y se pausan cuando otras apps reproducen sonido.
   *[no] , incluso cuando otras apps reproducen sonido.
 }
settings-home-colours-desc = Cómo llegan los colores del fondo al resto del escritorio.
settings-home-colours-say = { settings-theme-policy-label }: { $policy }.{ " " }Más en { $place }.
settings-home-rotation-desc = Aplica un fondo aleatorio cada vez que vence el intervalo.
settings-home-rotation-say = Un fondo aleatorio cada { $interval }.
settings-home-rotation-off = La rotación está desactivada. El fondo solo cambia cuando eliges uno.
settings-home-transitions-desc = Cómo pasa un fondo al siguiente.
settings-home-transitions-say = Un fondo pasa al siguiente en { $duration }.
settings-home-transitions-off = Las transiciones están desactivadas. Los fondos cambian de golpe al siguiente.
settings-home-changed = Cambiado respecto al predeterminado
settings-home-changed-desc = Los ajustes que has cambiado, en un solo lugar, cada uno con Restablecer.
settings-home-changed-none = Nada difiere del valor predeterminado entre los ajustes comparados aquí.
settings-home-changed-say = { $count ->
    [one] { $count } ajuste difiere de su valor predeterminado.
   *[other] { $count } ajustes difieren de sus valores predeterminados.
 }
settings-home-review = Revisar
settings-home-reset = Restablecer
settings-home-reset-all = Restablecer todo
settings-home-badge = { $count } cambiados
settings-home-automatic = Automático
settings-home-displays-none = Ninguna pantalla
settings-home-displays-none-say = { $link } muestra fondos.
settings-home-reset-scope = Restablece todos los ajustes de arriba, incluidos los atajos de teclado. Las claves de API, los nombres de usuario, las ubicaciones y las carpetas se conservan.
settings-home-items = { $count ->
    [one] { $count } elemento
   *[other] { $count } elementos
 }
