settings-automation-rotation-desc = 間隔が過ぎるたびに、プールからランダムな壁紙を適用します。
settings-automation-rotation-say = { $interval }ごとに、{ $val }{ $pool }{ $end }からランダムに選びます。
settings-automation-rotation-off = オフ。壁紙は自分で選んだときだけ変わります。
settings-automation-pool = { $count ->
        [none] 空のプール
        [one] { $first }
        [two] { $first }と{ $second }
       *[three] { $first }、{ $second }、{ $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] お気に入りの{ $pool }
       *[all] { $pool }
    }
settings-automation-pool-images = 画像
settings-automation-pool-videos = 動画
settings-automation-pool-engine = Wallpaper Engine アイテム
settings-automation-pool-heading = プールの内容
settings-automation-timing = タイミング
settings-automation-interval-hint = 最小 10 秒
settings-automation-schedule-desc = ルールが時刻、曜日、天気、電源、ディスプレイに応じて壁紙とテーマを選びます。最初に一致したルールが優先されます。
settings-automation-schedule-say = { $start ->
        [applies] ルールはオンです。最初に一致したルールは{ $val }起動時に適用されます{ $end }。自分で選んだ壁紙は引き続き優先されます。
       *[waits] ルールはオンです。最初に一致したルールは{ $val }次の変化を待ちます{ $end }。自分で選んだ壁紙は引き続き優先されます。
    }
settings-automation-schedule-off = オフ。オンにするまでルールは保持されます。
settings-automation-start-hint = サービス起動時に最初に一致したルール
settings-automation-editor-hint = 条件ブロックでルールを作成
settings-automation-location = 場所と天気
settings-automation-location-desc = 日の出と日の入りの時刻、天気の判定に使います。
settings-automation-location-say = { $city }の天気、{ $lat }、{ $lon }の日照時刻。天気による切り替え：{ $mode }。
settings-automation-no-city = 都市なし
settings-automation-no-latitude = 緯度なし
settings-automation-no-longitude = 経度なし
settings-automation-location-hint = 都市を空にすると座標を使います。緯度は北が正、経度は東が正です。
settings-automation-startup = 起動と通知
settings-automation-startup-desc = サービスの起動時と壁紙が変わったときの動作。
settings-automation-startup-say = { $restore ->
        [restores] 起動時、Skwd は{ $val }最後の壁紙を再適用します{ $end }。
       *[leaves] 起動時、Skwd は{ $val }デスクトップをそのままにします{ $end }。
    }{ $notify ->
        [notifies] 壁紙が変わると{ $val }通知を送ります{ $end }。
       *[silent] 壁紙が変わっても{ $val }通知しません{ $end }。
    }
settings-automation-commands = 壁紙の適用後
settings-automation-commands-desc = 変更のたびに独自のコマンドを実行します。たとえば別の壁紙ツールを動かすときに使えます。
settings-automation-command-count = { $count ->
       *[other] 変更のたびに{ $val }{ $count } 個のコマンド{ $end }が実行されます。
    }
settings-automation-commands-say = { $listed ->
        [none] 変更後に実行される{ $val }コマンドはありません{ $end }。
       *[some] { $commands }
    }{ $apply ->
        [commands] { $rerun ->
            [also] Skwd は{ $val }適用をコマンドに任せ{ $end }、コマンドは{ $val }起動時の復元でも実行されます{ $end }。
           *[skip] Skwd は{ $val }適用をコマンドに任せ{ $end }、コマンドは{ $val }起動時の復元では実行されません{ $end }。
        }
       *[itself] { $rerun ->
            [also] Skwd は{ $val }壁紙を自分で適用し{ $end }、コマンドは{ $val }起動時の復元でも実行されます{ $end }。
           *[skip] Skwd は{ $val }壁紙を自分で適用し{ $end }、コマンドは{ $val }起動時の復元では実行されません{ $end }。
        }
    }
settings-automation-pick-only-hint = コマンドが適用します
settings-automation-commands-heading = コマンド
settings-automation-placeholders = コマンドでは %path%、%thumb%、%type%、%name% を使えます。
settings-automation-minutes = { $count } 分
settings-automation-mode-off = オフ
settings-automation-mode-outdoors = 屋外の天気に合わせる
settings-automation-mode-playlists = プレイリストから選ぶ
settings-automation-mode-semantic = ライブラリを検索
settings-automation-condition-clear = 晴れ
settings-automation-condition-cloudy = 曇り
settings-automation-condition-rainy = 雨
settings-automation-condition-snowy = 雪
settings-automation-condition-stormy = 嵐
settings-automation-condition-foggy = 霧
settings-automation-condition-windy = 強風
settings-automation-display-another = 別の壁紙
settings-automation-display-exclude = 除外
settings-automation-display-keep = 保持
settings-automation-playlists-heading = 天気ごとのプレイリスト
settings-automation-hours = { $count } 時間
