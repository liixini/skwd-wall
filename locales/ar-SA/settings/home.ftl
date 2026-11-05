settings-home-search = ابحث في كل الإعدادات: معدل الإطارات، الصوت، شاشة القفل، التمويه
settings-home-displays = { $count ->
    [one] شاشة واحدة
   *[other] { $count } شاشات
 }
settings-home-displays-say = { $link } { $count ->
    [one] تعرض الخلفيات.
   *[other] تعرض الخلفيات.
 }
settings-home-watching = { " " }مراقبة المكتبة: { $val }{ $state }{ $end }.
settings-home-model-ready = جاهز
settings-home-model-missing = لا يوجد نموذج
settings-home-model-off = متوقف
settings-home-colours-from = { " " }{ $link } مصدرها { $val }{ $engine }{ $end }.
settings-home-sound-desc = صوت خلفيات الفيديو وخلفيات Wallpaper Engine.
settings-home-sound-say = تعمل الخلفيات بمستوى { $volume }{ $duck ->
    [yes] { " " }وتتوقف مؤقتًا عندما تشغّل تطبيقات أخرى صوتًا.
   *[no] ، حتى عندما تشغّل تطبيقات أخرى صوتًا.
 }
settings-home-colours-desc = كيف تصل ألوان الخلفية إلى بقية سطح المكتب.
settings-home-colours-say = { settings-theme-policy-label }: { $policy }.{ " " }المزيد في { $place }.
settings-home-rotation-desc = تطبيق خلفية عشوائية كلما انتهت الفترة.
settings-home-rotation-say = خلفية عشوائية كل { $interval }.
settings-home-rotation-off = التدوير متوقف. تتغير الخلفية فقط عندما تختار واحدة.
settings-home-transitions-desc = كيف تتحول خلفية إلى التالية.
settings-home-transitions-say = تتحول الخلفية إلى التالية خلال { $duration }.
settings-home-transitions-off = الانتقالات متوقفة. تنتقل الخلفيات مباشرة إلى التالية.
settings-home-changed = تغييرات عن الافتراضي
settings-home-changed-desc = الإعدادات التي غيّرتها عن قيمها الافتراضية في مكان واحد، ولكل منها زر إعادة التعيين.
settings-home-changed-none = لا شيء يختلف عن القيمة الافتراضية ضمن الإعدادات المقارَنة هنا.
settings-home-changed-say = { $count ->
    [one] إعداد واحد يختلف عن قيمته الافتراضية.
   *[other] الإعدادات المختلفة عن قيمها الافتراضية: { $count }.
 }
settings-home-review = مراجعة
settings-home-reset = إعادة تعيين
settings-home-reset-all = إعادة تعيين الكل
settings-home-badge = تغييرات: { $count }
settings-home-automatic = تلقائي
settings-home-displays-none = لا توجد شاشات
settings-home-displays-none-say = { $link } تعرض الخلفيات.
settings-home-reset-scope = تعيد ضبط كل الإعدادات أعلاه، بما فيها اختصارات المفاتيح. تبقى مفاتيح API وأسماء المستخدمين والمواقع والمجلدات.
settings-home-items = { $count ->
    [one] عنصر واحد
   *[other] { $count } عناصر
 }
