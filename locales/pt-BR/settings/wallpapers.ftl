settings-wallpapers-displays-desc = O que cada tela mostra e sua posição.
settings-wallpapers-play = Como os papéis de parede tocam
settings-wallpapers-play-desc = Motores, a camada deles e cenas.
settings-wallpapers-transitions-desc = Como um papel de parede muda para o próximo.
settings-wallpapers-awww-desc = A transição do awww para imagens.
settings-wallpapers-drawing = Desenho
settings-wallpapers-drawing-desc = Papéis preparados se desenham na tela.
settings-wallpapers-sound = Som
settings-wallpapers-sound-desc = Volume, e quando o áudio do papel dá lugar.
settings-wallpapers-pause-desc = Pausa papéis animados por regra.
settings-wallpapers-plasma = Tela de bloqueio do Plasma
settings-wallpapers-plasma-desc = Qual papel o Plasma mostra ao bloquear.
settings-wallpapers-niri = Visão geral do Niri
settings-wallpapers-niri-desc = No niri. Um fundo atrás da visão geral.
settings-wallpapers-display-more = Mais
settings-wallpapers-lock = Bloquear { $name }
settings-wallpapers-lock-hint = Só atualiza pelo multisseletor
settings-wallpapers-layer-bottom = Inferior
settings-wallpapers-gpu-hint = Volta para Automático
settings-wallpapers-timeout-hint = Para imagens muito grandes
settings-wallpapers-animated = Papéis animados
settings-wallpapers-scenes = Cenas
settings-wallpapers-scenes-hint = Limite de FPS · escala
settings-wallpapers-particles-hint = Só cenas
settings-wallpapers-preview-hint = Taxa de quadros
settings-wallpapers-quality-short = A areia precisa de pelo menos { $ms } ms.
settings-wallpapers-quality-hint = Full sem limite, Low limitado a 30 fps
settings-wallpapers-sharpen-hint = Quando a qualidade reduz a resolução
settings-wallpapers-monitors = Telas
settings-wallpapers-primary-hint = Anima só com Principal
settings-wallpapers-primary-blank = a primeira disponível
settings-wallpapers-primary-only-hint = O resto usa todas as telas
settings-wallpapers-wave = Onda
settings-wallpapers-wave-hint = Largura × altura
settings-wallpapers-position-hint = center, top-left ou 0.5,0.5
settings-wallpapers-bezier-hint = x1,y1,x2,y2
settings-wallpapers-bezier-blank = padrão do awww
settings-wallpapers-invert-hint = Inverte o eixo Y da origem
settings-wallpapers-colours-files = Cores e arquivos
settings-wallpapers-preparing = Preparação
settings-wallpapers-paper-ink = Papel e tinta
settings-wallpapers-prepare = Preparar
settings-wallpapers-prepare-all = Todos os papéis
settings-wallpapers-scope-view = Vista atual · { $count }
settings-wallpapers-pack-ready = Arquivos de processamento prontos.
settings-wallpapers-pack-none = Ainda sem arquivos de processamento.
settings-wallpapers-pack-blank = Nenhum ainda
settings-wallpapers-drawing-blocked = Desenho indisponível. Escolha primeiro os arquivos de processamento.
settings-wallpapers-volume = Volume
settings-wallpapers-other-apps = Outros apps
settings-wallpapers-pause-apps = Pausa enquanto os apps da lista rodam.
settings-wallpapers-pause-none = Nenhuma regra de pausa ativa.
settings-wallpapers-process-hint = Aceita * e ?
settings-wallpapers-process-action = Escolher
settings-wallpapers-process-blank = nenhum ainda
settings-wallpapers-resume-hint = Depois que a última regra termina
settings-wallpapers-scope-all = todas as telas
settings-wallpapers-scope-display = a tela da janela
settings-wallpapers-rule-fullscreen = janelas em tela cheia
settings-wallpapers-rule-maximised = janelas maximizadas
settings-wallpapers-rule-focused = janelas em foco
settings-wallpapers-rule-full-width = colunas de largura total
settings-wallpapers-plasma-left = fica a cargo do Plasma
settings-wallpapers-plasma-static = mostra uma imagem fixa
settings-wallpapers-plasma-follow = segue o último papel aplicado
settings-wallpapers-plasma-off-row = Sem gestão deixa o Plasma como está.
settings-wallpapers-image-hint = Caminho absoluto, ou começando com ~/
settings-wallpapers-image-blank = sem imagem
settings-wallpapers-dynamic-hint = O quadro é uma imagem muda
settings-wallpapers-niri-off = Desativada. A visão geral mantém o fundo do niri.
settings-wallpapers-source = Origem
settings-wallpapers-look = Aparência
settings-wallpapers-source-blank = último papel
settings-wallpapers-blur = Desfocar o fundo
settings-wallpapers-radii = Raio do desfoque
settings-wallpapers-radii-hint = Imagens · vídeos · cenas
settings-wallpapers-dim-hint = 0 nada, 100 preto
settings-wallpapers-transitions-off = Desativadas. Cada papel corta direto para o próximo.
settings-wallpapers-transitions-off-foot = Ative as transições para escolher a prévia, as telas e a qualidade da areia.
settings-wallpapers-awww-none = As imagens mudam com { $type }.
settings-wallpapers-awww-no-type = nenhuma transição
settings-wallpapers-sound-muted = O áudio dos papéis está mudo.
settings-wallpapers-displays-say = { $state ->
        [none] Nenhuma tela bloqueada.
        [one] { $names } está bloqueada.
        [some] { $names } estão bloqueadas.
       *[detecting] Detectando telas.
    } Sem posição própria, as telas usam { $fill }.
