settings-colours-source = De onde vem a cor
settings-colours-source-desc = Seguir cada papel de parede, manter uma paleta ou deixar as cores como estão.
settings-colours-source-follow = seguem cada papel de parede
settings-colours-source-fixed = vêm de uma paleta fixa
settings-colours-source-off = ficam como estão
settings-colours-source-say = As cores { $policy }.
settings-colours-source-made = As cores { $policy }, criadas por { $authority }.
settings-colours-end4 = end-4
settings-colours-control-menu = Controle de cor · shells detectados
settings-colours-generator-menu = Gerador · só instalados
settings-colours-theme-menu = Tema · predefinidos, seus, sementes
settings-colours-all-presets = Todos os { $count } predefinidos…
settings-colours-generator = Gerador
settings-colours-generator-desc = O programa que transforma o papel de parede em paleta.
settings-colours-generator-say = { $engine } cria a paleta.
settings-colours-generator-seeded = { $engine } cria a paleta, com semente do espaço { $index } e contraste { $contrast }.
settings-colours-generator-shell = { $shell } cria a paleta a partir do papel de parede.
settings-colours-palette = Paleta
settings-colours-palette-desc = Uma paleta para todos os papéis de parede.
settings-colours-palette-say = { $theme } para todos os papéis de parede.
settings-colours-idle = Não é usado enquanto as cores ficam como estão.
settings-colours-index-hint = Qual espaço da paleta usar como semente
settings-colours-contrast-hint = -1 a 1
settings-colours-wallust-hint = Força uma paleta wallust, ex. dark16. Vazio escolhe pelo modo
settings-colours-seeds-hint = Usado por Sementes próprias
settings-colours-config-hint = Matugen externo
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = Automática
settings-colours-look = Aparência
settings-colours-look-desc = Quanto a cor se espalha, o acabamento e claro ou escuro.
settings-colours-look-say = Caráter { $scheme } com acabamento { $finish }, variante { $variant }.
settings-colours-look-imported = Paleta de { $shell }, importada como variante { $variant }.
settings-colours-look-shell = { $shell } define a própria aparência.
settings-colours-look-fixed = A paleta fixa define a própria aparência.
settings-colours-no-designer = O designer de temas está disponível com Seguir papel de parede ou Paleta fixa.
settings-colours-designer-hint = Ajuste ou crie uma paleta à mão
settings-colours-go = Para onde vão as cores
settings-colours-go-desc = Temas de apps que seguem a paleta quando ela muda.
settings-colours-find-hint = Verifique de novo após instalar um app
settings-colours-matugen-hint = Chave das saídas do Matugen
settings-colours-reload-hint = Executado após gravar o arquivo
settings-colours-output-heading = Saída { $count }
settings-colours-shells = Shells
settings-colours-shells-desc = Noctalia e Dank Material Shell podem usar as cores do Skwd.
settings-colours-dms-hint = O DMS precisa usar Auto (papel de parede)
settings-colours-mode-hint = Aplicado com cada esquema de cores
settings-colours-bin-hint = Se não estiver no PATH
settings-colours-lock = Telas de bloqueio
settings-colours-lock-desc = Telas de bloqueio que mostram seu papel de parede.
settings-colours-lock-follow = o último papel de parede aplicado
settings-colours-lock-still = uma imagem estática
settings-colours-sddm-hint = Exige aprovação de administrador
settings-colours-go-say = { $val }{ $on } de { $all } apps{ $end } { $followers ->
        [one] segue
       *[some] seguem
    } suas cores{ $changed ->
        [one] ; { $val }a configuração de { $name } mudou fora do Skwd{ $end }.
        [some] ; { $val }{ $count } apps mudaram fora do Skwd{ $end }.
       *[none] .
    }
settings-colours-outputs-say = { $state ->
        [none] Nenhuma integração gera um modelo ainda.
        [single] { $val }1 integração{ $end } gera um modelo em um arquivo após cada mudança de cor.
       *[many] { $val }{ $count } integrações{ $end } geram modelos em arquivos após cada mudança de cor.
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] segue as cores do Skwd
       *[keeps] mantém as próprias cores
    }{ $end }; DMS { $val }{ $dms ->
        [follows] segue as cores do Skwd
       *[keeps] mantém as próprias cores
    }{ $end }{ $caelestia ->
        [yes] ; os papéis de parede do Caelestia sincronizam.
       *[no] .
    }
settings-colours-lock-say = { $state ->
        [none] Nenhuma tela de bloqueio configurada. As novas usam { $mode }.
        [one] { $names } usa { $mode }.
       *[some] { $names } usam { $mode }.
    }
settings-colours-lock-say-static = { $state ->
        [none] Nenhuma tela de bloqueio configurada. As novas usam { $mode }: { $image }.
        [one] { $names } usa { $mode }: { $image }.
       *[some] { $names } usam { $mode }: { $image }.
    }
settings-menu-more = Mais
