settings-library-watch-section-desc = يكتشف الملفات المضافة من خارج skwd-wall ويبقي المكتبة محدّثة.
settings-library-watch-fallback-label = الفحص الدوري الاحتياطي
settings-library-watch-fallback-desc = يفحص فقط مجلدات المكتبة التي تعجز مراقبة الملفات الأصلية عن مراقبتها. فعّله لنقاط تركيب الشبكة أو FUSE التي تفوّت التغييرات، ثم أعد تشغيل skwd-walld.
settings-library-watch-interval-label = فترة الفحص الدوري
settings-library-watch-interval-desc = ينتظر هذا العدد من الثواني بين الفحوصات المحدودة. القيم الأقل تجد التغييرات أسرع لكنها تقرأ نظام الملفات أكثر. أعد تشغيل skwd-walld بعد تغييره.
settings-library-watch-unknown-label = حالة المراقبة غير متاحة
settings-library-watch-unknown-desc = لا تبلّغ هذه الخدمة عن حالة مراقبة المكتبة. حدّث skwd-walld أو أعد تشغيله.
settings-library-watch-poll-failed-label = يتعذر على الفحص الدوري قراءة مجلد مكتبة
settings-library-watch-poll-failed-desc = تأكد من أن كل مجلد مكتبة مضبوط مركّب وقابل للقراءة. سيعيد الفحص الدوري المحاولة بعد { $interval } ثانية.
settings-library-watch-polling-label = الفحص الدوري الاحتياطي نشط
settings-library-watch-polling-desc = تعذّرت المراقبة الأصلية في { $count ->
    [zero] صفر من مجلدات المكتبة
    [one] مجلد مكتبة واحد
    [two] مجلدي مكتبة
    [few] { $count } مجلدات مكتبة
    [many] { $count } مجلدًا من مجلدات المكتبة
   *[other] { $count } مجلدٍ من مجلدات المكتبة
    }. يُفحص ما يصل إلى { $budget } إدخال كل { $interval } ثانية. آخر مزامنة ناجحة: { $convergence }.
settings-library-watch-recovering-label = تعافت المراقبة الأصلية
settings-library-watch-recovering-desc = المراقب الأصلي نشط من جديد. لا يزال فحص التسليم الكامل جاريًا قبل اعتبار المكتبة محدّثة.
settings-library-watch-unavailable-label = مراقبة المكتبة غير متاحة
settings-library-watch-unavailable-desc = تعذّرت مراقبة الملفات الأصلية والفحص الدوري الاحتياطي متوقف. فعّل «الفحص الدوري الاحتياطي»، ثم أعد تشغيل skwd-walld.
settings-library-watch-recovered-label = استُعيدت المراقبة الأصلية
settings-library-watch-recovered-desc = المراقب الأصلي وفحص التسليم الخاص به محدّثان. آخر مزامنة ناجحة: { $convergence }.
settings-library-watch-native-label = مراقبة الملفات الأصلية
settings-library-watch-native-desc = أحداث نظام الملفات نشطة لكل مجلد مكتبة. الفحص الدوري خامل.
settings-library-watch-convergence-never = لم تكتمل بعد
settings-library-watch-convergence-seconds = { $value ->
    [zero] الآن
    [one] منذ ثانية واحدة
    [two] منذ ثانيتين
    [few] منذ { $value } ثوانٍ
    [many] منذ { $value } ثانيةً
   *[other] منذ { $value } ثانيةٍ
    }
settings-library-watch-convergence-minutes = { $value ->
    [zero] منذ أقل من دقيقة
    [one] منذ دقيقة واحدة
    [two] منذ دقيقتين
    [few] منذ { $value } دقائق
    [many] منذ { $value } دقيقةً
   *[other] منذ { $value } دقيقةٍ
    }
settings-library-watch-convergence-hours = { $value ->
    [zero] منذ أقل من ساعة
    [one] منذ ساعة واحدة
    [two] منذ ساعتين
    [few] منذ { $value } ساعات
    [many] منذ { $value } ساعةً
   *[other] منذ { $value } ساعةٍ
    }
