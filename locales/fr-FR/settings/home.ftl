settings-home-search = Rechercher un réglage : fréquence d’images, son, écran de verrouillage, flou
settings-home-displays = { $count ->
    [one] { $count } écran
   *[other] { $count } écrans
 }
settings-home-displays-say = { $link } { $count ->
    [one] affiche des fonds d’écran.
   *[other] affichent des fonds d’écran.
 }
settings-home-watching = { " " }Surveillance de la bibliothèque : { $val }{ $state }{ $end }.
settings-home-model-ready = prêt
settings-home-model-missing = aucun modèle
settings-home-model-off = désactivé
settings-home-colours-from = { " " }{ $link } proviennent de { $val }{ $engine }{ $end }.
settings-home-sound-desc = Audio des fonds vidéo et Wallpaper Engine.
settings-home-sound-say = Les fonds d’écran jouent à { $volume }{ $duck ->
    [yes] { " " }et se mettent en pause quand d’autres applis jouent du son.
   *[no] , même quand d’autres applis jouent du son.
 }
settings-home-colours-desc = Comment les couleurs du fond atteignent le reste du bureau.
settings-home-colours-say = { settings-theme-policy-label } : { $policy }.{ " " }Plus dans { $place }.
settings-home-rotation-desc = Appliquer un fond aléatoire à chaque fin d’intervalle.
settings-home-rotation-say = Un fond aléatoire toutes les { $interval }.
settings-home-rotation-off = La rotation est désactivée. Le fond ne change que lorsque vous en choisissez un.
settings-home-transitions-desc = Comment un fond passe au suivant.
settings-home-transitions-say = Un fond passe au suivant en { $duration }.
settings-home-transitions-off = Les transitions sont désactivées. Les fonds passent directement au suivant.
settings-home-changed = Modifié par rapport au défaut
settings-home-changed-desc = Les réglages que vous avez modifiés, au même endroit, chacun avec Réinitialiser.
settings-home-changed-none = Rien ne diffère de la valeur par défaut parmi les réglages comparés ici.
settings-home-changed-say = { $count ->
    [one] { $count } réglage diffère de sa valeur par défaut.
   *[other] { $count } réglages diffèrent de leurs valeurs par défaut.
 }
settings-home-review = Vérifier
settings-home-reset = Réinitialiser
settings-home-reset-all = Tout réinitialiser
settings-home-badge = { $count } modifiés
settings-home-automatic = Automatique
settings-home-displays-none = Aucun écran
settings-home-displays-none-say = { $link } n’affiche de fond d’écran.
settings-home-reset-scope = Réinitialise tous les réglages ci-dessus, raccourcis clavier compris. Les clés d’API, noms d’utilisateur, positions et dossiers restent.
settings-home-items = { $count ->
    [one] { $count } élément
   *[other] { $count } éléments
 }
