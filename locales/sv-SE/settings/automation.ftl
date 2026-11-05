settings-automation-rotation-desc = Använd en slumpmässig bakgrund ur poolen varje gång intervallet löper ut.
settings-automation-rotation-say = Med { $interval } mellanrum, ett slumpmässigt val bland { $val }{ $pool }{ $end }.
settings-automation-rotation-off = Av. Bakgrunden byts bara när du väljer en själv.
settings-automation-pool = { $count ->
        [none] en tom pool
        [one] { $first }
        [two] { $first } och { $second }
       *[three] { $first }, { $second } och { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] favoritmärkta { $pool }
       *[all] { $pool }
    }
settings-automation-pool-images = bilder
settings-automation-pool-videos = videor
settings-automation-pool-engine = Wallpaper Engine-objekt
settings-automation-pool-heading = Poolen innehåller
settings-automation-timing = Tidsinställning
settings-automation-interval-hint = minst 10 s
settings-automation-schedule-desc = Regler väljer bakgrund och tema utifrån tid, veckodag, väder, ström och skärmar. Den första matchande regeln vinner.
settings-automation-schedule-say = { $start ->
        [applies] Reglerna är på; den första matchande regeln { $val }tillämpas vid start{ $end }. En bakgrund du väljer själv vinner ändå.
       *[waits] Reglerna är på; den första matchande regeln { $val }väntar på nästa ändring{ $end }. En bakgrund du väljer själv vinner ändå.
    }
settings-automation-schedule-off = Av. Dina regler sparas tills du slår på det här.
settings-automation-start-hint = första matchande regeln när tjänsten startar
settings-automation-editor-hint = bygg regler av villkorsblock
settings-automation-location = Plats och väder
settings-automation-location-desc = Används för soluppgång och solnedgång och för väderförhållanden.
settings-automation-location-say = Väder för { $city }, soltider vid { $lat }, { $lon }. Väderstyrda byten: { $mode }.
settings-automation-no-city = ingen stad
settings-automation-no-latitude = ingen latitud
settings-automation-no-longitude = ingen longitud
settings-automation-location-hint = Lämna staden tom för att använda koordinaterna i stället. Latitud är positiv norrut, longitud positiv österut.
settings-automation-startup = Start och aviseringar
settings-automation-startup-desc = Vad som händer när tjänsten startar och när bakgrunden byts.
settings-automation-startup-say = { $restore ->
        [restores] Vid start { $val }använder Skwd den senaste bakgrunden igen{ $end }.
       *[leaves] Vid start { $val }lämnar Skwd skrivbordet som det är{ $end }.
    } { $notify ->
        [notifies] Bakgrundsbyten { $val }skickar en avisering{ $end }.
       *[silent] Bakgrundsbyten { $val }sker tyst{ $end }.
    }
settings-automation-commands = När en bakgrund har tillämpats
settings-automation-commands-desc = Kör dina egna kommandon efter varje byte, till exempel för att styra ett annat bakgrundsverktyg.
settings-automation-command-count = { $count ->
        [one] { $val }{ $count } kommando{ $end } körs efter varje byte.
       *[other] { $val }{ $count } kommandon{ $end } körs efter varje byte.
    }
settings-automation-commands-say = { $listed ->
        [none] { $val }Inga kommandon{ $end } körs efter varje byte.
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }lämnar tillämpningen åt dina kommandon{ $end }, och kommandona { $val }körs även vid återställningen vid start{ $end }.
           *[skip] Skwd { $val }lämnar tillämpningen åt dina kommandon{ $end }, och kommandona { $val }hoppar över återställningen vid start{ $end }.
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }tillämpar bakgrunden själv{ $end }, och kommandona { $val }körs även vid återställningen vid start{ $end }.
           *[skip] Skwd { $val }tillämpar bakgrunden själv{ $end }, och kommandona { $val }hoppar över återställningen vid start{ $end }.
        }
    }
settings-automation-pick-only-hint = dina kommandon tillämpar den
settings-automation-commands-heading = Kommandon
settings-automation-placeholders = Använd %path%, %thumb%, %type% och %name% i ett kommando.
settings-automation-minutes = { $count } min
settings-automation-mode-off = Av
settings-automation-mode-outdoors = Följ vädret utomhus
settings-automation-mode-playlists = Välj ur dina spellistor
settings-automation-mode-semantic = Sök i ditt bibliotek
settings-automation-condition-clear = Klart
settings-automation-condition-cloudy = Molnigt
settings-automation-condition-rainy = Regnigt
settings-automation-condition-snowy = Snöigt
settings-automation-condition-stormy = Stormigt
settings-automation-condition-foggy = Dimmigt
settings-automation-condition-windy = Blåsigt
settings-automation-display-another = En annan
settings-automation-display-exclude = Uteslut
settings-automation-display-keep = Behåll
settings-automation-playlists-heading = Spellista per väder
settings-automation-hours = { $count } h
