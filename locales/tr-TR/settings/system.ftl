settings-system-power-desc = Seçici ve duvar kâğıtlarının pil sınırları ayrıdır.
settings-system-power-say = Pildeyken { $cap ->
        [none] seçicinin ek bir sınırı yok ({ $fps })
       *[cap] seçici { $fps } ile sınırlanır
    }{ $idle ->
        [off] , yalnızca normal boştayken duraklatma uygulanır ({ $seconds })
       *[on] , girdi olmadan { $seconds } sonra video durur
    } ve { $detail ->
        [full] duvar kâğıtları tüm ayrıntıyı korur
       *[performance] duvar kâğıtları performans moduna geçer
    }.
settings-system-power-off = Kapalı. Pildeyken seçici ve duvar kâğıtları prize takılıykenki sınırlarla çalışır.
settings-system-power-source = Geçerli güç kaynağı: { $source }
settings-system-always = Her zaman
settings-system-on-battery = Pildeyken · tasarruf açıkken
settings-system-max-fps-hint = Ekran yenileme hızı ya da daha serin bir dizüstü için 60
settings-system-idle-hint = 0 kapatır
settings-system-battery-fps-hint = Bu ve en yüksek FPS'ten düşük olanı · 0 kapatır
settings-system-battery-idle-hint = Kısa olan geçerlidir · 0 kapatır
settings-system-battery-wallpaper-hint = Doğrudan video, 30 fps sahneler
settings-system-gpu-desc = Değiştirdikten sonra seçiciyi yeniden açın.
settings-system-gpu-say = { $mode ->
        [low] Seçici { $gpu } kullanıyor: tümleşik bir GPU ister.
        [high] Seçici { $gpu } kullanıyor: harici bir GPU ister.
        [none] Seçici { $gpu } kullanıyor: kararı sürücü verir.
        [auto] Seçici { $gpu } kullanıyor: pil tasarrufu etkinken düşük güçlü bir GPU'yu tercih eder.
       *[custom] Seçici { $gpu } kullanıyor, ancak bu değeri tanımadığı için Otomatik gibi davranıyor.
    }
settings-system-language-desc = Değişiklikler hemen uygulanır ve kaydedilir.
settings-system-language-say = { $follow ->
        [desktop] Arayüz dili { $language }; masaüstü dilinizi izler. Duvar kâğıdı adları ve etiketleri olduğu gibi kalır.
       *[chosen] Arayüz dili { $language }. Duvar kâğıdı adları ve etiketleri olduğu gibi kalır.
    }
settings-system-bug-title = Hata raporu
settings-system-bug-desc = Bir soruna eklenecek tek dosya.
settings-system-bug-say = Sürümü, ortamı ve son günlükleri tek bir dosyada toplayın.
