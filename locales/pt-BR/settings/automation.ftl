settings-automation-rotation-desc = Aplica um papel de parede aleatório do conjunto sempre que o intervalo termina.
settings-automation-rotation-say = A cada { $interval }, uma escolha aleatória entre { $val }{ $pool }{ $end }.
settings-automation-rotation-off = Desativado. O papel de parede só muda quando você escolhe um.
settings-automation-pool = { $count ->
        [none] um conjunto vazio
        [one] { $first }
        [two] { $first } e { $second }
       *[three] { $first }, { $second } e { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] { $pool } favoritos
       *[all] { $pool }
    }
settings-automation-pool-images = imagens
settings-automation-pool-videos = vídeos
settings-automation-pool-engine = itens do Wallpaper Engine
settings-automation-pool-heading = O conjunto inclui
settings-automation-timing = Tempo
settings-automation-interval-hint = mínimo de 10 s
settings-automation-schedule-desc = Regras escolhem o papel de parede e o tema pelo horário, dia da semana, clima, energia e telas. A primeira regra que corresponder vence.
settings-automation-schedule-say = { $start ->
        [applies] As regras estão ativas; a primeira regra que corresponder { $val }é aplicada na inicialização{ $end }. Um papel de parede que você escolher ainda vence.
       *[waits] As regras estão ativas; a primeira regra que corresponder { $val }espera a próxima mudança{ $end }. Um papel de parede que você escolher ainda vence.
    }
settings-automation-schedule-off = Desativado. Suas regras ficam guardadas até você ativar isto.
settings-automation-start-hint = primeira regra que corresponder quando o serviço iniciar
settings-automation-editor-hint = monte regras com blocos de condições
settings-automation-location = Local e clima
settings-automation-location-desc = Usado para os horários de nascer e pôr do sol e para as condições do clima.
settings-automation-location-say = Clima para { $city }, horários do sol em { $lat }, { $lon }. Mudanças pelo clima: { $mode }.
settings-automation-no-city = sem cidade
settings-automation-no-latitude = sem latitude
settings-automation-no-longitude = sem longitude
settings-automation-location-hint = Deixe a cidade vazia para usar as coordenadas. A latitude é positiva ao norte e a longitude, a leste.
settings-automation-startup = Inicialização e avisos
settings-automation-startup-desc = O que acontece quando o serviço inicia e quando o papel de parede muda.
settings-automation-startup-say = { $restore ->
        [restores] Na inicialização, o Skwd { $val }reaplica o último papel de parede{ $end }.
       *[leaves] Na inicialização, o Skwd { $val }deixa a área de trabalho como está{ $end }.
    } { $notify ->
        [notifies] As trocas de papel de parede { $val }enviam uma notificação{ $end }.
       *[silent] As trocas de papel de parede { $val }ficam em silêncio{ $end }.
    }
settings-automation-commands = Após aplicar um papel de parede
settings-automation-commands-desc = Execute seus próprios comandos após cada troca, por exemplo para controlar outro programa de papel de parede.
settings-automation-command-count = { $count ->
        [one] { $val }{ $count } comando{ $end } é executado após cada troca.
       *[other] { $val }{ $count } comandos{ $end } são executados após cada troca.
    }
settings-automation-commands-say = { $listed ->
        [none] { $val }Nenhum comando{ $end } é executado após cada troca.
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] O Skwd { $val }deixa a aplicação para seus comandos{ $end }, e os comandos { $val }também rodam na restauração da inicialização{ $end }.
           *[skip] O Skwd { $val }deixa a aplicação para seus comandos{ $end }, e os comandos { $val }pulam a restauração da inicialização{ $end }.
        }
       *[itself] { $rerun ->
            [also] O Skwd { $val }aplica o papel de parede sozinho{ $end }, e os comandos { $val }também rodam na restauração da inicialização{ $end }.
           *[skip] O Skwd { $val }aplica o papel de parede sozinho{ $end }, e os comandos { $val }pulam a restauração da inicialização{ $end }.
        }
    }
settings-automation-pick-only-hint = seus comandos aplicam
settings-automation-commands-heading = Comandos
settings-automation-placeholders = Use %path%, %thumb%, %type% e %name% em um comando.
settings-automation-minutes = { $count } min
settings-automation-mode-off = Desativado
settings-automation-mode-outdoors = Acompanhar o tempo lá fora
settings-automation-mode-playlists = Escolher das suas playlists
settings-automation-mode-semantic = Buscar na sua biblioteca
settings-automation-condition-clear = Limpo
settings-automation-condition-cloudy = Nublado
settings-automation-condition-rainy = Chuvoso
settings-automation-condition-snowy = Com neve
settings-automation-condition-stormy = Tempestuoso
settings-automation-condition-foggy = Com neblina
settings-automation-condition-windy = Com vento
settings-automation-display-another = Outro
settings-automation-display-exclude = Excluir
settings-automation-display-keep = Manter
settings-automation-playlists-heading = Playlist por clima
settings-automation-hours = { $count } h
