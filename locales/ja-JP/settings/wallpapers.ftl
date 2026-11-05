settings-wallpapers-displays-desc = 各画面の表示内容と配置。
settings-wallpapers-play = 壁紙の再生
settings-wallpapers-play-desc = エンジン、描画レイヤー、シーン。
settings-wallpapers-transitions-desc = 壁紙が次へ切り替わる方法。
settings-wallpapers-awww-desc = awww が画像に使うトランジション。
settings-wallpapers-drawing = ドローイング
settings-wallpapers-drawing-desc = 準備済みの壁紙が自分で描かれます。
settings-wallpapers-sound = サウンド
settings-wallpapers-sound-desc = 音量と、壁紙音声を控えるタイミング。
settings-wallpapers-pause-desc = ルールで動く壁紙を一時停止。
settings-wallpapers-plasma = Plasma ロック画面
settings-wallpapers-plasma-desc = ロック時に Plasma が出す壁紙。
settings-wallpapers-niri = Niri オーバービュー
settings-wallpapers-niri-desc = niri のオーバービュー背景。
settings-wallpapers-display-more = その他
settings-wallpapers-lock = { $name } をロック
settings-wallpapers-lock-hint = マルチピッカーのみ
settings-wallpapers-layer-bottom = 下
settings-wallpapers-gpu-hint = 自動に戻ります
settings-wallpapers-timeout-hint = 非常に大きな画像向け
settings-wallpapers-animated = 動く壁紙
settings-wallpapers-scenes = シーン
settings-wallpapers-scenes-hint = FPS 上限 · スケーリング
settings-wallpapers-particles-hint = シーンのみ
settings-wallpapers-preview-hint = フレームレート
settings-wallpapers-quality-short = 砂には { $ms } ms 以上が必要です。
settings-wallpapers-quality-hint = Full は無制限、Low は 30 fps まで
settings-wallpapers-sharpen-hint = 画質で解像度を下げているとき
settings-wallpapers-monitors = モニター
settings-wallpapers-primary-hint = プライマリのみアニメーション
settings-wallpapers-primary-blank = 最初に使えるもの
settings-wallpapers-primary-only-hint = 他はすべてのモニターを使用
settings-wallpapers-wave = 波
settings-wallpapers-wave-hint = 幅 × 高さ
settings-wallpapers-position-hint = center、top-left、0.5,0.5 など
settings-wallpapers-bezier-hint = x1,y1,x2,y2
settings-wallpapers-bezier-blank = awww の既定
settings-wallpapers-invert-hint = 原点の Y 軸を反転
settings-wallpapers-colours-files = 色とファイル
settings-wallpapers-preparing = 準備
settings-wallpapers-paper-ink = 紙とインク
settings-wallpapers-prepare = 準備
settings-wallpapers-prepare-all = すべての壁紙
settings-wallpapers-scope-view = 現在のビュー · { $count }
settings-wallpapers-pack-ready = 処理ファイルの準備完了。
settings-wallpapers-pack-none = 処理ファイルはまだありません。
settings-wallpapers-pack-blank = まだなし
settings-wallpapers-drawing-blocked = ドローイングは使えません。先に処理ファイルを選んでください。
settings-wallpapers-volume = 音量
settings-wallpapers-other-apps = 他のアプリ
settings-wallpapers-pause-apps = 一覧のアプリの実行中は一時停止します。
settings-wallpapers-pause-none = 一時停止ルールはオフです。
settings-wallpapers-process-hint = * と ? が使えます
settings-wallpapers-process-action = 選択
settings-wallpapers-process-blank = まだなし
settings-wallpapers-resume-hint = 最後のルールが解除された後
settings-wallpapers-scope-all = すべての画面
settings-wallpapers-scope-display = ウィンドウの画面
settings-wallpapers-rule-fullscreen = 全画面表示
settings-wallpapers-rule-maximised = 最大化
settings-wallpapers-rule-focused = フォーカス中
settings-wallpapers-rule-full-width = 全幅カラム
settings-wallpapers-plasma-left = Plasma に任せます
settings-wallpapers-plasma-static = 固定画像を表示します
settings-wallpapers-plasma-follow = 最後に適用した壁紙に従います
settings-wallpapers-plasma-off-row = 管理なしでは Plasma に触れません。
settings-wallpapers-image-hint = 絶対パス、または ~/ で始まるパス
settings-wallpapers-image-blank = 画像なし
settings-wallpapers-dynamic-hint = ポスターフレームは無音の 1 枚です
settings-wallpapers-niri-off = オフ。オーバービューは niri の背景のままです。
settings-wallpapers-source = ソース
settings-wallpapers-look = 見た目
settings-wallpapers-source-blank = 最後の壁紙
settings-wallpapers-blur = 背景をぼかす
settings-wallpapers-radii = ぼかし半径
settings-wallpapers-radii-hint = 画像 · 動画 · シーン
settings-wallpapers-dim-hint = 0 なし、100 黒
settings-wallpapers-transitions-off = オフ。壁紙はそのまま次へ切り替わります。
settings-wallpapers-transitions-off-foot = トランジションをオンにすると、プレビュー、モニター、砂の画質を選べます。
settings-wallpapers-awww-none = 画像は { $type } で切り替わります。
settings-wallpapers-awww-no-type = トランジションなし
settings-wallpapers-sound-muted = 壁紙の音声はミュートです。
settings-wallpapers-displays-say = { $state ->
        [none] ロック中の画面なし。
        [one] { $names } はロック中。
        [some] { $names } はロック中。
       *[detecting] 画面を検出中。
    }専用の配置がない画面は { $fill }。
