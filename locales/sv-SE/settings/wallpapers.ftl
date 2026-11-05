settings-wallpapers-displays-desc = Vad varje skärm visar och hur den placeras.
settings-wallpapers-play = Så spelas bakgrunder
settings-wallpapers-play-desc = Motorer, deras lager och scener.
settings-wallpapers-transitions-desc = Hur en bakgrund övergår i nästa.
settings-wallpapers-awww-desc = Övergången awww spelar för bilder.
settings-wallpapers-drawing = Teckning
settings-wallpapers-drawing-desc = Förberedda bakgrunder tecknar fram sig själva.
settings-wallpapers-sound = Ljud
settings-wallpapers-sound-desc = Volym, och när bakgrundsljudet viker undan.
settings-wallpapers-pause-desc = Pausa animerade bakgrunder när en regel uppfylls.
settings-wallpapers-plasma = Plasmas låsskärm
settings-wallpapers-plasma-desc = Vilken bakgrund Plasma visar när skärmen är låst.
settings-wallpapers-niri = Niri-översikt
settings-wallpapers-niri-desc = Visas på niri. En bakgrund bakom översikten.
settings-wallpapers-display-more = Mer
settings-wallpapers-lock = Lås { $name }
settings-wallpapers-lock-hint = Uppdateras bara genom multiväljaren
settings-wallpapers-layer-bottom = Nederst
settings-wallpapers-gpu-hint = Faller tillbaka på Automatisk
settings-wallpapers-timeout-hint = För mycket stora bilder
settings-wallpapers-animated = Animerade bakgrunder
settings-wallpapers-scenes = Scener
settings-wallpapers-scenes-hint = FPS-tak · skalning
settings-wallpapers-particles-hint = Bara scener
settings-wallpapers-preview-hint = Bildfrekvens
settings-wallpapers-quality-short = Sand kräver minst { $ms } ms.
settings-wallpapers-quality-hint = Full saknar tak, Låg stannar vid 30 fps
settings-wallpapers-sharpen-hint = När kvaliteten sänker upplösningen
settings-wallpapers-monitors = Skärmar
settings-wallpapers-primary-hint = Animerar bara för Primär
settings-wallpapers-primary-blank = första tillgängliga
settings-wallpapers-primary-only-hint = Resten använder alla skärmar
settings-wallpapers-wave = Våg
settings-wallpapers-wave-hint = Bredd × höjd
settings-wallpapers-position-hint = center, top-left eller 0.5,0.5
settings-wallpapers-bezier-hint = x1,y1,x2,y2
settings-wallpapers-bezier-blank = awwws standard
settings-wallpapers-invert-hint = Vänder ursprungets Y-axel
settings-wallpapers-colours-files = Färger och filer
settings-wallpapers-preparing = Förberedelse
settings-wallpapers-paper-ink = Papper och bläck
settings-wallpapers-prepare = Förbered
settings-wallpapers-prepare-all = Alla bakgrunder
settings-wallpapers-scope-view = Aktuell vy · { $count }
settings-wallpapers-pack-ready = Bearbetningsfilerna är klara.
settings-wallpapers-pack-none = Inga bearbetningsfiler än.
settings-wallpapers-pack-blank = Inga än
settings-wallpapers-drawing-blocked = Teckning är inte tillgänglig. Välj bearbetningsfiler först.
settings-wallpapers-volume = Volym
settings-wallpapers-other-apps = Andra appar
settings-wallpapers-pause-apps = Pausar medan listade appar körs.
settings-wallpapers-pause-none = Ingen pausregel är på.
settings-wallpapers-process-hint = * och ? matchar
settings-wallpapers-process-action = Välj
settings-wallpapers-process-blank = inga än
settings-wallpapers-resume-hint = När sista regeln släppt
settings-wallpapers-scope-all = alla skärmar
settings-wallpapers-scope-display = fönstrets skärm
settings-wallpapers-rule-fullscreen = helskärmsfönster
settings-wallpapers-rule-maximised = maximerade fönster
settings-wallpapers-rule-focused = fönster i fokus
settings-wallpapers-rule-full-width = kolumner i full bredd
settings-wallpapers-plasma-left = lämnas till Plasma
settings-wallpapers-plasma-static = visar en fast bild
settings-wallpapers-plasma-follow = följer den senast tillämpade bakgrunden
settings-wallpapers-plasma-off-row = Ohanterad lämnar Plasma i fred.
settings-wallpapers-image-hint = Absolut sökväg, eller en som börjar med ~/
settings-wallpapers-image-blank = ingen bild
settings-wallpapers-dynamic-hint = Stillbilden är en tyst bild
settings-wallpapers-niri-off = Av. Översikten behåller niris egen bakgrund.
settings-wallpapers-source = Källa
settings-wallpapers-look = Utseende
settings-wallpapers-source-blank = senaste bakgrunden
settings-wallpapers-blur = Gör bakgrunden suddig
settings-wallpapers-radii = Oskärperadie
settings-wallpapers-radii-hint = Bilder · videor · scener
settings-wallpapers-dim-hint = 0 ingen, 100 svart
settings-wallpapers-transitions-off = Av. Varje bakgrund klipper direkt till nästa.
settings-wallpapers-transitions-off-foot = Slå på övergångar för att välja förhandsvisning, skärmar och sandkvalitet.
settings-wallpapers-awww-none = Bilder byts med { $type }.
settings-wallpapers-awww-no-type = ingen övergång
settings-wallpapers-sound-muted = Bakgrundsljudet är tystat.
settings-wallpapers-displays-say = { $state ->
        [none] Ingen skärm är låst.
        [one] { $names } är låst.
        [some] { $names } är låsta.
       *[detecting] Söker efter skärmar.
    } Skärmar utan egen placering använder { $fill }.
