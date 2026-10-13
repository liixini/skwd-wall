settings-library-watch-section-desc = فایل‌هایی را که بیرون از skwd-wall افزوده می‌شوند شناسایی کنید تا کتابخانه به‌روز بماند.
settings-library-watch-fallback-label = پشتیبان بررسی دوره‌ای
settings-library-watch-fallback-desc = فقط پوشه‌های کتابخانه‌ای را بررسی کنید که پایش بومی فایل‌ها نمی‌تواند آن‌ها را بپاید. این گزینه را برای اتصال‌های شبکه‌ای یا FUSE که تغییرات را از دست می‌دهند فعال کنید، سپس skwd-walld را دوباره راه‌اندازی کنید.
settings-library-watch-interval-label = فاصلهٔ بررسی دوره‌ای
settings-library-watch-interval-desc = فاصلهٔ بین بررسی‌های محدود، بر حسب ثانیه. مقدار کمتر تغییرات را زودتر پیدا می‌کند، اما بیشتر از سامانهٔ فایل می‌خواند. پس از تغییر، skwd-walld را دوباره راه‌اندازی کنید.
settings-library-watch-unknown-label = وضعیت پایشگر در دسترس نیست
settings-library-watch-unknown-desc = این دیمن وضعیت پایش کتابخانه را گزارش نمی‌کند. skwd-walld را به‌روزرسانی یا دوباره راه‌اندازی کنید.
settings-library-watch-poll-failed-label = بررسی دوره‌ای نمی‌تواند پوشهٔ کتابخانه را بخواند
settings-library-watch-poll-failed-desc = مطمئن شوید همهٔ پوشه‌های کتابخانهٔ تنظیم‌شده متصل و خواندنی‌اند. بررسی دوره‌ای پس از { $interval } ثانیه دوباره تلاش می‌کند.
settings-library-watch-polling-label = پشتیبان بررسی دوره‌ای فعال است
settings-library-watch-polling-desc = پایش بومی برای { $count ->
    [one] { $count } پوشهٔ کتابخانه
   *[other] { $count } پوشهٔ کتابخانه
    } ناموفق بود. هر { $interval } ثانیه حداکثر { $budget } ورودی بررسی می‌شود. آخرین همگام‌سازی موفق: { $convergence }.
settings-library-watch-recovering-label = پایش بومی بازیابی شد
settings-library-watch-recovering-desc = پایشگر بومی دوباره فعال است. پیش از آنکه کتابخانه به‌روز اعلام شود، یک پویش کامل انتقالی همچنان در حال اجرا است.
settings-library-watch-unavailable-label = پایش کتابخانه در دسترس نیست
settings-library-watch-unavailable-desc = پایش بومی فایل‌ها ناموفق بود و پشتیبان بررسی دوره‌ای خاموش است. آن را فعال و سپس skwd-walld را دوباره راه‌اندازی کنید.
settings-library-watch-recovered-label = پایش بومی بازگردانده شد
settings-library-watch-recovered-desc = پایشگر بومی و پویش انتقالی آن به‌روز هستند. آخرین همگام‌سازی موفق: { $convergence }.
settings-library-watch-native-label = پایش بومی فایل‌ها
settings-library-watch-native-desc = رویدادهای سامانهٔ فایل برای همهٔ پوشه‌های کتابخانه فعال‌اند. بررسی دوره‌ای غیرفعال است.
settings-library-watch-convergence-never = هنوز کامل نشده
settings-library-watch-convergence-seconds = { $value ->
    [one] { $value } ثانیه پیش
   *[other] { $value } ثانیه پیش
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] { $value } دقیقه پیش
   *[other] { $value } دقیقه پیش
    }
settings-library-watch-convergence-hours = { $value ->
    [one] { $value } ساعت پیش
   *[other] { $value } ساعت پیش
    }
