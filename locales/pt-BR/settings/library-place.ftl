settings-library-folders-desc = Verifica estas pastas e subpastas em busca de imagens e vídeos.
settings-library-folders-say = Verificando { $val }{ $folders }{ $end }.{ $state ->
        [polling] { " " }A sondagem cobre o que a vigilância nativa perde, a cada { $every }.{ $unread }
        [native] { " " }A vigilância nativa está ativa; a sondagem está parada.
        [down] { " " }A vigilância nativa falhou e a sondagem está desligada.
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }{ $count } pasta não pode ser lida.
       *[other] { " " }{ $count } pastas não podem ser lidas.
    }
settings-library-watching = Vigilância
settings-library-poll-hint = Para pastas de rede ou FUSE
settings-library-interval-hint = Reinicie o skwd-walld após alterar
settings-library-excluded-desc = Ignora estas pastas e subpastas. Os metadados existentes ficam salvos.
settings-library-excluded-say = Nunca verificando { $val }{ $folders }{ $end }.{ $outside }
settings-library-outside = { $count ->
        [one] { " " }{ $count } está fora de toda pasta de papéis de parede e não ignora nada.
       *[other] { " " }{ $count } estão fora de toda pasta de papéis de parede e não ignoram nada.
    }
settings-library-excluded-hint = Pastas excluídas nunca são verificadas, então nenhum perfil pode mostrá-las. Para manter uma pasta mas ocultá-la em público, use { $link }.
settings-library-profiles-desc = Decida quais pastas o seletor, o filtro de pastas, os papéis aleatórios e os agendamentos usam.
settings-library-profiles-say = Perfil ativo { $state ->
        [none] { $menu }, que mostra todas as pastas.
       *[some] { $menu }: { $summary }.
    } Ctrl+H ou o botão de ocultos da barra mostra os papéis ocultos, inclusive os ocultados pelo cartão, até a próxima abertura.
settings-library-active = Perfil ativo
settings-library-terminal = Trocar por um terminal
settings-library-tree-note = Ocultar fica fora do aleatório até Ctrl+H. Excluir fica fora mesmo com Ctrl+H. Uma pasta define suas subpastas.
settings-library-show-tip = No seletor e no aleatório
settings-library-hide-tip = Até Ctrl+H, nunca aleatório
settings-library-exclude-tip = Fora deste perfil, Ctrl+H ignora
settings-library-from = De { $link }
settings-library-excluded-inside = { $count } excluídas dentro
settings-library-both-inside = { $hidden } ocultas, { $excluded } excluídas dentro
settings-library-storage = Armazenamento e lixeira
settings-library-storage-desc = Cópias WebP menores, armazenamento de recuperação e cópias recoloridas.
settings-library-storage-say = { $optimise ->
        [on] Imagens novas viram WebP com qualidade { $quality }, até { $size }.
       *[off] Imagens novas ficam como estão. Otimizar tudo usa qualidade { $quality }, até { $size }.
    } { $trash ->
        [on] Originais na lixeira ficam por { $days } e depois são apagados.
       *[off] Originais na lixeira ficam até você apagá-los.
    } { $recolour ->
        [on] Papéis novos também ganham uma cópia recolorida.
       *[off] Papéis novos não são recoloridos.
    }
settings-library-retention-hint = Dias para manter imagens na lixeira
settings-library-optimise-hint = Otimiza todas as imagens da biblioteca
settings-library-recolour = Recolorir
settings-library-thumbnails = Miniaturas e reparo
settings-library-thumbnails-desc = Refaça prévias e dados de cor quando algo parecer errado.
settings-library-thumbnails-say = Até { $jobs } miniaturas são criadas de uma vez nas reconstruções.
settings-library-rebuild = Reconstruir
settings-library-start-over = Recomeçar
settings-library-recompute-hint = A partir das miniaturas existentes
settings-library-clear-hint = As miniaturas voltam na próxima verificação
