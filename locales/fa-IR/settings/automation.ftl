settings-automation-rotation-desc = هر بار که فاصله تمام شود، یک کاغذدیواری تصادفی از مجموعه اعمال می‌کند.
settings-automation-rotation-say = هر { $interval }، انتخابی تصادفی از { $val }{ $pool }{ $end }.
settings-automation-rotation-off = خاموش. کاغذدیواری فقط وقتی عوض می‌شود که خودتان یکی را انتخاب کنید.
settings-automation-pool = { $count ->
        [none] مجموعه‌ای خالی
        [one] { $first }
        [two] { $first } و { $second }
       *[three] { $first }، { $second } و { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] { $pool } محبوب
       *[all] { $pool }
    }
settings-automation-pool-images = تصویرها
settings-automation-pool-videos = ویدیوها
settings-automation-pool-engine = موارد Wallpaper Engine
settings-automation-pool-heading = مجموعه شامل
settings-automation-timing = زمان
settings-automation-interval-hint = دست‌کم ۱۰ ثانیه
settings-automation-schedule-desc = قانون‌ها کاغذدیواری و پوسته را بر اساس زمان، روز هفته، آب‌وهوا، برق و نمایشگرها انتخاب می‌کنند. نخستین قانون منطبق برنده است.
settings-automation-schedule-say = { $start ->
        [applies] قانون‌ها روشن‌اند؛ نخستین قانون منطبق { $val }هنگام شروع اعمال می‌شود{ $end }. کاغذدیواری‌ای که خودتان انتخاب کنید همچنان اولویت دارد.
       *[waits] قانون‌ها روشن‌اند؛ نخستین قانون منطبق { $val }منتظر تغییر بعدی می‌ماند{ $end }. کاغذدیواری‌ای که خودتان انتخاب کنید همچنان اولویت دارد.
    }
settings-automation-schedule-off = خاموش. قانون‌های شما تا وقتی این را روشن کنید نگه داشته می‌شوند.
settings-automation-start-hint = نخستین قانون منطبق هنگام شروع سرویس
settings-automation-editor-hint = قانون‌ها را از بلوک‌های شرط بسازید
settings-automation-location = مکان و آب‌وهوا
settings-automation-location-desc = برای زمان طلوع و غروب خورشید و وضعیت آب‌وهوا به کار می‌رود.
settings-automation-location-say = آب‌وهوا برای { $city }، زمان‌های خورشید در { $lat }، { $lon }. تغییر با آب‌وهوا: { $mode }.
settings-automation-no-city = بدون شهر
settings-automation-no-latitude = بدون عرض جغرافیایی
settings-automation-no-longitude = بدون طول جغرافیایی
settings-automation-location-hint = برای استفاده از مختصات، شهر را خالی بگذارید. عرض جغرافیایی به سمت شمال و طول جغرافیایی به سمت شرق مثبت است.
settings-automation-startup = شروع و اعلان‌ها
settings-automation-startup-desc = آنچه هنگام شروع سرویس و هنگام تغییر کاغذدیواری رخ می‌دهد.
settings-automation-startup-say = { $restore ->
        [restores] هنگام شروع، Skwd { $val }آخرین کاغذدیواری را دوباره اعمال می‌کند{ $end }.
       *[leaves] هنگام شروع، Skwd { $val }میز کار را همان‌طور که هست می‌گذارد{ $end }.
    } { $notify ->
        [notifies] تغییرهای کاغذدیواری { $val }اعلان می‌فرستند{ $end }.
       *[silent] تغییرهای کاغذدیواری { $val }بی‌صدا انجام می‌شوند{ $end }.
    }
settings-automation-commands = پس از اعمال کاغذدیواری
settings-automation-commands-desc = پس از هر تغییر فرمان‌های خودتان را اجرا کنید، برای مثال برای راه‌اندازی ابزار کاغذدیواری دیگری.
settings-automation-command-count = { $count ->
        [one] { $val }{ $count } فرمان{ $end } پس از هر تغییر اجرا می‌شود.
       *[other] { $val }{ $count } فرمان{ $end } پس از هر تغییر اجرا می‌شوند.
    }
settings-automation-commands-say = { $listed ->
        [none] پس از هر تغییر { $val }هیچ فرمانی{ $end } اجرا نمی‌شود.
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }اعمال را به فرمان‌های شما می‌سپارد{ $end } و فرمان‌ها { $val }در بازیابی هنگام شروع هم اجرا می‌شوند{ $end }.
           *[skip] Skwd { $val }اعمال را به فرمان‌های شما می‌سپارد{ $end } و فرمان‌ها { $val }از بازیابی هنگام شروع می‌گذرند{ $end }.
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }کاغذدیواری را خودش اعمال می‌کند{ $end } و فرمان‌ها { $val }در بازیابی هنگام شروع هم اجرا می‌شوند{ $end }.
           *[skip] Skwd { $val }کاغذدیواری را خودش اعمال می‌کند{ $end } و فرمان‌ها { $val }از بازیابی هنگام شروع می‌گذرند{ $end }.
        }
    }
settings-automation-pick-only-hint = فرمان‌های شما اعمالش می‌کنند
settings-automation-commands-heading = فرمان‌ها
settings-automation-placeholders = در فرمان از %path%، %thumb%، %type% و %name% استفاده کنید.
settings-automation-minutes = { $count } دقیقه
settings-automation-mode-off = خاموش
settings-automation-mode-outdoors = هماهنگی با هوای بیرون
settings-automation-mode-playlists = انتخاب از فهرست‌های پخش
settings-automation-mode-semantic = جست‌وجو در کتابخانه
settings-automation-condition-clear = صاف
settings-automation-condition-cloudy = ابری
settings-automation-condition-rainy = بارانی
settings-automation-condition-snowy = برفی
settings-automation-condition-stormy = طوفانی
settings-automation-condition-foggy = مه‌آلود
settings-automation-condition-windy = بادی
settings-automation-display-another = یکی دیگر
settings-automation-display-exclude = حذف
settings-automation-display-keep = نگه‌داشتن
settings-automation-playlists-heading = فهرست پخش برای هر هوا
settings-automation-hours = { $count } ساعت
