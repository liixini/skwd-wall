settings-system-power-desc = O seletor e os papéis de parede têm limites de bateria separados.
settings-system-power-say = Na bateria, { $cap ->
        [none] o seletor não tem limite extra ({ $fps })
       *[cap] o seletor fica limitado a { $fps }
    }{ $idle ->
        [off] , só vale a pausa por inatividade normal ({ $seconds })
       *[on] , o vídeo para após { $seconds } sem uso
    } e { $detail ->
        [full] os papéis de parede mantêm todos os detalhes
       *[performance] os papéis de parede passam para o modo de desempenho
    }.
settings-system-power-off = Desligado. Na bateria, o seletor e os papéis de parede usam os mesmos limites que na tomada.
settings-system-power-source = Energia atual: { $source }
settings-system-always = Sempre
settings-system-on-battery = Na bateria · enquanto a economia estiver ligada
settings-system-max-fps-hint = A taxa de atualização da tela, ou 60 para um notebook mais fresco
settings-system-idle-hint = 0 desativa
settings-system-battery-fps-hint = O menor entre este e o FPS máx. · 0 desativa
settings-system-battery-idle-hint = Vale o mais curto · 0 desativa
settings-system-battery-wallpaper-hint = Vídeo direto, cenas 30 fps
settings-system-gpu-desc = Reabra o seletor depois de mudar.
settings-system-gpu-say = { $mode ->
        [low] O seletor usa { $gpu }: pede uma GPU integrada.
        [high] O seletor usa { $gpu }: pede uma GPU dedicada.
        [none] O seletor usa { $gpu }: o driver decide.
        [auto] O seletor usa { $gpu }: prefere uma GPU de baixo consumo com a economia de bateria ativa.
       *[custom] O seletor usa { $gpu }, que não reconhece, então age como Auto.
    }
settings-system-language-desc = As mudanças valem na hora e são salvas.
settings-system-language-say = { $follow ->
        [desktop] A interface usa { $language }, o idioma da área de trabalho. Nomes e tags dos papéis de parede não mudam.
       *[chosen] A interface usa { $language }. Nomes e tags dos papéis de parede não mudam.
    }
settings-system-bug-title = Relatório de bug
settings-system-bug-desc = Um arquivo para anexar a um problema.
settings-system-bug-say = Junta versão, ambiente e logs recentes em um único arquivo.