settings-wallpapers-play-say = 画像は { $backend }、動画は { $val }Vulkan{ $end }、{ $layer } レイヤー。
settings-wallpapers-transitions-say = { $family } 系{ $kind ->
        [random] 、毎回違うものを { $duration } で。
        [preset] 、{ $shader } を { $ms } ms で。
       *[shader] 、{ $shader } を { $duration } で。
    }フレームレート { $rate }。
settings-wallpapers-primary-count = { $state ->
        [none] { $all } 個のうちなし
        [all] { $all } 個すべて
       *[some] { $all } 個のうち { $shown } 個
    }
settings-wallpapers-awww-say = 画像は { $type }、{ $filter } フィルター、{ $duration } で再生。
settings-wallpapers-drawing-say = { $state ->
        [on] 準備済みの壁紙は { $duration } で描かれます。
       *[off] オフ。ドローイング指定のカードは { $duration } で描かれます。
    }
settings-wallpapers-sound-say = 壁紙の音声は { $volume } で再生{ $duck ->
        [yes] { "" }、他のアプリの音で一時停止します。
       *[no] します。
    }
settings-wallpapers-pause-say = { $rules }で停止 ({ $scope }){ $apps ->
        [yes] 、一覧のアプリでも。
       *[no] 。
    }
settings-wallpapers-pause-rules = ルール { $count } 件
settings-wallpapers-plasma-say = { $state ->
        [off] ロック画面は { $mode }。
        [static] ロック画面は { $mode }: { $image }。
       *[follow] ロック画面は { $mode }。シーンは{ $scenes ->
        [live] ライブ描画です。
       *[poster] ポスターフレームです。
    }
    }
settings-wallpapers-niri-say = 表示: { $source ->
       *[current] 現在の壁紙
        [own] 専用のソース
        [stationary] 固定の壁紙
    }{ $blur ->
        [yes] 、ぼかし
       *[no] { "" }
    }、減光 { $dim }{ $auto ->
        [yes] 、再配色
       *[no] { "" }
    }。
settings-wallpapers-device-hint = 自動は CUDA、WebGPU、CPU の順に試します
settings-wallpapers-memory-hint = 上限を超えると自動は CPU でやり直します
