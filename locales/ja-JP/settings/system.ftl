settings-system-power-desc = ピッカーと壁紙には別々のバッテリー制限があります。
settings-system-power-say = バッテリー時は{ $cap ->
        [none] ピッカーに追加の上限はなく（{ $fps }）
       *[cap] ピッカーは { $fps } に制限され
    }{ $idle ->
        [off] 、通常のアイドル時の一時停止のみが適用され（{ $seconds }）
       *[on] 、入力がないと { $seconds } で動画が止まり
    }、{ $detail ->
        [full] 壁紙は詳細をそのまま保ちます
       *[performance] 壁紙はパフォーマンスモードに切り替わります
    }。
settings-system-power-off = オフ。バッテリー時も、ピッカーと壁紙は AC 電源時と同じ制限で動作します。
settings-system-power-source = 現在の電源: { $source }
settings-system-always = 常時
settings-system-on-battery = バッテリー時 · セーバーがオンのとき
settings-system-max-fps-hint = リフレッシュレート、PC を涼しく保つなら 60
settings-system-idle-hint = 0 で無効
settings-system-battery-fps-hint = これと最大 FPS の低いほう · 0 で無効
settings-system-battery-idle-hint = 短いほうが優先 · 0 で無効
settings-system-battery-wallpaper-hint = 動画を直接再生、シーンは 30 fps
settings-system-gpu-desc = 変更後はピッカーを開き直してください。
settings-system-gpu-say = { $mode ->
        [low] ピッカーは { $gpu } を使います: 内蔵 GPU を要求します。
        [high] ピッカーは { $gpu } を使います: ディスクリート GPU を要求します。
        [none] ピッカーは { $gpu } を使います: ドライバーが決めます。
        [auto] ピッカーは { $gpu } を使います: バッテリーセーバー中は低消費電力 GPU を優先。
       *[custom] ピッカーは { $gpu } を使いますが、認識できないため自動として動作します。
    }
settings-system-language-desc = 変更はすぐに適用され、保存されます。
settings-system-language-say = { $follow ->
        [desktop] 表示言語は { $language } (デスクトップに従う)。壁紙の名前とタグは変わりません。
       *[chosen] 表示言語は { $language } です。壁紙の名前とタグは変わりません。
    }
settings-system-bug-title = バグレポート
settings-system-bug-desc = Issue に添付する 1 つのファイル。
settings-system-bug-say = バージョン、環境、最近のログを 1 つのファイルにまとめます。
