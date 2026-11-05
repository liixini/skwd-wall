settings-home-search = Sök i alla inställningar: bildfrekvens, ljud, låsskärm, oskärpa
settings-home-displays = { $count ->
    [one] { $count } skärm
   *[other] { $count } skärmar
 }
settings-home-displays-say = { $link } visar bakgrundsbilder.
settings-home-watching = { " " }Biblioteksbevakning: { $val }{ $state }{ $end }.
settings-home-model-ready = redo
settings-home-model-missing = ingen modell
settings-home-model-off = av
settings-home-colours-from = { " " }{ $link } kommer från { $val }{ $engine }{ $end }.
settings-home-sound-desc = Ljud från video- och Wallpaper Engine-bakgrunder.
settings-home-sound-say = Bakgrunder spelas på { $volume }{ $duck ->
    [yes] { " " }och pausas när andra appar spelar ljud.
   *[no] , även när andra appar spelar ljud.
 }
settings-home-colours-desc = Hur bakgrundens färger når resten av skrivbordet.
settings-home-colours-say = { settings-theme-policy-label }: { $policy }.{ " " }Mer i { $place }.
settings-home-rotation-desc = Sätt en slumpad bakgrund varje gång intervallet löper ut.
settings-home-rotation-say = En slumpad bakgrund var { $interval }.
settings-home-rotation-off = Rotation är av. Bakgrunden byts bara när du väljer en.
settings-home-transitions-desc = Hur en bakgrund övergår i nästa.
settings-home-transitions-say = En bakgrund övergår i nästa på { $duration }.
settings-home-transitions-off = Övergångar är av. Bakgrunder byts direkt till nästa.
settings-home-changed = Ändrat från standard
settings-home-changed-desc = Inställningar du har ändrat, på ett ställe, var och en med Återställ.
settings-home-changed-none = Inget avviker från standardvärdet bland de inställningar som jämförs här.
settings-home-changed-say = { $count ->
    [one] { $count } inställning skiljer sig från sitt standardvärde.
   *[other] { $count } inställningar skiljer sig från sina standardvärden.
 }
settings-home-review = Granska
settings-home-reset = Återställ
settings-home-reset-all = Återställ alla
settings-home-badge = { $count } ändrade
settings-home-automatic = Automatiskt
settings-home-displays-none = Inga skärmar
settings-home-displays-none-say = { $link } visar bakgrundsbilder.
settings-home-reset-scope = Återställer alla inställningar ovan, även kortkommandon. API-nycklar, användarnamn, platser och mappar behålls.
settings-home-items = { $count ->
    [one] { $count } objekt
   *[other] { $count } objekt
 }
