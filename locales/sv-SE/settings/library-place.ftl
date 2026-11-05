settings-library-folders-desc = Sök igenom de här mapparna och deras undermappar efter bilder och videor.
settings-library-folders-say = Söker i { $val }{ $folders }{ $end }.{ $state ->
        [polling] { " " }Avläsning täcker det inbyggd bevakning missar, var { $every }.{ $unread }
        [native] { " " }Inbyggd filbevakning är aktiv; avläsning vilar.
        [down] { " " }Inbyggd bevakning misslyckades och avläsning är av.
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }En mapp går inte att läsa.
       *[other] { " " }{ $count } mappar går inte att läsa.
    }
settings-library-watching = Bevakning
settings-library-poll-hint = För nätverks- och FUSE-mappar
settings-library-interval-hint = Starta om skwd-walld efter ändring
settings-library-excluded-desc = Hoppa över de här mapparna och deras undermappar. Befintlig metadata sparas.
settings-library-excluded-say = Söker aldrig i { $val }{ $folders }{ $end }.{ $outside }
settings-library-outside = { $count ->
        [one] { " " }En ligger utanför alla bakgrundsmappar och hoppar inte över något.
       *[other] { " " }{ $count } ligger utanför alla bakgrundsmappar och hoppar inte över något.
    }
settings-library-excluded-hint = Uteslutna mappar söks aldrig igenom, så ingen profil kan visa dem. Vill du behålla en mapp men dölja den offentligt, använd { $link }.
settings-library-profiles-desc = Bestäm vilka mappar väljaren, mappfiltret, slumpade bakgrunder och scheman använder.
settings-library-profiles-say = Aktiv profil { $state ->
        [none] { $menu }, som visar alla mappar.
       *[some] { $menu }: { $summary }.
    } Ctrl+H eller fältets knapp för dolda visar dolda bakgrunder, även de som dolts från sitt kort, fram till nästa start.
settings-library-active = Aktiv profil
settings-library-terminal = Byt från en terminal
settings-library-tree-note = Dölj håller utanför slump tills Ctrl+H. Uteslut håller utanför även med Ctrl+H. En mapp styr sina undermappar.
settings-library-show-tip = I väljaren och slumpen
settings-library-hide-tip = Tills Ctrl+H, aldrig slump
settings-library-exclude-tip = Utanför den här profilen, Ctrl+H hoppar över
settings-library-from = Från { $link }
settings-library-excluded-inside = { $count } uteslutna inuti
settings-library-both-inside = { $hidden } dolda, { $excluded } uteslutna inuti
settings-library-storage = Lagring och papperskorg
settings-library-storage-desc = Mindre WebP-kopior, återställningslagring och omfärgade kopior.
settings-library-storage-say = { $optimise ->
        [on] Nya bilder blir WebP med kvalitet { $quality }, högst { $size }.
       *[off] Nya bilder förblir som de är. Optimera alla använder kvalitet { $quality }, högst { $size }.
    } { $trash ->
        [on] Kastade original sparas i { $days } och raderas sedan.
       *[off] Kastade original sparas tills du raderar dem.
    } { $recolour ->
        [on] Nya bakgrunder får också en omfärgad kopia.
       *[off] Nya bakgrunder färgas inte om.
    }
settings-library-retention-hint = Dagar att behålla kastade bilder
settings-library-optimise-hint = Kör bildoptimering på hela biblioteket
settings-library-recolour = Omfärgning
settings-library-thumbnails = Miniatyrer och reparation
settings-library-thumbnails-desc = Bygg om förhandsvisningar och färgdata när något ser fel ut.
settings-library-thumbnails-say = Upp till { $jobs } miniatyrer skapas samtidigt vid ombyggnad.
settings-library-rebuild = Bygg om
settings-library-start-over = Börja om
settings-library-recompute-hint = Från befintliga miniatyrer
settings-library-clear-hint = Miniatyrer skapas igen vid nästa sökning
