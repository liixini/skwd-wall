settings-automation-rotation-desc = Aralık her dolduğunda havuzdan rastgele bir duvar kâğıdı uygular.
settings-automation-rotation-say = { $interval } aralıklarla, { $val }{ $pool }{ $end } arasından rastgele bir seçim.
settings-automation-rotation-off = Kapalı. Duvar kâğıdı yalnızca siz seçtiğinizde değişir.
settings-automation-pool = { $count ->
        [none] boş bir havuz
        [one] { $first }
        [two] { $first } ve { $second }
       *[three] { $first }, { $second } ve { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] favori { $pool }
       *[all] { $pool }
    }
settings-automation-pool-images = görüntüler
settings-automation-pool-videos = videolar
settings-automation-pool-engine = Wallpaper Engine öğeleri
settings-automation-pool-heading = Havuzda olanlar
settings-automation-timing = Zamanlama
settings-automation-interval-hint = en az 10 sn
settings-automation-schedule-desc = Kurallar duvar kâğıdını ve temayı saate, haftanın gününe, havaya, güce ve ekranlara göre seçer. İlk eşleşen kural kazanır.
settings-automation-schedule-say = { $start ->
        [applies] Kurallar açık; ilk eşleşen kural { $val }başlangıçta uygulanır{ $end }. Kendi seçtiğiniz duvar kâğıdı yine de kazanır.
       *[waits] Kurallar açık; ilk eşleşen kural { $val }bir sonraki değişikliği bekler{ $end }. Kendi seçtiğiniz duvar kâğıdı yine de kazanır.
    }
settings-automation-schedule-off = Kapalı. Bunu açana kadar kurallarınız saklanır.
settings-automation-start-hint = hizmet başlarken ilk eşleşen kural
settings-automation-editor-hint = koşul bloklarından kural oluşturun
settings-automation-location = Konum ve hava
settings-automation-location-desc = Gün doğumu ve gün batımı saatleri ile hava koşulları için kullanılır.
settings-automation-location-say = { $city } için hava durumu, { $lat }, { $lon } konumunda güneş saatleri. Hava durumuna göre değişim: { $mode }.
settings-automation-no-city = şehir yok
settings-automation-no-latitude = enlem yok
settings-automation-no-longitude = boylam yok
settings-automation-location-hint = Koordinatları kullanmak için şehri boş bırakın. Enlem kuzeye, boylam doğuya doğru pozitiftir.
settings-automation-startup = Başlangıç ve bildirimler
settings-automation-startup-desc = Hizmet başladığında ve duvar kâğıdı değiştiğinde olanlar.
settings-automation-startup-say = { $restore ->
        [restores] Başlangıçta Skwd { $val }son duvar kâğıdını yeniden uygular{ $end }.
       *[leaves] Başlangıçta Skwd { $val }masaüstünü olduğu gibi bırakır{ $end }.
    } { $notify ->
        [notifies] Duvar kâğıdı değişiklikleri { $val }bildirim gönderir{ $end }.
       *[silent] Duvar kâğıdı değişiklikleri { $val }sessiz kalır{ $end }.
    }
settings-automation-commands = Bir duvar kâğıdı uygulandıktan sonra
settings-automation-commands-desc = Her değişiklikten sonra kendi komutlarınızı çalıştırın, örneğin başka bir duvar kâğıdı aracını yönetmek için.
settings-automation-command-count = { $count ->
        [one] Her değişiklikten sonra { $val }{ $count } komut{ $end } çalışır.
       *[other] Her değişiklikten sonra { $val }{ $count } komut{ $end } çalışır.
    }
settings-automation-commands-say = { $listed ->
        [none] Her değişiklikten sonra { $val }hiçbir komut{ $end } çalışmaz.
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }uygulamayı komutlarınıza bırakır{ $end } ve komutlar { $val }başlangıçtaki geri yüklemede de çalışır{ $end }.
           *[skip] Skwd { $val }uygulamayı komutlarınıza bırakır{ $end } ve komutlar { $val }başlangıçtaki geri yüklemeyi atlar{ $end }.
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }duvar kâğıdını kendisi uygular{ $end } ve komutlar { $val }başlangıçtaki geri yüklemede de çalışır{ $end }.
           *[skip] Skwd { $val }duvar kâğıdını kendisi uygular{ $end } ve komutlar { $val }başlangıçtaki geri yüklemeyi atlar{ $end }.
        }
    }
settings-automation-pick-only-hint = komutlarınız uygular
settings-automation-commands-heading = Komutlar
settings-automation-placeholders = Bir komutta %path%, %thumb%, %type% ve %name% kullanın.
settings-automation-minutes = { $count } dk
settings-automation-mode-off = Kapalı
settings-automation-mode-outdoors = Dışarıdaki havayı izle
settings-automation-mode-playlists = Çalma listelerinden seç
settings-automation-mode-semantic = Kitaplığında ara
settings-automation-condition-clear = Açık
settings-automation-condition-cloudy = Bulutlu
settings-automation-condition-rainy = Yağmurlu
settings-automation-condition-snowy = Karlı
settings-automation-condition-stormy = Fırtınalı
settings-automation-condition-foggy = Sisli
settings-automation-condition-windy = Rüzgârlı
settings-automation-display-another = Başka
settings-automation-display-exclude = Hariç tut
settings-automation-display-keep = Koru
settings-automation-playlists-heading = Hava durumuna göre liste
settings-automation-hours = { $count } sa
