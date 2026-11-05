settings-home-search = すべての設定を検索: フレームレート、サウンド、ロック画面、ぼかし
settings-home-displays = { $count ->
   *[other] { $count } 台のディスプレイ
 }
settings-home-displays-say = { $link }で壁紙を表示中です。
settings-home-watching = { " " }ライブラリの監視: { $val }{ $state }{ $end }。
settings-home-model-ready = 準備完了
settings-home-model-missing = モデルなし
settings-home-model-off = オフ
settings-home-colours-from = { " " }{ $link }の出どころ: { $val }{ $engine }{ $end }。
settings-home-sound-desc = 動画壁紙と Wallpaper Engine 壁紙の音声。
settings-home-sound-say = 壁紙は { $volume } で再生{ $duck ->
    [yes] され、他のアプリが音を出すと一時停止します。
   *[no] され、他のアプリが音を出しても続きます。
 }
settings-home-colours-desc = 壁紙の色がデスクトップ全体に届く仕組み。
settings-home-colours-say = { settings-theme-policy-label }: { $policy }。{ " " }詳しくは{ $place }へ。
settings-home-rotation-desc = 間隔が過ぎるたびにランダムな壁紙を適用します。
settings-home-rotation-say = { $interval } ごとにランダムな壁紙に変わります。
settings-home-rotation-off = ローテーションはオフです。壁紙は選んだときだけ変わります。
settings-home-transitions-desc = 壁紙が次の壁紙へ変わる方法。
settings-home-transitions-say = 壁紙は { $duration } で次の壁紙へ変わります。
settings-home-transitions-off = トランジションはオフです。壁紙はすぐに次へ切り替わります。
settings-home-changed = 既定値からの変更
settings-home-changed-desc = 既定値から変更した設定を一か所にまとめ、それぞれリセットできます。
settings-home-changed-none = ここで比較する設定に、既定値と異なるものはありません。
settings-home-changed-say = { $count ->
   *[other] { $count } 件の設定が既定値と異なります。
 }
settings-home-review = 確認
settings-home-reset = リセット
settings-home-reset-all = すべてリセット
settings-home-badge = { $count } 件変更
settings-home-automatic = 自動
settings-home-displays-none = どのディスプレイ
settings-home-displays-none-say = { $link }にも壁紙は表示されていません。
settings-home-reset-scope = 上の設定をすべてリセットします。キー操作も含みます。API キー、ユーザー名、場所、フォルダーはそのままです。
settings-home-items = { $count ->
   *[other] { $count } 件
 }
