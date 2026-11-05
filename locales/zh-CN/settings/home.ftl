settings-home-search = 搜索所有设置：帧率、声音、锁屏、模糊
settings-home-displays = { $count ->
   *[other] { $count } 个显示器
 }
settings-home-displays-say = { $link }正在显示壁纸。
settings-home-watching = { " " }壁纸库监视：{ $val }{ $state }{ $end }。
settings-home-model-ready = 就绪
settings-home-model-missing = 无模型
settings-home-model-off = 关闭
settings-home-colours-from = { " " }{ $link }来自 { $val }{ $engine }{ $end }。
settings-home-sound-desc = 视频壁纸和 Wallpaper Engine 壁纸的音频。
settings-home-sound-say = 壁纸以 { $volume } 播放{ $duck ->
    [yes] ，其他应用播放声音时暂停。
   *[no] ，其他应用播放声音时也不暂停。
 }
settings-home-colours-desc = 壁纸颜色如何传递到桌面其他部分。
settings-home-colours-say = { settings-theme-policy-label }：{ $policy }。{ " " }更多内容见{ $place }。
settings-home-rotation-desc = 每当间隔结束时应用一张随机壁纸。
settings-home-rotation-say = 每 { $interval } 更换一张随机壁纸。
settings-home-rotation-off = 轮换已关闭。壁纸只在你选择时更改。
settings-home-transitions-desc = 一张壁纸如何过渡到下一张。
settings-home-transitions-say = 壁纸在 { $duration } 内过渡到下一张。
settings-home-transitions-off = 过渡已关闭。壁纸直接切换到下一张。
settings-home-changed = 与默认值不同
settings-home-changed-desc = 你从默认值更改过的设置都在这里，每项都可重置。
settings-home-changed-none = 此处比较的设置中，没有与默认值不同的项。
settings-home-changed-say = { $count ->
   *[other] { $count } 项设置与默认值不同。
 }
settings-home-review = 查看
settings-home-reset = 重置
settings-home-reset-all = 全部重置
settings-home-badge = 已更改 { $count } 项
settings-home-automatic = 自动
settings-home-displays-none = 没有显示器
settings-home-displays-none-say = { $link }正在显示壁纸。
settings-home-reset-scope = 重置上方所有设置，包括快捷键。API 密钥、用户名、位置和文件夹保持不变。
settings-home-items = { $count ->
   *[other] { $count } 项
 }
