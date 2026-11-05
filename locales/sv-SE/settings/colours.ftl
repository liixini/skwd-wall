settings-colours-source = Var färgen kommer ifrån
settings-colours-source-desc = Följ varje bakgrund, behåll en palett eller låt färgerna vara.
settings-colours-source-follow = följer varje bakgrund
settings-colours-source-fixed = kommer från en fast palett
settings-colours-source-off = lämnas som de är
settings-colours-source-say = Färgerna { $policy }.
settings-colours-source-made = Färgerna { $policy }, skapade av { $authority }.
settings-colours-end4 = end-4
settings-colours-control-menu = Färgstyrning · hittade skal
settings-colours-generator-menu = Generator · endast installerade
settings-colours-theme-menu = Tema · förval, dina, egna frön
settings-colours-all-presets = Alla { $count } förval…
settings-colours-generator = Generator
settings-colours-generator-desc = Programmet som gör bakgrunden till en palett.
settings-colours-generator-say = { $engine } skapar paletten.
settings-colours-generator-seeded = { $engine } skapar paletten, med frö från plats { $index } och kontrast { $contrast }.
settings-colours-generator-shell = { $shell } skapar paletten från bakgrunden.
settings-colours-palette = Palett
settings-colours-palette-desc = En palett för alla bakgrunder.
settings-colours-palette-say = { $theme } för alla bakgrunder.
settings-colours-idle = Används inte medan färgerna lämnas som de är.
settings-colours-index-hint = Vilken palettplats som blir fröfärg
settings-colours-contrast-hint = -1 till 1
settings-colours-wallust-hint = Tvinga en wallust-palett, t.ex. dark16. Tomt väljer efter läge
settings-colours-seeds-hint = Används av Egna frön
settings-colours-config-hint = Extern Matugen
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = Auto
settings-colours-look = Utseende
settings-colours-look-desc = Hur starkt färgen sprids, dess finish och ljust eller mörkt.
settings-colours-look-say = Karaktären { $scheme } med finishen { $finish }, variant { $variant }.
settings-colours-look-imported = Paletten från { $shell }, importerad som varianten { $variant }.
settings-colours-look-shell = { $shell } bestämmer sitt eget utseende.
settings-colours-look-fixed = Den fasta paletten bestämmer sitt eget utseende.
settings-colours-no-designer = Temadesignern finns med Följ bakgrund eller Fast palett.
settings-colours-designer-hint = Justera eller bygg en palett för hand
settings-colours-go = Vart färgerna går
settings-colours-go-desc = Appteman som följer paletten när den ändras.
settings-colours-find-hint = Kontrollera igen efter att du installerat en app
settings-colours-matugen-hint = Funktionsbrytare för Matugen-utdata
settings-colours-reload-hint = Körs när filen har skrivits
settings-colours-output-heading = Utdata { $count }
settings-colours-shells = Skal
settings-colours-shells-desc = Noctalia och Dank Material Shell kan ta emot Skwds färger.
settings-colours-dms-hint = DMS måste använda Auto (bakgrund)
settings-colours-mode-hint = Används med varje färgschema
settings-colours-bin-hint = Om den inte finns i PATH
settings-colours-lock = Låsskärmar
settings-colours-lock-desc = Låsskärmar som visar din bakgrund.
settings-colours-lock-follow = den senast använda bakgrunden
settings-colours-lock-still = en statisk bild
settings-colours-sddm-hint = Kräver administratörsgodkännande
settings-colours-go-say = { $val }{ $on } av { $all } appar{ $end } { $followers ->
        [one] följer
       *[some] följer
    } dina färger{ $changed ->
        [one] ; { $val }konfigurationen för { $name } ändrades utanför Skwd{ $end }.
        [some] ; { $val }{ $count } appar ändrades utanför Skwd{ $end }.
       *[none] .
    }
settings-colours-outputs-say = { $state ->
        [none] Ingen integration renderar någon mall ännu.
        [single] { $val }1 integration{ $end } renderar en mall till en fil efter varje färgbyte.
       *[many] { $val }{ $count } integrationer{ $end } renderar mallar till filer efter varje färgbyte.
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] följer Skwds färger
       *[keeps] behåller sina egna färger
    }{ $end }; DMS { $val }{ $dms ->
        [follows] följer Skwds färger
       *[keeps] behåller sina egna färger
    }{ $end }{ $caelestia ->
        [yes] ; Caelestias bakgrunder synkas.
       *[no] .
    }
settings-colours-lock-say = { $state ->
        [none] Ingen låsskärm är inställd. Nya använder { $mode }.
        [one] { $names } använder { $mode }.
       *[some] { $names } använder { $mode }.
    }
settings-colours-lock-say-static = { $state ->
        [none] Ingen låsskärm är inställd. Nya använder { $mode }: { $image }.
        [one] { $names } använder { $mode }: { $image }.
       *[some] { $names } använder { $mode }: { $image }.
    }
settings-menu-more = Mer
