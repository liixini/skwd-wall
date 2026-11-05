settings-sources-other = その他のソース
settings-sources-other-desc = Unsplash、Pexels、Bing の今日の画像、YouTube。
settings-sources-wallhaven-say = { $top ->
        [yes] 新しいセッションは { $sort } 順 ({ $range })、比率 { $ratio }、{ $categories } を { $purity } で表示します。
       *[no] 新しいセッションは { $sort } 順、比率 { $ratio }、{ $categories } を { $purity } で表示します。
    }
settings-sources-wallhaven-off = オフ。Wallhaven ブラウザーは非表示です。
settings-sources-workshop-say = { $trend ->
        [yes] 新しいセッションは { $sort } 順 ({ $days })、タイプ { $type }、解像度 { $resolution }、カテゴリー { $category } です。
       *[no] 新しいセッションは { $sort } 順、タイプ { $type }、解像度 { $resolution }、カテゴリー { $category } です。
    }
settings-sources-workshop-off = オフ。Steam ワークショップのブラウザーと Wallpaper Engine のシーン設定は非表示です。
settings-sources-no-category = カテゴリーなし
settings-sources-no-content = コンテンツレベルなし
settings-sources-content = コンテンツ
settings-sources-account = アカウント
settings-sources-browser = ブラウザー
settings-sources-browser-folders = ブラウザーとフォルダー
settings-sources-photos = 写真
settings-sources-daily = 今日の画像と動画
settings-sources-not-set = 未設定
settings-sources-detected = Steam から検出
settings-sources-nsfw-hint = NSFW の結果に必要
settings-sources-grid-hint = 列 × 行
settings-sources-thumb-hint = 幅 × 高さ
settings-sources-steamcmd-hint = steamcmd 用
settings-sources-workshop-only = ワークショップのみ
settings-sources-method-steam = Steam クライアント
settings-sources-method-steamcmd = steamcmd
settings-sources-method-steam-hint = skwd-deck-steamworks と起動中の Steam が必要
settings-sources-method-steamcmd-hint = steamcmd と一度のログインが必要
settings-sources-unsplash-hint = unsplash.com/developers で無料
settings-sources-pexels-hint = pexels.com/api で無料
settings-sources-bing-hint = トップページの最新 8 枚
settings-sources-youtube-hint = PATH に yt-dlp が必要
settings-sources-clip-hint = 0 ですべてダウンロード
settings-sources-searching = { $sources } を検索中です。
settings-sources-searching-none = 検索中のソースはありません。
settings-sources-needs-key = { $key ->
        [access] { $source } にはアクセスキーが必要です。
       *[api] { $source } には API キーが必要です。
    }
settings-sources-off = { $count ->
        [one] { $sources } はオフです。
       *[some] { $sources } はオフです。
    }