settings-wallpapers-play-say = Bilder använder { $backend }, videor { $val }Vulkan{ $end }, på lagret { $layer }.
settings-wallpapers-transitions-say = Familjen { $family }{ $kind ->
        [random] , en ny varje gång, över { $duration }.
        [preset] , med { $shader }, på { $ms } ms.
       *[shader] , med { $shader }, över { $duration }.
    } Bildfrekvens { $rate }.
settings-wallpapers-primary-count = { $state ->
        [none] ingen av { $all }
        [all] alla { $all }
       *[some] { $shown } av { $all }
    }
settings-wallpapers-awww-say = Bilder spelar { $type } med filtret { $filter }, över { $duration }.
settings-wallpapers-drawing-say = { $state ->
        [on] Förberedda bakgrunder tecknas över { $duration }.
       *[off] Av. Kort med Teckning tecknas ändå över { $duration }.
    }
settings-wallpapers-sound-say = Bakgrundsljudet spelas på { $volume }{ $duck ->
        [yes] { " " }och pausas när andra appar spelar ljud.
       *[no] .
    }
settings-wallpapers-pause-say = Pausar vid { $rules } på { $scope }{ $apps ->
        [yes] , och för listade appar.
       *[no] .
    }
settings-wallpapers-pause-rules = { $count } fönsterregler
settings-wallpapers-plasma-say = { $state ->
        [off] Låsskärmen { $mode }.
        [static] Låsskärmen { $mode }: { $image }.
       *[follow] Låsskärmen { $mode }, scener som { $scenes ->
        [live] direktrendering.
       *[poster] stillbild.
    }
    }
settings-wallpapers-niri-say = Visar { $source ->
       *[current] den aktuella bakgrunden
        [own] sin egen källa
        [stationary] en fast bakgrund
    }{ $blur ->
        [yes] , suddig
       *[no] { "" }
    }, dämpad { $dim }{ $auto ->
        [yes] , omfärgad
       *[no] { "" }
    }.
settings-wallpapers-device-hint = Automatiskt provar CUDA, WebGPU, sedan CPU
settings-wallpapers-memory-hint = Över gränsen försöker Automatiskt igen på CPU
