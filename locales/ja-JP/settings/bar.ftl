settings-bar-filter = フィルターバー
settings-bar-filter-desc = ピッカー上部の種類・並び替え・フィルターのボタン列。
settings-bar-filter-say = { $val }{ $buttons }{ $end }と{ $val }{ $sorts }{ $end }を{ $orientation ->
        [vertical] 縦
       *[horizontal] 横
    }に並べて表示します。{ $pinned ->
        [yes] 常に固定表示されます。
       *[no] 必要なときだけ表示されます。
    }表示スタイル：{ $style }。
settings-bar-button-count = { $all } 個中 { $shown } 個のボタン
settings-bar-sort-count = { $all } 件中 { $shown } 件の並び替え
settings-bar-offers = 表示するもの
settings-bar-behaves = 表示のしかた
settings-bar-buttons = バーのボタン
settings-bar-pinned-hint = 自動で隠さず固定表示
settings-bar-placed = バーの位置は「レイアウト」で設定します。
settings-bar-sizes = サイズ帯
settings-bar-sizes-desc = 「サイズ」ボタンで絞り込む解像度の範囲。
settings-bar-sizes-say = 「サイズ」ボタンの選択肢は { $bands } です。
settings-bar-sizes-none = サイズ帯がまだないため、「サイズ」ボタンには選択肢がありません。
settings-bar-unnamed = 名前なし
settings-bar-from = 最小
settings-bar-up-to = 最大
settings-bar-no-limit = 上限なし
settings-bar-add-hint = 用意された帯かカスタム範囲
settings-bar-start = ピッカーの開始位置
settings-bar-start-desc = ピッカーを開いたときに表示するフォルダー。
settings-bar-start-say = ピッカーは { $folder } で開き、{ $sticky ->
        [yes] 閉じた後もフィルターを保持します。
       *[no] 開くたびにフィルターをリセットします。
    }
settings-bar-sticky-hint = ピッカーを閉じても種類・フォルダー・色・並び替え・お気に入りのフィルターを保持
settings-bar-search-desc = タグは保存した語句と照合します。「説明で検索」は短い説明から画像を探します。
settings-bar-search-say = 検索は{ $mode ->
        [describe] 説明で検索
       *[tags] タグ
    }で開きます。説明検索は{ $views ->
        [multiview] 構図と切り抜き
       *[full] 画像全体
    }に { $model } を使います。
settings-bar-search-off = 検索は{ $mode ->
        [describe] 説明で検索
       *[tags] タグ
    }で開きます。説明検索はオフです。モデルは読み込まれず、インデックスも作られません。タグは使えます。
settings-bar-model-status = モデルの状態：{ $val }{ $status }{ $end }。
settings-bar-searching = 検索
settings-bar-unused = 未使用
settings-bar-name-hint = アクティブなモデルの一覧に表示
settings-bar-manual-hint = パックをその場所のまま参照
settings-bar-forget-hint = ファイルは残ります
settings-bar-delete-hint = セマンティック検索もオフになります
settings-menu-choose = 選択
settings-checks-count = { $all } 中 { $shown }
settings-bar-floats-hint = 配置で動かせます
settings-bar-delete-named = { $name } を削除
settings-bar-forget-named = { $name } を一覧から外す
