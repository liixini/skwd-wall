settings-colours-source = 色の出どころ
settings-colours-source-desc = 壁紙ごとに追従、1 つのパレットを固定、または色を変更しません。
settings-colours-source-follow = 壁紙ごとに追従します
settings-colours-source-fixed = 固定パレットを使います
settings-colours-source-off = そのままです
settings-colours-source-say = 色は { $policy }。
settings-colours-source-made = 色は { $policy }。作成: { $authority }。
settings-colours-end4 = end-4
settings-colours-control-menu = 色の制御 · 検出されたシェル
settings-colours-generator-menu = ジェネレーター · インストール済みのみ
settings-colours-theme-menu = テーマ · プリセット、自作、カスタムシード
settings-colours-all-presets = { $count } 個のプリセットすべて…
settings-colours-generator = ジェネレーター
settings-colours-generator-desc = 壁紙をパレットに変換するプログラム。
settings-colours-generator-say = { $engine } がパレットを作成します。
settings-colours-generator-seeded = { $engine } がパレットを作成します。シードはスロット { $index }、コントラスト { $contrast }。
settings-colours-generator-shell = { $shell } が壁紙からパレットを作成します。
settings-colours-palette = パレット
settings-colours-palette-desc = すべての壁紙に 1 つのパレット。
settings-colours-palette-say = すべての壁紙に { $theme }。
settings-colours-idle = 色をそのままにしている間は使われません。
settings-colours-index-hint = シード色に使うパレットのスロット
settings-colours-contrast-hint = -1 から 1
settings-colours-wallust-hint = wallust パレットを指定 (例: dark16)。空ならモード次第
settings-colours-seeds-hint = カスタムシードで使用
settings-colours-config-hint = 外部 Matugen
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = 自動
settings-colours-look = 外観
settings-colours-look-desc = 色の広がり、仕上げ、明暗。
settings-colours-look-say = { $scheme } の性格、{ $finish } の仕上げ、バリアント { $variant }。
settings-colours-look-imported = { $shell } のパレットを { $variant } バリアントとして取り込みます。
settings-colours-look-shell = { $shell } が外観を決めます。
settings-colours-look-fixed = 固定パレットが外観を決めます。
settings-colours-no-designer = テーマデザイナーは「壁紙に追従」または「固定パレット」で使えます。
settings-colours-designer-hint = パレットを手動で調整または作成
settings-colours-go = 色の行き先
settings-colours-go-desc = パレットの変更に追従するアプリテーマ。
settings-colours-find-hint = アプリをインストールしたら再確認
settings-colours-matugen-hint = Matugen 出力のスイッチ
settings-colours-reload-hint = ファイルの書き込み後に実行
settings-colours-output-heading = 出力 { $count }
settings-colours-shells = シェル
settings-colours-shells-desc = Noctalia と Dank Material Shell は Skwd の色を受け取れます。
settings-colours-dms-hint = DMS は Auto (壁紙) が必要
settings-colours-mode-hint = 配色ごとに適用
settings-colours-bin-hint = PATH にない場合
settings-colours-lock = ロック画面
settings-colours-lock-desc = 壁紙を表示するロック画面。
settings-colours-lock-follow = 最後に適用した壁紙
settings-colours-lock-still = 静止画像
settings-colours-sddm-hint = 管理者の承認が必要
settings-colours-go-say = { $val }{ $all } 個中 { $on } 個のアプリ{ $end }が{ $followers ->
        [one] 色に
       *[some] 色に
    }追従します{ $changed ->
        [one] 。{ $val }{ $name } の設定が Skwd の外で変更されました{ $end }。
        [some] 。{ $val }{ $count } 個のアプリが Skwd の外で変更されました{ $end }。
       *[none] 。
    }
settings-colours-outputs-say = { $state ->
        [none] テンプレートを出力する連携はまだありません。
        [single] { $val }1 個の連携{ $end }が色の変更ごとにテンプレートをファイルへ出力します。
       *[many] { $val }{ $count } 個の連携{ $end }が色の変更ごとにテンプレートをファイルへ出力します。
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] Skwd の色に追従
       *[keeps] 独自の色を維持
    }{ $end }、DMS { $val }{ $dms ->
        [follows] Skwd の色に追従
       *[keeps] 独自の色を維持
    }{ $end }{ $caelestia ->
        [yes] 、Caelestia の壁紙は同期します。
       *[no] 。
    }
settings-colours-lock-say = { $state ->
        [none] ロック画面は未設定です。新しいものは { $mode } を使います。
        [one] { $names } は { $mode } を使います。
       *[some] { $names } は { $mode } を使います。
    }
settings-colours-lock-say-static = { $state ->
        [none] ロック画面は未設定です。新しいものは { $mode } を使います: { $image }。
        [one] { $names } は { $mode } を使います: { $image }。
       *[some] { $names } は { $mode } を使います: { $image }。
    }
settings-menu-more = その他
