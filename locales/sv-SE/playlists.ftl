playlists-unnamed = (namnlös)
playlists-generated-name = Spellista { $number }
playlists-new-name = Namn på ny spellista
playlists-name = Spellistans namn
playlists-play-now = Spela nu
playlists-delete = Ta bort
playlists-type-curated = Manuell
playlists-type-smart = Filtrerad
playlists-order-shuffle = Blanda
playlists-order-sequential = I ordning
playlists-every = Var
playlists-seconds-minimum = sekund (minst 5)
playlists-all = Alla
playlists-favourites = Favoriter
playlists-filter-example = t.ex. type:video color:blue tag:cat,-anime
playlists-empty-smart = Inga bakgrundsbilder matchar filtret än.
playlists-empty-curated = Spellistan är tom. Öppna ett bakgrundskort och välj Spellista för att lägga till det.
playlists-stop-all = ■ Stoppa alla
playlists-card-add-title = Lägg till i spellista
playlists-card-none = Det finns inga spellistor än. Ge en ett namn för att skapa den med den här bakgrundsbilden redan tillagd:
playlists-card-new = Eller skapa en ny spellista med den här bakgrundsbilden:
playlists-masthead = Spellistor  /  { $selected }
playlists-masthead-fallback = Bibliotek
playlists-index-title = Spellistor
playlists-index-subtitle = En manuell spellista innehåller bakgrundsbilder som du väljer. En filtrerad spellista hålls synkad med sina regler.
playlists-index-live-detail = { $kind } · aktiv på { $outputs }
playlists-index-count-detail =
    { $kind }  ·  { $count } { $count ->
        [one] bakgrundsbild
       *[other] bakgrundsbilder
    }
playlists-index-empty-title = Inga spellistor än
playlists-index-empty-hint = Skapa en nedan för att komma igång.
playlists-new-playlist = Ny spellista
playlists-live-routing = Aktiva skärmar
playlists-active-assignments =
    { $count } { $count ->
        [one] aktiv tilldelning
       *[other] aktiva tilldelningar
    }
playlists-library = Sparade spellistor
playlists-library-stats = { $count } sparade · ingen aktiv
playlists-live-results = Matchande bakgrundsbilder
playlists-the-sequence = Uppspelningsordning
playlists-sequence-smart-hint = Bakgrundsbilder som matchar det aktuella filtret, i den ordning de visas.
playlists-sequence-curated-hint = Bakgrundsbilderna spelas uppifrån och ner. Dra dem för att ändra ordningen.
playlists-empty-smart-hint = Inga bakgrundsbilder matchar reglerna. Ändra filtret för att få med fler.
playlists-empty-curated-hint = Öppna ett bakgrundskort och välj Spellista för att lägga till den första bakgrundsbilden.
playlists-live-badge = Aktiv
playlists-empty-eyebrow = Spellistor / bibliotek
playlists-empty-title = Bygg en rotation
playlists-empty-choose = Välj en spellista i indexet
playlists-empty-desc = Eller skapa en ny manuell spellista och börja lägga till bakgrundsbilder.
playlists-kind-smart = Filtrerad spellista
playlists-kind-curated = Manuell spellista
playlists-state-live = Aktiv / { $outputs }
playlists-state-query-ready = Filtret är redo
playlists-state-ready =
    { $padded } { $count ->
        [one] bakgrundsbild redo
       *[other] bakgrundsbilder redo
    }
playlists-edition-eyebrow = Spellistor / redigera
playlists-kind-tagline = { $kind } · redo att tilldelas skärmar
playlists-stop-edition = Stoppa spellistan
playlists-colour-notes = Färger
playlists-folder-notes = Mappar
playlists-live-collection-title = Filtrerad spellista
playlists-live-collection-desc = Listan uppdateras automatiskt utifrån valda filter för källa, taggar, upplösning och färg.
playlists-definition-title = Spellistans detaljer
playlists-definition-desc = Ange namnet och välj om bakgrundsbilder läggs till manuellt eller väljs av ett filter.
playlists-playback-title = Uppspelning
playlists-playback-desc = Välj ordningen och hur länge varje bakgrundsbild visas.
playlists-routing-title = Använd på skärmar
playlists-routing-desc = Välj var spellistan körs. Om du väljer alla skärmar skapas en gemensam tilldelning.
playlists-routing-all = Alla anslutna skärmar
playlists-routing-live = Aktiv tilldelning
playlists-routing-none = Ingen aktiv tilldelning
playlists-demo-name = Det värsta klippet

playlists-id = ID { $id }
playlists-picker-entry = { $name } · ID { $id }
playlists-choose = Choose a playlist
playlists-browse-all = All wallpapers
playlists-edit = Edit playlists
playlists-picker-search = Search by name or ID
playlists-picker-no-match = No playlists match this search.
playlists-picker-hint = Choose which wallpapers to browse. Playback stays as it is.
playlists-filter-help = Filter reference
playlists-help-combine = Separera filter med mellanslag. Varje filter måste matcha, utom mappregler: ett objekt i någon av de angivna mapparna räknas. Citera sökvägar som innehåller mellanslag.
playlists-help-source = all / favourites
    Start with all wallpapers or only favourites.
playlists-help-folder = folder:nature/forest
    Matchar den här mappen och dess undermappar. Flera mappregler matchar vilken som helst av dem; citera sökvägar som innehåller mellanslag.
playlists-help-type = type:image / type:video / type:we
    Images, videos, or Wallpaper Engine wallpapers. Also accepts static, picture, pic, img; vid, mp4; scene, wallpaperengine, wallpaper-engine.
playlists-help-tag = tag:cat,night / tag:cat|dog,-anime
    Commas require every tag; | accepts either tag; - excludes a tag. Include at least one positive tag. Tags ignore letter case.
playlists-help-colour = color:blue,cyan / colour:gray
    Commas accept any listed colour. Colours: red, orange, yellow, lime, green, spring, cyan, azure, blue, violet, magenta, pink, gray. Numeric buckets 0-11 and 99 also work.
    Aliases: chartreuse; mint, emerald; teal, aqua; sky; purple, indigo; fuchsia; rose; grey, mono, monochrome, grayscale, greyscale, black, white.
playlists-help-ratio = ratio:portrait / ratio:landscape / ratio:square
    Tall, wide, or nearly square (within 0.05 of 1:1). Use decimal ratios for comparisons, such as ratio:>1.7.
playlists-help-width = width:>=1920
    Compare the wallpaper width in pixels.
playlists-help-height = height:>=1080
    Compare the wallpaper height in pixels.
playlists-help-resolution = res:>=1920x1080
    Both dimensions must satisfy the comparison.
playlists-help-comparison = Comparisons: >=, <=, >, <, =
    With no operator, numbers mean at least. Numeric = allows a difference below 0.5, including ratios. Use lowercase filter keywords.
playlists-help-example = Example: favourites type:video ratio:landscape color:blue
    Blue, wide videos from your favourites.
