settings-home-search = Tüm ayarlarda ara: kare hızı, ses, kilit ekranı, bulanıklık
settings-home-displays = { $count ->
    [one] { $count } ekran
   *[other] { $count } ekran
 }
settings-home-displays-say = { $link } { $count ->
    [one] duvar kâğıdı gösteriyor.
   *[other] duvar kâğıdı gösteriyor.
 }
settings-home-watching = { " " }Kitaplık izleme: { $val }{ $state }{ $end }.
settings-home-model-ready = hazır
settings-home-model-missing = model yok
settings-home-model-off = kapalı
settings-home-colours-from = { " " }{ $link } kaynağı: { $val }{ $engine }{ $end }.
settings-home-sound-desc = Video ve Wallpaper Engine duvar kâğıtlarının sesi.
settings-home-sound-say = Duvar kâğıtları { $volume } düzeyinde çalar{ $duck ->
    [yes] { " " }ve başka uygulamalar ses çaldığında duraklar.
   *[no] , başka uygulamalar ses çalarken bile.
 }
settings-home-colours-desc = Duvar kâğıdı renkleri masaüstünün geri kalanına nasıl ulaşır.
settings-home-colours-say = { settings-theme-policy-label }: { $policy }.{ " " }Daha fazlası: { $place }.
settings-home-rotation-desc = Aralık her dolduğunda rastgele bir duvar kâğıdı uygular.
settings-home-rotation-say = Her { $interval } rastgele bir duvar kâğıdı.
settings-home-rotation-off = Döndürme kapalı. Duvar kâğıdı yalnızca siz seçtiğinizde değişir.
settings-home-transitions-desc = Bir duvar kâğıdı sonrakine nasıl geçer.
settings-home-transitions-say = Bir duvar kâğıdı { $duration } içinde sonrakine geçer.
settings-home-transitions-off = Geçişler kapalı. Duvar kâğıtları doğrudan sonrakine geçer.
settings-home-changed = Varsayılandan değiştirilen
settings-home-changed-desc = Varsayılandan değiştirdiğiniz ayarlar tek yerde, her biri Sıfırla ile.
settings-home-changed-none = Burada karşılaştırılan ayarlar arasında varsayılandan farklı olan yok.
settings-home-changed-say = { $count ->
    [one] { $count } ayar varsayılanından farklı.
   *[other] { $count } ayar varsayılanlarından farklı.
 }
settings-home-review = İncele
settings-home-reset = Sıfırla
settings-home-reset-all = Tümünü sıfırla
settings-home-badge = { $count } değişti
settings-home-automatic = Otomatik
settings-home-displays-none = Hiçbir ekran
settings-home-displays-none-say = { $link } duvar kâğıdı göstermiyor.
settings-home-reset-scope = Yukarıdaki tüm ayarları sıfırlar, klavye kısayolları dahil. API anahtarları, kullanıcı adları, konumlar ve klasörler kalır.
settings-home-items = { $count ->
    [one] { $count } öge
   *[other] { $count } öge
 }
