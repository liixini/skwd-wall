settings-system-power-desc = Väljaren och bakgrunderna har separata batterigränser.
settings-system-power-say = På batteri { $cap ->
        [none] har väljaren ingen extra gräns ({ $fps })
       *[cap] begränsas väljaren till { $fps }
    }{ $idle ->
        [off] , bara den vanliga vilopausen gäller ({ $seconds })
       *[on] , video stannar efter { $seconds } utan inmatning
    } och { $detail ->
        [full] bakgrunderna behåller full detaljnivå
       *[performance] bakgrunderna går över till prestandaläge
    }.
settings-system-power-off = Av. På batteri kör väljaren och bakgrunderna med samma gränser som på nätström.
settings-system-power-source = Aktuell strömkälla: { $source }
settings-system-always = Alltid
settings-system-on-battery = På batteri · när sparläget är på
settings-system-max-fps-hint = Skärmens frekvens, eller 60 för en svalare dator
settings-system-idle-hint = 0 stänger av
settings-system-battery-fps-hint = Lägst av detta och Max FPS · 0 stänger av
settings-system-battery-idle-hint = Kortast vinner · 0 stänger av
settings-system-battery-wallpaper-hint = Direkt video, scener i 30 fps
settings-system-gpu-desc = Öppna väljaren igen efter en ändring.
settings-system-gpu-say = { $mode ->
        [low] Väljaren använder { $gpu }: den ber om en integrerad GPU.
        [high] Väljaren använder { $gpu }: den ber om en separat GPU.
        [none] Väljaren använder { $gpu }: drivrutinen bestämmer.
        [auto] Väljaren använder { $gpu }: den föredrar en strömsnål GPU när batterisparläget är aktivt.
       *[custom] Väljaren använder { $gpu }, som den inte känner igen, så den beter sig som Auto.
    }
settings-system-language-desc = Ändringar gäller direkt och sparas.
settings-system-language-say = { $follow ->
        [desktop] Gränssnittet är på { $language }, som följer skrivbordet. Bakgrundernas namn och taggar ändras inte.
       *[chosen] Gränssnittet är på { $language }. Bakgrundernas namn och taggar ändras inte.
    }
settings-system-bug-title = Felrapport
settings-system-bug-desc = En fil att bifoga till ett ärende.
settings-system-bug-say = Samla version, miljö och senaste loggar i en fil.