settings-profiles-card = نمایه‌ها
settings-profiles-card-desc = تعیین کنید انتخابگر، فیلتر پوشه، تصاویر زمینه تصادفی و زمان‌بندی‌ها از کدام پوشه‌ها استفاده کنند. پوشه‌های پنهان با کلید پنهان‌ها در نوار یا میانبر آن برمی‌گردند و هر اجرا دوباره با پنهان بودن آن‌ها شروع می‌شود.
settings-profiles-everything = همه
settings-profiles-name-label = نام
settings-profiles-name-placeholder = نام نمایه
settings-profiles-state-show = نمایش
settings-profiles-state-hide = پنهان
settings-profiles-state-exclude = کنار گذاشتن
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = موارد Workshop پوشه ندارند، پس این برای همه آن‌ها اعمال می‌شود. یک تصویر زمینه را از پشت کارتش پنهان کنید.
settings-profiles-summary = { $hidden } پنهان، { $excluded } کنارگذاشته
settings-profiles-summary-none = همه پوشه‌ها نمایش داده می‌شوند
settings-profiles-delete-action = حذف نمایه
settings-profiles-add-label = نمایه جدید
settings-profiles-new-name = نمایه جدید
settings-profiles-tree-label = پوشه‌های این نمایه
settings-profiles-tree-desc = نمایش پوشه را در انتخابگر و تصادفی نگه می‌دارد. پنهان آن را تا زدن کلید پنهان‌ها در نوار یا میانبر آن از دید دور نگه می‌دارد. کنار گذاشتن آن را از نمایه بیرون می‌گذارد، پس کلید از آن می‌گذرد. پوشه زیرپوشه‌هایش را هم تنظیم می‌کند؛ برای استثنا بعداً یک زیرپوشه را تغییر دهید.
settings-profiles-never-scanned = هرگز پویش نمی‌شود
settings-profiles-mixed = درونش مخلوط است
settings-profiles-hidden-inside = { $count ->
    [one] { $count } پنهان درون
   *[other] { $count } پنهان درون
    }
settings-profiles-copy-action = کپی
settings-profiles-helm-desc = این را نمایه فعال می‌کند تا چرخش و زمان‌بندی‌ها از آن پیروی کنند.
settings-profiles-flag-desc = یک انتخابگر در این نمایه باز می‌کند و نمایه فعال را دست نمی‌زند.
settings-profiles-make-active = فعال کردن
settings-profiles-active-tag = فعال
settings-profiles-builtin-tag = داخلی
settings-profiles-filter-placeholder = فیلتر پوشه‌ها
settings-profiles-no-match = هیچ پوشه‌ای مطابقت ندارد.
settings-profiles-switch-label = رفتن به این نمایه
settings-profiles-switch-desc = skwd-helm نمایهٔ ذخیره‌شده را عوض می‌کند، پس چرخش و زمان‌بندی‌ها از آن پیروی می‌کنند. پرچم راه‌اندازی یک انتخابگر در این نمایه باز می‌کند و نمایهٔ ذخیره‌شده را دست نمی‌زند.
settings-profiles-hidden-note-label = تصاویر پنهان
settings-profiles-hidden-note-desc = دکمهٔ پنهان در نوار (Ctrl+H) پوشه‌های پنهان نمایهٔ فعال و هر تصویری را که از پشت کارتش پنهان شده نشان می‌دهد. هر راه‌اندازی دوباره آن‌ها را پنهان می‌کند.
settings-profiles-tree-note = «هرگز پویش نشده» از پوشه‌های مستثنا در کتابخانه › پوشه‌ها می‌آید، پس هیچ نمایه‌ای نمی‌تواند آن را نشان دهد. مستثنا کردن در اینجا فقط پوشه را از این نمایه بیرون می‌گذارد. تصاویری که از کارتشان پنهان شده‌اند در هر نمایه، از جمله «همه»، تا «نمایش پنهان‌ها» پنهان می‌مانند. پوشه‌های نقطه‌دار مانند .private اینجا پوشه‌های معمولی‌اند.
settings-profiles-legend-show = در انتخابگر و تصادفی.
settings-profiles-legend-hide = تا دکمهٔ پنهان در نوار یا Ctrl+H. هرگز تصادفی نیست.
settings-profiles-legend-exclude = از این نمایه. «نمایش پنهان‌ها» از آن می‌گذرد.
settings-profiles-everything-desc = «همه» هر پوشهٔ پویش‌شده و هر مورد Workshop را نشان می‌دهد. برای پنهان یا مستثنا کردن برخی، نمایهٔ جدیدی بسازید.
settings-profiles-delete-hint = برای حذف دوباره کلیک کنید. تصاویر و پوشه‌ها سر جایشان می‌مانند.