settings-wallpapers-play-say = Imagens com { $backend }, vídeos com { $val }Vulkan{ $end }, na camada { $layer }.
settings-wallpapers-transitions-say = Família { $family }{ $kind ->
        [random] , diferente a cada vez, em { $duration }.
        [preset] , com { $shader }, em { $ms } ms.
       *[shader] , com { $shader }, em { $duration }.
    } Taxa { $rate }.
settings-wallpapers-primary-count = { $state ->
        [none] nenhuma de { $all }
        [all] todas as { $all }
       *[some] { $shown } de { $all }
    }
settings-wallpapers-awww-say = Imagens com { $type } e filtro { $filter }, em { $duration }.
settings-wallpapers-drawing-say = { $state ->
        [on] Papéis preparados se desenham em { $duration }.
       *[off] Desativado. Cartões com Desenho levam { $duration }.
    }
settings-wallpapers-sound-say = O áudio dos papéis toca a { $volume }{ $duck ->
        [yes] { " " }e pausa quando outro app toca som.
       *[no] .
    }
settings-wallpapers-pause-say = Pausa com { $rules } em { $scope }{ $apps ->
        [yes] , e com apps da lista.
       *[no] .
    }
settings-wallpapers-pause-rules = { $count } regras
settings-wallpapers-plasma-say = { $state ->
        [off] A tela de bloqueio { $mode }.
        [static] A tela de bloqueio { $mode }: { $image }.
       *[follow] A tela de bloqueio { $mode }, cenas como { $scenes ->
        [live] render ao vivo.
       *[poster] quadro fixo.
    }
    }
settings-wallpapers-niri-say = Mostra { $source ->
       *[current] o papel atual
        [own] sua própria origem
        [stationary] um papel fixo
    }{ $blur ->
        [yes] , desfocado
       *[no] { "" }
    }, escurecido { $dim }{ $auto ->
        [yes] , recolorido
       *[no] { "" }
    }.
settings-wallpapers-device-hint = Automático tenta CUDA, WebGPU e depois CPU
settings-wallpapers-memory-hint = Acima do limite, Automático tenta na CPU
