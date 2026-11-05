settings-home-search = جستجوی همه تنظیمات: نرخ فریم، صدا، صفحه قفل، تاری
settings-home-displays = { $count ->
    [one] { $count } نمایشگر
   *[other] { $count } نمایشگر
 }
settings-home-displays-say = { $link } { $count ->
    [one] تصویر زمینه نشان می‌دهد.
   *[other] تصویر زمینه نشان می‌دهند.
 }
settings-home-watching = { " " }پایش کتابخانه: { $val }{ $state }{ $end }.
settings-home-model-ready = آماده
settings-home-model-missing = بدون مدل
settings-home-model-off = خاموش
settings-home-colours-from = { " " }{ $link } از { $val }{ $engine }{ $end } می‌آیند.
settings-home-sound-desc = صدای تصاویر زمینه ویدیویی و Wallpaper Engine.
settings-home-sound-say = تصاویر زمینه با { $volume } پخش می‌شوند{ $duck ->
    [yes] { " " }و وقتی برنامه‌های دیگر صدا پخش کنند مکث می‌کنند.
   *[no] ، حتی وقتی برنامه‌های دیگر صدا پخش می‌کنند.
 }
settings-home-colours-desc = رنگ‌های تصویر زمینه چگونه به بقیه میز کار می‌رسند.
settings-home-colours-say = { settings-theme-policy-label }: { $policy }.{ " " }بیشتر در { $place }.
settings-home-rotation-desc = با پایان هر بازه یک تصویر زمینه تصادفی اعمال می‌شود.
settings-home-rotation-say = هر { $interval } یک تصویر زمینه تصادفی.
settings-home-rotation-off = چرخش خاموش است. تصویر زمینه فقط با انتخاب شما عوض می‌شود.
settings-home-transitions-desc = یک تصویر زمینه چگونه به بعدی تبدیل می‌شود.
settings-home-transitions-say = یک تصویر زمینه در { $duration } به بعدی تبدیل می‌شود.
settings-home-transitions-off = گذارها خاموش‌اند. تصاویر زمینه مستقیم به بعدی می‌روند.
settings-home-changed = تغییرات نسبت به پیش‌فرض
settings-home-changed-desc = تنظیماتی که از پیش‌فرض تغییر داده‌اید در یک جا، هرکدام با بازنشانی.
settings-home-changed-none = در میان تنظیمات مقایسه‌شده در اینجا هیچ‌چیز با پیش‌فرض تفاوت ندارد.
settings-home-changed-say = { $count ->
    [one] { $count } تنظیم با پیش‌فرض خود فرق دارد.
   *[other] { $count } تنظیم با پیش‌فرض خود فرق دارند.
 }
settings-home-review = بازبینی
settings-home-reset = بازنشانی
settings-home-reset-all = بازنشانی همه
settings-home-badge = { $count } تغییر
settings-home-automatic = خودکار
settings-home-displays-none = هیچ نمایشگری
settings-home-displays-none-say = { $link } تصویر زمینه نشان نمی‌دهد.
settings-home-reset-scope = همه تنظیمات بالا را بازنشانی می‌کند، از جمله میان‌برها. کلیدهای API، نام‌های کاربری، مکان‌ها و پوشه‌ها می‌مانند.
settings-home-items = { $count ->
    [one] { $count } مورد
   *[other] { $count } مورد
 }
