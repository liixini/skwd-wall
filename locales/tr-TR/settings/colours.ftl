settings-colours-source = Renk nereden geliyor
settings-colours-source-desc = Her duvar kâğıdını izle, tek palet kullan ya da renklere dokunma.
settings-colours-source-follow = her duvar kâğıdını izliyor
settings-colours-source-fixed = tek bir sabit paletten geliyor
settings-colours-source-off = olduğu gibi kalıyor
settings-colours-source-say = Renkler { $policy }.
settings-colours-source-made = Renkler { $policy }, { $authority } üretiyor.
settings-colours-end4 = end-4
settings-colours-control-menu = Renk denetimi · algılanan kabuklar
settings-colours-generator-menu = Üreteç · yalnızca kurulu
settings-colours-theme-menu = Tema · hazır, sizin, özel tohumlar
settings-colours-all-presets = { $count } hazır temanın tümü…
settings-colours-generator = Üreteç
settings-colours-generator-desc = Duvar kâğıdını palete çeviren program.
settings-colours-generator-say = Paleti { $engine } üretiyor.
settings-colours-generator-seeded = Paleti { $engine } üretiyor, tohum yuvası { $index }, karşıtlık { $contrast }.
settings-colours-generator-shell = { $shell } paleti duvar kâğıdından üretiyor.
settings-colours-palette = Palet
settings-colours-palette-desc = Tüm duvar kâğıtları için tek palet.
settings-colours-palette-say = Tüm duvar kâğıtları için { $theme }.
settings-colours-idle = Renkler olduğu gibi kalırken kullanılmaz.
settings-colours-index-hint = Tohum renk olarak kullanılacak palet yuvası
settings-colours-contrast-hint = -1 ile 1 arası
settings-colours-wallust-hint = Bir wallust paleti zorla, ör. dark16. Boş: moda göre
settings-colours-seeds-hint = Özel tohumlar kullanır
settings-colours-config-hint = Harici Matugen
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = Otomatik
settings-colours-look = Görünüm
settings-colours-look-desc = Rengin ne kadar yayıldığı, bitişi, açık ya da koyu.
settings-colours-look-say = { $scheme } karakteri, { $finish } bitiş, { $variant } varyant.
settings-colours-look-imported = { $shell } paleti, { $variant } varyant olarak alındı.
settings-colours-look-shell = { $shell } görünümünü kendi belirliyor.
settings-colours-look-fixed = Sabit palet görünümünü kendi belirliyor.
settings-colours-no-designer = Tema tasarımcısı Duvar kâğıdını izle veya Sabit palet ile kullanılabilir.
settings-colours-designer-hint = Paleti elle ayarlayın veya oluşturun
settings-colours-go = Renkler nereye gidiyor
settings-colours-go-desc = Palet değişince onu izleyen uygulama temaları.
settings-colours-find-hint = Uygulama kurduktan sonra yeniden denetleyin
settings-colours-matugen-hint = Matugen çıktıları anahtarı
settings-colours-reload-hint = Dosya yazıldıktan sonra çalışır
settings-colours-output-heading = Çıktı { $count }
settings-colours-shells = Kabuklar
settings-colours-shells-desc = Noctalia ve Dank Material Shell Skwd renklerini alabilir.
settings-colours-dms-hint = DMS Auto (duvar kâğıdı) kullanmalı
settings-colours-mode-hint = Her renk şemasıyla uygulanır
settings-colours-bin-hint = PATH içinde değilse
settings-colours-lock = Kilit ekranları
settings-colours-lock-desc = Duvar kâğıdınızı gösteren kilit ekranları.
settings-colours-lock-follow = son uygulanan duvar kâğıdı
settings-colours-lock-still = sabit bir görsel
settings-colours-sddm-hint = Yönetici onayı gerekir
settings-colours-go-say = { $val }{ $all } uygulamadan { $on } tanesi{ $end } { $followers ->
        [one] renklerinizi
       *[some] renklerinizi
    } izliyor{ $changed ->
        [one] ; { $val }{ $name } yapılandırması Skwd dışında değişti{ $end }.
        [some] ; { $val }{ $count } uygulama Skwd dışında değişti{ $end }.
       *[none] .
    }
settings-colours-outputs-say = { $state ->
        [none] Henüz hiçbir tümleşim şablon oluşturmuyor.
        [single] { $val }1 tümleşim{ $end } her renk değişiminden sonra bir şablonu dosyaya yazar.
       *[many] { $val }{ $count } tümleşim{ $end } her renk değişiminden sonra şablonları dosyalara yazar.
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] Skwd renklerini izliyor
       *[keeps] kendi renklerini koruyor
    }{ $end }; DMS { $val }{ $dms ->
        [follows] Skwd renklerini izliyor
       *[keeps] kendi renklerini koruyor
    }{ $end }{ $caelestia ->
        [yes] ; Caelestia duvar kâğıtları eşleniyor.
       *[no] .
    }
settings-colours-lock-say = { $state ->
        [none] Kilit ekranı ayarlı değil. Yeniler { $mode } kullanır.
        [one] { $names }, { $mode } kullanıyor.
       *[some] { $names }, { $mode } kullanıyor.
    }
settings-colours-lock-say-static = { $state ->
        [none] Kilit ekranı ayarlı değil. Yeniler { $mode } kullanır: { $image }.
        [one] { $names }, { $mode } kullanıyor: { $image }.
       *[some] { $names }, { $mode } kullanıyor: { $image }.
    }
settings-menu-more = Daha fazla
