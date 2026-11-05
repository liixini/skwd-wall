settings-colours-source = رنگ از کجا می‌آید
settings-colours-source-desc = هر پس‌زمینه را دنبال کن، یک پالت را نگه دار یا رنگ‌ها را تغییر نده.
settings-colours-source-follow = هر پس‌زمینه را دنبال می‌کنند
settings-colours-source-fixed = از یک پالت ثابت می‌آیند
settings-colours-source-off = همان‌طور می‌مانند
settings-colours-source-say = رنگ‌ها { $policy }.
settings-colours-source-made = رنگ‌ها { $policy }، ساخته‌ی { $authority }.
settings-colours-end4 = end-4
settings-colours-control-menu = کنترل رنگ · پوسته‌های شناسایی‌شده
settings-colours-generator-menu = سازنده · فقط نصب‌شده‌ها
settings-colours-theme-menu = تم · آماده، مال شما، بذرهای سفارشی
settings-colours-all-presets = همه‌ی { $count } تم آماده…
settings-colours-generator = سازنده
settings-colours-generator-desc = برنامه‌ای که پس‌زمینه را به پالت تبدیل می‌کند.
settings-colours-generator-say = پالت را { $engine } می‌سازد.
settings-colours-generator-seeded = پالت را { $engine } می‌سازد، با بذر خانه‌ی { $index } و کنتراست { $contrast }.
settings-colours-generator-shell = پالت را { $shell } از پس‌زمینه می‌سازد.
settings-colours-palette = پالت
settings-colours-palette-desc = یک پالت برای همه‌ی پس‌زمینه‌ها.
settings-colours-palette-say = برای همه‌ی پس‌زمینه‌ها { $theme }.
settings-colours-idle = تا وقتی رنگ‌ها تغییر نکنند استفاده نمی‌شود.
settings-colours-index-hint = کدام خانه‌ی پالت رنگ بذر باشد
settings-colours-contrast-hint = از -1 تا 1
settings-colours-wallust-hint = اجبار یک پالت wallust، مثل dark16. خالی: طبق حالت
settings-colours-seeds-hint = برای بذرهای سفارشی
settings-colours-config-hint = برنامه‌ی Matugen بیرونی
settings-colours-command-hint = ‎%config% %path% %scheme% %mode% %index%‎
settings-colours-auto = خودکار
settings-colours-look = ظاهر
settings-colours-look-desc = میزان پخش رنگ، پرداخت و روشن یا تیره.
settings-colours-look-say = شخصیت { $scheme } با پرداخت { $finish }، گونه‌ی { $variant }.
settings-colours-look-imported = پالت { $shell }، وارد شده با گونه‌ی { $variant }.
settings-colours-look-shell = ظاهر را { $shell } خودش تعیین می‌کند.
settings-colours-look-fixed = پالت ثابت ظاهرش را خودش تعیین می‌کند.
settings-colours-no-designer = طراح تم با دنبال کردن پس‌زمینه یا پالت ثابت در دسترس است.
settings-colours-designer-hint = پالت را دستی تنظیم کنید یا بسازید
settings-colours-go = رنگ‌ها به کجا می‌روند
settings-colours-go-desc = تم برنامه‌هایی که با تغییر پالت همراه می‌شوند.
settings-colours-find-hint = پس از نصب برنامه دوباره بررسی کنید
settings-colours-matugen-hint = کلید خروجی‌های Matugen
settings-colours-reload-hint = پس از نوشتن فایل اجرا می‌شود
settings-colours-output-heading = خروجی { $count }
settings-colours-shells = پوسته‌ها
settings-colours-shells-desc = پوسته‌های Noctalia و Dank Material Shell می‌توانند رنگ‌های Skwd را بگیرند.
settings-colours-dms-hint = برنامه‌ی DMS باید از Auto (پس‌زمینه) استفاده کند
settings-colours-mode-hint = با هر طرح رنگ اعمال می‌شود
settings-colours-bin-hint = اگر در PATH نیست
settings-colours-lock = صفحه‌های قفل
settings-colours-lock-desc = صفحه‌های قفلی که پس‌زمینه‌ی شما را نشان می‌دهند.
settings-colours-lock-follow = آخرین پس‌زمینه‌ی اعمال‌شده
settings-colours-lock-still = یک تصویر ثابت
settings-colours-sddm-hint = نیاز به تأیید مدیر
settings-colours-go-say = { $val }{ $on } از { $all } برنامه{ $end } { $followers ->
        [one] رنگ‌های شما را
       *[some] رنگ‌های شما را
    } دنبال می‌کنند{ $changed ->
        [one] ؛ { $val }پیکربندی { $name } بیرون از Skwd تغییر کرد{ $end }.
        [some] ؛ { $val }{ $count } برنامه بیرون از Skwd تغییر کردند{ $end }.
       *[none] .
    }
settings-colours-outputs-say = { $state ->
        [none] هنوز هیچ یکپارچه‌سازی قالبی نمی‌سازد.
        [single] { $val }۱ یکپارچه‌سازی{ $end } پس از هر تغییر رنگ قالبی را در فایل می‌نویسد.
       *[many] { $val }{ $count } یکپارچه‌سازی{ $end } پس از هر تغییر رنگ قالب‌ها را در فایل‌ها می‌نویسند.
    }
settings-colours-shells-say = برنامه‌ی Noctalia { $val }{ $noctalia ->
        [follows] رنگ‌های Skwd را دنبال می‌کند
       *[keeps] رنگ‌های خودش را نگه می‌دارد
    }{ $end }؛ برنامه‌ی DMS { $val }{ $dms ->
        [follows] رنگ‌های Skwd را دنبال می‌کند
       *[keeps] رنگ‌های خودش را نگه می‌دارد
    }{ $end }{ $caelestia ->
        [yes] ؛ پس‌زمینه‌های Caelestia همگام می‌شوند.
       *[no] .
    }
settings-colours-lock-say = { $state ->
        [none] هیچ صفحه‌ی قفلی تنظیم نشده. موارد جدید از { $mode } استفاده می‌کنند.
        [one] صفحه‌ی قفل { $names } از { $mode } استفاده می‌کند.
       *[some] صفحه‌های قفل { $names } از { $mode } استفاده می‌کنند.
    }
settings-colours-lock-say-static = { $state ->
        [none] هیچ صفحه‌ی قفلی تنظیم نشده. موارد جدید از { $mode } استفاده می‌کنند: { $image }.
        [one] صفحه‌ی قفل { $names } از { $mode } استفاده می‌کند: { $image }.
       *[some] صفحه‌های قفل { $names } از { $mode } استفاده می‌کنند: { $image }.
    }
settings-menu-more = بیشتر
