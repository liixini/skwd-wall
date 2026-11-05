settings-library-folders-desc = これらのフォルダーとサブフォルダーから画像と動画を探します。
settings-library-folders-say = { $val }{ $folders }{ $end }をスキャン中です。{ $state ->
        [polling] ポーリングが { $every } ごとにネイティブ監視の漏れを補います。{ $unread }
        [native] ネイティブ監視が動作中で、ポーリングは待機中です。
        [down] ネイティブ監視に失敗し、ポーリングはオフです。
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] 1 つのフォルダーを読み取れません。
       *[other] { $count } 個のフォルダーを読み取れません。
    }
settings-library-watching = 監視
settings-library-poll-hint = ネットワークや FUSE フォルダー用
settings-library-interval-hint = 変更後は skwd-walld を再起動
settings-library-excluded-desc = これらのフォルダーとサブフォルダーをスキップします。既存のメタデータは保持します。
settings-library-excluded-say = { $val }{ $folders }{ $end }は常にスキャンしません。{ $outside }
settings-library-outside = { $count ->
        [one] 1 つはどの壁紙フォルダーの外にもあり、何もスキップしません。
       *[other] { $count } 個はどの壁紙フォルダーの外にもあり、何もスキップしません。
    }
settings-library-excluded-hint = 除外フォルダーはスキャンされないため、どのプロファイルにも表示できません。保持したまま人前で隠すには { $link } を使います。
settings-library-profiles-desc = ピッカー、フォルダーフィルター、ランダム壁紙、スケジュールが使うフォルダーを決めます。
settings-library-profiles-say = 有効なプロファイル { $state ->
        [none] { $menu }、すべてのフォルダーを表示します。
       *[some] { $menu }:{ $summary }。
    }Ctrl+H またはバーの非表示トグルで、カードから隠したものも含め非表示の壁紙を次回起動まで表示します。
settings-library-active = 有効なプロファイル
settings-library-terminal = ターミナルから切り替え
settings-library-tree-note = 非表示は Ctrl+H までランダムから外れます。除外は Ctrl+H でも出ません。フォルダーの設定はサブフォルダーにも及びます。
settings-library-show-tip = ピッカーとランダムに表示
settings-library-hide-tip = Ctrl+H まで、ランダムなし
settings-library-exclude-tip = このプロファイルから除外、Ctrl+H も飛ばす
settings-library-from = 元: { $link }
settings-library-excluded-inside = 内部で { $count } 個除外
settings-library-both-inside = 内部で { $hidden } 個非表示、{ $excluded } 個除外
settings-library-storage = ストレージとごみ箱
settings-library-storage-desc = 小さな WebP コピー、復元用ストレージ、再着色コピー。
settings-library-storage-say = { $optimise ->
        [on] 新しい画像は品質 { $quality }、上限 { $size } の WebP になります。
       *[off] 新しい画像はそのままです。すべて最適化は品質 { $quality }、上限 { $size } を使います。
    }{ $trash ->
        [on] ごみ箱の元画像は { $days } 保持され、その後削除されます。
       *[off] ごみ箱の元画像は削除するまで保持されます。
    }{ $recolour ->
        [on] 新しい壁紙には再着色コピーも作ります。
       *[off] 新しい壁紙は再着色しません。
    }
settings-library-retention-hint = ごみ箱の画像を保持する日数
settings-library-optimise-hint = ライブラリ全体の画像を最適化
settings-library-recolour = 再着色
settings-library-thumbnails = サムネイルと修復
settings-library-thumbnails-desc = 表示がおかしいときにプレビューと色データを再構築します。
settings-library-thumbnails-say = 再構築時は同時に最大 { $jobs } 個のサムネイルを作ります。
settings-library-rebuild = 再構築
settings-library-start-over = やり直す
settings-library-recompute-hint = 既存のサムネイルから
settings-library-clear-hint = 次回スキャンで再生成します