settings-profiles-card = الملفات الشخصية
settings-profiles-card-desc = حدّد المجلدات التي يستخدمها المنتقي ومرشح المجلدات والخلفيات العشوائية والجداول. تعود المجلدات المخفية بمفتاح المخفي في الشريط أو اختصاره، وتبدأ كل مرة تشغيل وهي مخفية من جديد.
settings-profiles-everything = الكل
settings-profiles-name-label = الاسم
settings-profiles-name-placeholder = اسم الملف الشخصي
settings-profiles-state-show = إظهار
settings-profiles-state-hide = إخفاء
settings-profiles-state-exclude = استبعاد
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = عناصر Workshop ليست في مجلدات، لذا ينطبق هذا عليها كلها. أخفِ خلفية واحدة من ظهر بطاقتها.
settings-profiles-summary = { $hidden } مخفية، { $excluded } مستبعدة
settings-profiles-summary-none = تُظهر كل المجلدات
settings-profiles-delete-action = حذف الملف الشخصي
settings-profiles-add-label = ملف شخصي جديد
settings-profiles-new-name = ملف شخصي جديد
settings-profiles-tree-label = مجلدات هذا الملف الشخصي
settings-profiles-tree-desc = إظهار يُبقي المجلد في المنتقي والعشوائي. إخفاء يبعده عن الأنظار حتى تضغط مفتاح المخفي في الشريط أو اختصاره. استبعاد يُخرجه من الملف الشخصي فيتجاوزه المفتاح. يضبط المجلد مجلداته الفرعية أيضًا؛ غيّر مجلدًا فرعيًا لاحقًا لاستثنائه.
settings-profiles-never-scanned = لا يُفحص أبدًا
settings-profiles-mixed = محتوى مختلط
settings-profiles-hidden-inside = { $count ->
    [zero] لا شيء مخفي بالداخل
    [one] عنصر واحد مخفي بالداخل
    [two] عنصران مخفيان بالداخل
    [few] { $count } عناصر مخفية بالداخل
    [many] { $count } عنصرًا مخفيًا بالداخل
   *[other] { $count } عنصر مخفي بالداخل
    }
settings-profiles-copy-action = نسخ
settings-profiles-helm-desc = يجعله الملف الشخصي النشط، فيتبعه التدوير والجداول.
settings-profiles-flag-desc = يفتح منتقيًا واحدًا في هذا الملف الشخصي ويترك النشط كما هو.
settings-profiles-make-active = تفعيل
settings-profiles-active-tag = نشط
settings-profiles-builtin-tag = مضمّن
settings-profiles-filter-placeholder = تصفية المجلدات
settings-profiles-no-match = لا يوجد مجلد مطابق.
settings-profiles-switch-label = التبديل إلى هذا الملف
settings-profiles-switch-desc = يغيّر skwd-helm الملف المحفوظ، فيتبعه التدوير والجداول. تفتح راية التشغيل منتقيًا واحدًا بهذا الملف وتترك المحفوظ كما هو.
settings-profiles-hidden-note-label = الخلفيات المخفية
settings-profiles-hidden-note-desc = يُظهر زر الإخفاء في الشريط (Ctrl+H) المجلدات المخفية للملف النشط وكل خلفية أُخفيت من خلف بطاقتها. يعود كل تشغيل لإخفائها.
settings-profiles-tree-note = «لم يُفحص قط» يأتي من المجلدات المستبعدة في المكتبة › المجلدات، فلا يستطيع أي ملف إظهاره. الاستبعاد هنا يترك المجلد خارج هذا الملف فقط. الخلفيات المخفية من بطاقتها تبقى مخفية في كل ملف، بما فيه «الكل»، حتى «إظهار المخفي». المجلدات المنقوطة مثل .private مجلدات عادية هنا.
settings-profiles-legend-show = في المنتقي والعشوائي.
settings-profiles-legend-hide = حتى زر الإخفاء في الشريط أو Ctrl+H. ليس عشوائيًا أبدًا.
settings-profiles-legend-exclude = من هذا الملف. «إظهار المخفي» يتخطاه.
settings-profiles-everything-desc = «الكل» يُظهر كل مجلد مفحوص وكل عنصر Workshop. أنشئ ملفًا جديدًا لإخفاء بعضها أو استبعاده.
settings-profiles-delete-hint = انقر مرة أخرى للحذف. تبقى الخلفيات والمجلدات في مكانها.
