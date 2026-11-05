settings-colours-source = De dónde viene el color
settings-colours-source-desc = Sigue cada fondo, conserva una paleta o deja los colores como están.
settings-colours-source-follow = siguen cada fondo
settings-colours-source-fixed = vienen de una paleta fija
settings-colours-source-off = se quedan como están
settings-colours-source-say = Los colores { $policy }.
settings-colours-source-made = Los colores { $policy }, creados por { $authority }.
settings-colours-end4 = end-4
settings-colours-control-menu = Control de color · shells detectados
settings-colours-generator-menu = Generador · solo instalados
settings-colours-theme-menu = Tema · predefinidos, tuyos, semillas
settings-colours-all-presets = Los { $count } predefinidos…
settings-colours-generator = Generador
settings-colours-generator-desc = El programa que convierte el fondo en una paleta.
settings-colours-generator-say = { $engine } crea la paleta.
settings-colours-generator-seeded = { $engine } crea la paleta, con semilla de la ranura { $index } y contraste { $contrast }.
settings-colours-generator-shell = { $shell } crea la paleta a partir del fondo.
settings-colours-palette = Paleta
settings-colours-palette-desc = Una paleta para todos los fondos.
settings-colours-palette-say = { $theme } para todos los fondos.
settings-colours-idle = No se usa mientras los colores se quedan como están.
settings-colours-index-hint = Qué ranura de la paleta usar como semilla
settings-colours-contrast-hint = -1 a 1
settings-colours-wallust-hint = Fuerza una paleta de wallust, p. ej. dark16. Vacío elige según el modo
settings-colours-seeds-hint = Lo usan las semillas propias
settings-colours-config-hint = Matugen externo
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = Automático
settings-colours-look = Aspecto
settings-colours-look-desc = Cuánto se extiende el color, su acabado y si es claro u oscuro.
settings-colours-look-say = Carácter { $scheme } con acabado { $finish }, variante { $variant }.
settings-colours-look-imported = Paleta de { $shell }, importada como variante { $variant }.
settings-colours-look-shell = { $shell } define su propio aspecto.
settings-colours-look-fixed = La paleta fija define su propio aspecto.
settings-colours-no-designer = El diseñador de temas está disponible con Seguir el fondo o Paleta fija.
settings-colours-designer-hint = Ajusta o crea una paleta a mano
settings-colours-go = A dónde van los colores
settings-colours-go-desc = Temas de apps que siguen la paleta cuando cambia.
settings-colours-find-hint = Vuelve a comprobar tras instalar una app
settings-colours-matugen-hint = Interruptor de las salidas de Matugen
settings-colours-reload-hint = Se ejecuta tras escribir el archivo
settings-colours-output-heading = Salida { $count }
settings-colours-shells = Shells
settings-colours-shells-desc = Noctalia y Dank Material Shell pueden usar los colores de Skwd.
settings-colours-dms-hint = DMS debe usar Auto (fondo)
settings-colours-mode-hint = Se aplica con cada esquema de color
settings-colours-bin-hint = Si no está en el PATH
settings-colours-lock = Pantallas de bloqueo
settings-colours-lock-desc = Pantallas de bloqueo que muestran tu fondo.
settings-colours-lock-follow = el último fondo aplicado
settings-colours-lock-still = una imagen fija
settings-colours-sddm-hint = Requiere aprobación de administrador
settings-colours-go-say = { $val }{ $on } de { $all } apps{ $end } { $followers ->
        [one] sigue
       *[some] siguen
    } tus colores{ $changed ->
        [one] ; { $val }la configuración de { $name } cambió fuera de Skwd{ $end }.
        [some] ; { $val }{ $count } apps cambiaron fuera de Skwd{ $end }.
       *[none] .
    }
settings-colours-outputs-say = { $state ->
        [none] Ninguna integración genera una plantilla todavía.
        [single] { $val }1 integración{ $end } genera una plantilla en un archivo tras cada cambio de color.
       *[many] { $val }{ $count } integraciones{ $end } generan plantillas en archivos tras cada cambio de color.
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] sigue los colores de Skwd
       *[keeps] conserva sus colores
    }{ $end }; DMS { $val }{ $dms ->
        [follows] sigue los colores de Skwd
       *[keeps] conserva sus colores
    }{ $end }{ $caelestia ->
        [yes] ; los fondos de Caelestia se sincronizan.
       *[no] .
    }
settings-colours-lock-say = { $state ->
        [none] No hay pantallas de bloqueo configuradas. Las nuevas usan { $mode }.
        [one] { $names } usa { $mode }.
       *[some] { $names } usan { $mode }.
    }
settings-colours-lock-say-static = { $state ->
        [none] No hay pantallas de bloqueo configuradas. Las nuevas usan { $mode }: { $image }.
        [one] { $names } usa { $mode }: { $image }.
       *[some] { $names } usan { $mode }: { $image }.
    }
settings-menu-more = Más
