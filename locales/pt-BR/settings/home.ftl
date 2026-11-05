settings-home-search = Pesquise qualquer configuração: taxa de quadros, som, tela de bloqueio, desfoque
settings-home-displays = { $count ->
    [one] { $count } tela
   *[other] { $count } telas
 }
settings-home-displays-say = { $link } { $count ->
    [one] está mostrando papéis de parede.
   *[other] estão mostrando papéis de parede.
 }
settings-home-watching = { " " }Monitoramento da biblioteca: { $val }{ $state }{ $end }.
settings-home-model-ready = pronto
settings-home-model-missing = sem modelo
settings-home-model-off = desativado
settings-home-colours-from = { " " }{ $link } vêm de { $val }{ $engine }{ $end }.
settings-home-sound-desc = Áudio de papéis de parede em vídeo e do Wallpaper Engine.
settings-home-sound-say = Os papéis de parede tocam a { $volume }{ $duck ->
    [yes] { " " }e pausam quando outros apps tocam som.
   *[no] , mesmo quando outros apps tocam som.
 }
settings-home-colours-desc = Como as cores do papel de parede chegam ao resto da área de trabalho.
settings-home-colours-say = { settings-theme-policy-label }: { $policy }.{ " " }Mais em { $place }.
settings-home-rotation-desc = Aplica um papel de parede aleatório sempre que o intervalo termina.
settings-home-rotation-say = Um papel de parede aleatório a cada { $interval }.
settings-home-rotation-off = A rotação está desativada. O papel de parede só muda quando você escolhe um.
settings-home-transitions-desc = Como um papel de parede passa para o próximo.
settings-home-transitions-say = Um papel de parede passa para o próximo em { $duration }.
settings-home-transitions-off = As transições estão desativadas. Os papéis de parede cortam direto para o próximo.
settings-home-changed = Alterado do padrão
settings-home-changed-desc = As configurações que você alterou, em um só lugar, cada um com Redefinir.
settings-home-changed-none = Nada difere do padrão entre as configurações comparadas aqui.
settings-home-changed-say = { $count ->
    [one] { $count } configuração difere do padrão.
   *[other] { $count } configurações diferem do padrão.
 }
settings-home-review = Revisar
settings-home-reset = Redefinir
settings-home-reset-all = Redefinir tudo
settings-home-badge = { $count } alterados
settings-home-automatic = Automático
settings-home-displays-none = Nenhuma tela
settings-home-displays-none-say = { $link } está mostrando papéis de parede.
settings-home-reset-scope = Redefine todas as configurações acima, inclusive os atalhos de teclado. Chaves de API, nomes de usuário, locais e pastas permanecem.
settings-home-items = { $count ->
    [one] { $count } item
   *[other] { $count } itens
 }
