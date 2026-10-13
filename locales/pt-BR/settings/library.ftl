settings-library-watch-section-desc = Detecta arquivos adicionados fora do skwd-wall e mantém a biblioteca atualizada.
settings-library-watch-fallback-label = Varredura periódica de reserva
settings-library-watch-fallback-desc = Verifica apenas as pastas da biblioteca que o monitoramento nativo de arquivos não consegue acompanhar. Ative para montagens de rede ou FUSE que perdem alterações e depois reinicie o skwd-walld.
settings-library-watch-interval-label = Intervalo da varredura
settings-library-watch-interval-desc = Aguarda esta quantidade de segundos entre verificações limitadas. Valores menores encontram alterações antes, mas leem o sistema de arquivos com mais frequência. Reinicie o skwd-walld depois de mudar.
settings-library-watch-unknown-label = Estado do monitoramento indisponível
settings-library-watch-unknown-desc = Este daemon não informa o estado do monitoramento da biblioteca. Atualize ou reinicie o skwd-walld.
settings-library-watch-poll-failed-label = A varredura não consegue ler uma pasta da biblioteca
settings-library-watch-poll-failed-desc = Verifique se todas as pastas configuradas da biblioteca estão montadas e legíveis. A varredura tentará de novo em { $interval } segundos.
settings-library-watch-polling-label = Varredura de reserva ativa
settings-library-watch-polling-desc = O monitoramento nativo falhou em { $count ->
    [one] { $count } pasta da biblioteca
   *[other] { $count } pastas da biblioteca
    }. Até { $budget } entradas são verificadas a cada { $interval } segundos. Última convergência bem-sucedida: { $convergence }.
settings-library-watch-recovering-label = Monitoramento nativo recuperado
settings-library-watch-recovering-desc = O monitoramento nativo está ativo de novo. Uma varredura completa de transição ainda está em andamento antes de a biblioteca ser considerada atualizada.
settings-library-watch-unavailable-label = Monitoramento da biblioteca indisponível
settings-library-watch-unavailable-desc = O monitoramento nativo de arquivos falhou e a varredura de reserva está desativada. Ative a Varredura periódica de reserva e depois reinicie o skwd-walld.
settings-library-watch-recovered-label = Monitoramento nativo restaurado
settings-library-watch-recovered-desc = O monitoramento nativo e a varredura de transição estão atualizados. Última convergência bem-sucedida: { $convergence }.
settings-library-watch-native-label = Monitoramento nativo de arquivos
settings-library-watch-native-desc = Eventos do sistema de arquivos estão ativos para todas as pastas da biblioteca. A varredura está ociosa.
settings-library-watch-convergence-never = Ainda não concluída
settings-library-watch-convergence-seconds = { $value ->
    [one] há { $value } segundo
   *[other] há { $value } segundos
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] há { $value } minuto
   *[other] há { $value } minutos
    }
settings-library-watch-convergence-hours = { $value ->
    [one] há { $value } hora
   *[other] há { $value } horas
    }
settings-profiles-card = Perfis
settings-profiles-card-desc = Escolha quais pastas o seletor, o filtro de pastas, os papéis de parede aleatórios e as programações usam. Pastas ocultas voltam com o botão de ocultos na barra ou seu atalho de teclado, e cada abertura começa com elas ocultas de novo.
settings-profiles-everything = Tudo
settings-profiles-name-label = Nome
settings-profiles-name-placeholder = Nome do perfil
settings-profiles-state-show = Mostrar
settings-profiles-state-hide = Ocultar
settings-profiles-state-exclude = Excluir
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = Itens do Workshop não têm pastas, então isto vale para todos. Oculte um papel de parede específico pelo verso do cartão.
settings-profiles-summary = { $hidden } ocultas, { $excluded } excluídas
settings-profiles-summary-none = Mostra todas as pastas
settings-profiles-delete-action = Excluir perfil
settings-profiles-add-label = Novo perfil
settings-profiles-new-name = Novo perfil
settings-profiles-tree-label = Pastas deste perfil
settings-profiles-tree-desc = Mostrar mantém a pasta no seletor e no aleatório. Ocultar a mantém fora de vista até o botão de ocultos na barra ou seu atalho. Excluir a deixa fora do perfil, então o botão a ignora. Uma pasta também define suas subpastas; altere uma subpasta depois para abrir uma exceção.
settings-profiles-never-scanned = Nunca verificada
settings-profiles-mixed = Misto dentro
settings-profiles-hidden-inside = { $count ->
    [one] { $count } oculta dentro
   *[other] { $count } ocultas dentro
    }
settings-profiles-copy-action = Copiar
settings-profiles-helm-desc = Torna este o perfil ativo, então a rotação e as programações o seguem.
settings-profiles-flag-desc = Abre um seletor neste perfil e deixa o ativo como está.
settings-profiles-make-active = Ativar
settings-profiles-active-tag = Ativo
settings-profiles-builtin-tag = Integrado
settings-profiles-filter-placeholder = Filtrar pastas
settings-profiles-no-match = Nenhuma pasta corresponde.
settings-profiles-switch-label = Mudar para este perfil
settings-profiles-switch-desc = skwd-helm muda o perfil salvo, então a rotação e os agendamentos o seguem. A opção de inicialização abre um seletor neste perfil e deixa o salvo intacto.
settings-profiles-hidden-note-label = Papéis de parede ocultos
settings-profiles-hidden-note-desc = O botão de ocultos na barra (Ctrl+H) mostra as pastas ocultas do perfil ativo e cada papel de parede ocultado pelo verso do cartão. Cada inicialização os oculta de novo.
settings-profiles-tree-note = Nunca verificado vem das pastas excluídas em Biblioteca › Pastas, então nenhum perfil pode mostrá-lo. Excluir aqui só deixa uma pasta fora deste perfil. Papéis de parede ocultados pelo cartão ficam ocultos em todos os perfis, Tudo incluído, até Mostrar ocultos. Pastas com ponto como .private são pastas comuns aqui.
settings-profiles-legend-show = no seletor e no aleatório.
settings-profiles-legend-hide = até o botão de ocultos na barra ou Ctrl+H. Nunca aleatório.
settings-profiles-legend-exclude = deste perfil. Mostrar ocultos o ignora.
settings-profiles-everything-desc = Tudo mostra cada pasta verificada e cada item do Workshop. Crie um novo perfil para ocultar ou excluir alguns.
settings-profiles-delete-hint = Clique de novo para excluir. Papéis de parede e pastas ficam onde estão.
