settings-system-power-desc = El selector y los fondos tienen límites de batería separados.
settings-system-power-say = Con batería { $cap ->
        [none] el selector no tiene límite adicional ({ $fps })
       *[cap] el selector se limita a { $fps }
    }{ $idle ->
        [off] , solo se aplica la pausa por inactividad normal ({ $seconds })
       *[on] , el vídeo se detiene tras { $seconds } sin actividad
    } y { $detail ->
        [full] los fondos mantienen todo el detalle
       *[performance] los fondos pasan al modo rendimiento
    }.
settings-system-power-off = Desactivado. Con batería, el selector y los fondos funcionan con los mismos límites que enchufados.
settings-system-power-source = Energía actual: { $source }
settings-system-always = Siempre
settings-system-on-battery = Con batería · mientras el ahorro está activo
settings-system-max-fps-hint = La frecuencia de la pantalla, o 60 para un portátil más fresco
settings-system-idle-hint = 0 desactiva
settings-system-battery-fps-hint = El menor de este y FPS máx. · 0 desactiva
settings-system-battery-idle-hint = Gana el más corto · 0 desactiva
settings-system-battery-wallpaper-hint = Vídeo directo, escenas a 30 fps
settings-system-gpu-desc = Vuelve a abrir el selector después de cambiarlo.
settings-system-gpu-say = { $mode ->
        [low] El selector usa { $gpu }: pide una GPU integrada.
        [high] El selector usa { $gpu }: pide una GPU dedicada.
        [none] El selector usa { $gpu }: decide el controlador.
        [auto] El selector usa { $gpu }: prefiere una GPU de bajo consumo con el ahorro de batería activo.
       *[custom] El selector usa { $gpu }, que no reconoce, así que actúa como Auto.
    }
settings-system-language-desc = Los cambios se aplican al momento y se guardan.
settings-system-language-say = { $follow ->
        [desktop] La interfaz usa { $language }, el idioma del escritorio. Los nombres y etiquetas de los fondos no cambian.
       *[chosen] La interfaz usa { $language }. Los nombres y etiquetas de los fondos no cambian.
    }
settings-system-bug-title = Informe de errores
settings-system-bug-desc = Un archivo para adjuntar a una incidencia.
settings-system-bug-say = Reúne la versión, el entorno y los registros recientes en un solo archivo.
