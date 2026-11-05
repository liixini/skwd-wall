settings-sources-other = Other sources
settings-sources-other-desc = Unsplash, Pexels, Bing daily and YouTube.
settings-sources-wallhaven-say = { $top ->
        [yes] New sessions sort by { $sort } over { $range }, ratio { $ratio }, showing { $categories } at { $purity }.
       *[no] New sessions sort by { $sort }, ratio { $ratio }, showing { $categories } at { $purity }.
    }
settings-sources-wallhaven-off = Off. The Wallhaven browser is hidden.
settings-sources-workshop-say = { $trend ->
        [yes] New sessions sort by { $sort } over { $days }, type { $type }, resolution { $resolution }, category { $category }.
       *[no] New sessions sort by { $sort }, type { $type }, resolution { $resolution }, category { $category }.
    }
settings-sources-workshop-off = Off. The Steam Workshop browser and the Wallpaper Engine scene settings are hidden.
settings-sources-no-category = no category
settings-sources-no-content = no content level
settings-sources-content = Content
settings-sources-account = Account
settings-sources-browser = Browser
settings-sources-browser-folders = Browser and folders
settings-sources-photos = Photos
settings-sources-daily = Daily images and video
settings-sources-not-set = not set
settings-sources-detected = Detected from Steam
settings-sources-nsfw-hint = Needed for NSFW results
settings-sources-grid-hint = Columns × rows
settings-sources-thumb-hint = Width × height
settings-sources-steamcmd-hint = For steamcmd
settings-sources-workshop-only = Workshop only
settings-sources-method-steam = Steam Client
settings-sources-method-steamcmd = steamcmd
settings-sources-method-steam-hint = Needs skwd-deck-steamworks and running Steam
settings-sources-method-steamcmd-hint = Needs steamcmd and a one-time login
settings-sources-unsplash-hint = Free from unsplash.com/developers
settings-sources-pexels-hint = Free from pexels.com/api
settings-sources-bing-hint = The last 8 homepage images
settings-sources-youtube-hint = Needs yt-dlp on PATH
settings-sources-clip-hint = 0 downloads everything
settings-sources-searching = Searching { $sources }.
settings-sources-searching-none = No source is searching.
settings-sources-needs-key = { $key ->
        [access] { " " }{ $source } needs an access key.
       *[api] { " " }{ $source } needs an API key.
    }
settings-sources-off = { $count ->
        [one] { " " }{ $sources } is off.
       *[some] { " " }{ $sources } are off.
    }
