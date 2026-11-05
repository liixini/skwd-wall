settings-library-watch-section-desc = Upptäck filer som läggs till utanför skwd-wall och håll biblioteket aktuellt.
settings-library-watch-fallback-label = Reservavläsning
settings-library-watch-fallback-desc = Kontrollera endast biblioteksmappar där inbyggd filbevakning inte fungerar. Aktivera detta för nätverks- eller FUSE-monteringar som missar ändringar och starta sedan om skwd-walld.
settings-library-watch-interval-label = Avläsningsintervall
settings-library-watch-interval-desc = Vänta så här många sekunder mellan begränsade kontroller. Lägre värden hittar ändringar snabbare men läser filsystemet oftare. Starta om skwd-walld efter en ändring.
settings-library-watch-unknown-label = Bevakarstatus saknas
settings-library-watch-unknown-desc = Den här demonen rapporterar inte biblioteksbevakningens status. Uppdatera eller starta om skwd-walld.
settings-library-watch-poll-failed-label = Reservavläsningen kan inte läsa en biblioteksmapp
settings-library-watch-poll-failed-desc = Kontrollera att alla konfigurerade biblioteksmappar är monterade och läsbara. En ny avläsning görs om { $interval } sekunder.
settings-library-watch-polling-label = Reservavläsning aktiv
settings-library-watch-polling-desc = Inbyggd bevakning misslyckades för { $count ->
    [one] en biblioteksmapp
   *[other] { $count } biblioteksmappar
    }. Upp till { $budget } poster kontrolleras med { $interval } sekunders mellanrum. Senaste slutförda synkronisering: { $convergence }.
settings-library-watch-recovering-label = Inbyggd bevakning har återhämtats
settings-library-watch-recovering-desc = Den inbyggda bevakaren är aktiv igen. En fullständig överlämningsskanning pågår innan biblioteket förklaras aktuellt.
settings-library-watch-unavailable-label = Biblioteksbevakning saknas
settings-library-watch-unavailable-desc = Inbyggd filbevakning misslyckades och reservavläsning är avstängd. Aktivera Reservavläsning och starta sedan om skwd-walld.
settings-library-watch-recovered-label = Inbyggd bevakning återställd
settings-library-watch-recovered-desc = Den inbyggda bevakaren och dess överlämningsskanning är aktuella. Senaste slutförda synkronisering: { $convergence }.
settings-library-watch-native-label = Inbyggd filbevakning
settings-library-watch-native-desc = Filsystemshändelser är aktiva för alla biblioteksmappar. Reservavläsningen vilar.
settings-library-watch-convergence-never = Inte slutförd ännu
settings-library-watch-convergence-seconds = { $value ->
    [one] för { $value } sekund sedan
   *[other] för { $value } sekunder sedan
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] för { $value } minut sedan
   *[other] för { $value } minuter sedan
    }
settings-library-watch-convergence-hours = { $value ->
    [one] för { $value } timme sedan
   *[other] för { $value } timmar sedan
    }
settings-profiles-card = Profiler
settings-profiles-card-desc = Bestäm vilka mappar väljaren, mappfiltret, slumpade bakgrunder och scheman använder. Dolda mappar kommer tillbaka med växeln för dolda i fältet eller dess tangentbindning, och varje start börjar med dem dolda igen.
settings-profiles-everything = Allt
settings-profiles-name-label = Namn
settings-profiles-name-placeholder = Profilnamn
settings-profiles-state-show = Visa
settings-profiles-state-hide = Dölj
settings-profiles-state-exclude = Undanta
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = Workshop-objekt har inga mappar, så det här gäller alla. Dölj en enskild bakgrund på kortets baksida.
settings-profiles-summary = { $hidden } dolda, { $excluded } undantagna
settings-profiles-summary-none = Visar alla mappar
settings-profiles-delete-action = Ta bort profil
settings-profiles-add-label = Ny profil
settings-profiles-new-name = Ny profil
settings-profiles-never-scanned = Söks aldrig igenom
settings-profiles-mixed = Blandat inuti
settings-profiles-hidden-inside = { $count ->
    [one] { $count } dold inuti
   *[other] { $count } dolda inuti
    }
settings-profiles-copy-action = Kopiera
settings-profiles-helm-desc = Gör den här till aktiv profil, så att rotation och scheman följer.
settings-profiles-flag-desc = Öppnar en väljare i den här profilen och lämnar den aktiva ifred.
settings-profiles-make-active = Gör aktiv
settings-profiles-active-tag = Aktiv
settings-profiles-builtin-tag = Inbyggd
settings-profiles-filter-placeholder = Filtrera mappar
settings-profiles-no-match = Ingen mapp matchar.
settings-profiles-hidden-note-label = Dolda bakgrunder
settings-profiles-everything-desc = Allt visar varje skannad mapp och varje Workshop-objekt. Skapa en ny profil för att dölja eller utesluta några.
settings-profiles-delete-hint = Klicka igen för att ta bort. Bakgrunder och mappar stannar där de är.
