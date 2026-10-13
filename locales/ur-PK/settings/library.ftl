settings-library-watch-section-desc = skwd-wall سے باہر شامل کی گئی فائلیں پہچانیں اور لائبریری کو تازہ رکھیں۔
settings-library-watch-fallback-label = پولنگ فال بیک
settings-library-watch-fallback-desc = صرف وہ لائبریری فولڈر جانچیں جنہیں نیٹو فائل واچنگ نہیں دیکھ سکتی۔ نیٹ ورک یا FUSE ماؤنٹس کے لیے جو تبدیلیاں چھوڑ دیتے ہیں اسے فعال کریں، پھر skwd-walld دوبارہ شروع کریں۔
settings-library-watch-interval-label = پولنگ کا وقفہ
settings-library-watch-interval-desc = محدود جانچوں کے درمیان اتنے سیکنڈ انتظار کریں۔ کم قدریں تبدیلیاں جلد پکڑتی ہیں مگر فائل سسٹم زیادہ بار پڑھتی ہیں۔ بدلنے کے بعد skwd-walld دوبارہ شروع کریں۔
settings-library-watch-unknown-label = واچر کی حالت دستیاب نہیں
settings-library-watch-unknown-desc = یہ ڈیمن لائبریری واچ کی حالت رپورٹ نہیں کرتا۔ skwd-walld اپ ڈیٹ یا دوبارہ شروع کریں۔
settings-library-watch-poll-failed-label = پولنگ ایک لائبریری فولڈر نہیں پڑھ سکتی
settings-library-watch-poll-failed-desc = جانچیں کہ ہر ترتیب دیا گیا لائبریری فولڈر ماؤنٹ اور پڑھنے کے قابل ہے۔ پولنگ { $interval } سیکنڈ میں دوبارہ کوشش کرے گی۔
settings-library-watch-polling-label = پولنگ فال بیک فعال
settings-library-watch-polling-desc = نیٹو واچنگ { $count ->
    [one] { $count } لائبریری فولڈر
   *[other] { $count } لائبریری فولڈروں
    } کے لیے ناکام ہوئی۔ ہر { $interval } سیکنڈ میں زیادہ سے زیادہ { $budget } اندراج جانچے جاتے ہیں۔ آخری کامیاب ہم آہنگی: { $convergence }۔
settings-library-watch-recovering-label = نیٹو واچنگ بحال ہو گئی
settings-library-watch-recovering-desc = نیٹو واچر دوبارہ فعال ہے۔ لائبریری کو تازہ قرار دینے سے پہلے مکمل ہینڈ آف اسکین ابھی چل رہا ہے۔
settings-library-watch-unavailable-label = لائبریری واچنگ دستیاب نہیں
settings-library-watch-unavailable-desc = نیٹو فائل واچنگ ناکام ہوئی اور پولنگ فال بیک بند ہے۔ پولنگ فال بیک فعال کریں، پھر skwd-walld دوبارہ شروع کریں۔
settings-library-watch-recovered-label = نیٹو واچنگ بحال
settings-library-watch-recovered-desc = نیٹو واچر اور اس کا ہینڈ آف اسکین تازہ ہیں۔ آخری کامیاب ہم آہنگی: { $convergence }۔
settings-library-watch-native-label = نیٹو فائل واچنگ
settings-library-watch-native-desc = ہر لائبریری فولڈر کے لیے فائل سسٹم ایونٹس فعال ہیں۔ پولنگ غیر فعال ہے۔
settings-library-watch-convergence-never = ابھی مکمل نہیں ہوئی
settings-library-watch-convergence-seconds = { $value ->
    [one] { $value } سیکنڈ پہلے
   *[other] { $value } سیکنڈ پہلے
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] { $value } منٹ پہلے
   *[other] { $value } منٹ پہلے
    }
settings-library-watch-convergence-hours = { $value ->
    [one] { $value } گھنٹہ پہلے
   *[other] { $value } گھنٹے پہلے
    }
settings-profiles-card = پروفائلز
settings-profiles-card-desc = طے کریں کہ پِکر، فولڈر فلٹر، بے ترتیب وال پیپر اور شیڈول کون سے فولڈر استعمال کریں۔ چھپے فولڈر بار میں چھپے ہوئے ٹوگل یا اس کی شارٹ کٹ سے واپس آتے ہیں، اور ہر بار کھلنے پر وہ دوبارہ چھپے ہوتے ہیں۔
settings-profiles-everything = سب
settings-profiles-name-label = نام
settings-profiles-name-placeholder = پروفائل کا نام
settings-profiles-state-show = دکھائیں
settings-profiles-state-hide = چھپائیں
settings-profiles-state-exclude = خارج کریں
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = Workshop آئٹمز کے فولڈر نہیں ہوتے، اس لیے یہ سب پر لاگو ہوتا ہے۔ کسی ایک وال پیپر کو اس کے کارڈ کی پشت سے چھپائیں۔
settings-profiles-summary = { $hidden } چھپے، { $excluded } خارج
settings-profiles-summary-none = تمام فولڈر دکھاتا ہے
settings-profiles-delete-action = پروفائل حذف کریں
settings-profiles-add-label = نیا پروفائل
settings-profiles-new-name = نیا پروفائل
settings-profiles-tree-label = اس پروفائل کے فولڈر
settings-profiles-tree-desc = دکھائیں فولڈر کو پِکر اور بے ترتیب میں رکھتا ہے۔ چھپائیں اسے بار کے چھپے ہوئے ٹوگل یا اس کی شارٹ کٹ دبانے تک نظروں سے دور رکھتا ہے۔ خارج کریں اسے پروفائل سے باہر رکھتا ہے، اس لیے ٹوگل اسے چھوڑ دیتا ہے۔ فولڈر اپنے ذیلی فولڈر بھی طے کرتا ہے؛ استثنا کے لیے بعد میں کوئی ذیلی فولڈر بدلیں۔
settings-profiles-never-scanned = کبھی اسکین نہیں ہوتا
settings-profiles-mixed = اندر ملا جلا
settings-profiles-hidden-inside = { $count ->
    [one] اندر { $count } چھپا
   *[other] اندر { $count } چھپے
    }
settings-profiles-copy-action = کاپی کریں
settings-profiles-helm-desc = اسے فعال پروفائل بناتا ہے، اس لیے گردش اور شیڈول اس پر چلتے ہیں۔
settings-profiles-flag-desc = اس پروفائل میں ایک پِکر کھولتا ہے اور فعال پروفائل کو ویسا ہی رہنے دیتا ہے۔
settings-profiles-make-active = فعال کریں
settings-profiles-active-tag = فعال
settings-profiles-builtin-tag = بلٹ اِن
settings-profiles-filter-placeholder = فولڈر چھانٹیں
settings-profiles-no-match = کوئی فولڈر مماثل نہیں۔
settings-profiles-switch-label = اس پروفائل پر جائیں
settings-profiles-switch-desc = skwd-helm محفوظ شدہ پروفائل بدلتا ہے، اس لیے روٹیشن اور شیڈول اسی کی پیروی کرتے ہیں۔ لانچ فلیگ اس پروفائل میں ایک پکر کھولتا ہے اور محفوظ شدہ پروفائل کو ویسا ہی رہنے دیتا ہے۔
settings-profiles-hidden-note-label = چھپے وال پیپر
settings-profiles-hidden-note-desc = بار کا چھپائیں ٹوگل (Ctrl+H) فعال پروفائل کے چھپے فولڈر اور کارڈ کی پشت سے چھپایا گیا ہر وال پیپر دکھاتا ہے۔ ہر لانچ پر وہ دوبارہ چھپ جاتے ہیں۔
settings-profiles-tree-note = کبھی اسکین نہیں، لائبریری › فولڈر کے خارج کردہ فولڈروں سے آتا ہے، اس لیے کوئی پروفائل اسے نہیں دکھا سکتا۔ یہاں خارج کرنا صرف فولڈر کو اس پروفائل سے باہر رکھتا ہے۔ کارڈ سے چھپائے گئے وال پیپر ہر پروفائل میں، سب کچھ سمیت، چھپے دکھائیں تک چھپے رہتے ہیں۔ .private جیسے ڈاٹ فولڈر یہاں عام فولڈر ہیں۔
settings-profiles-legend-show = پکر اور رینڈم میں۔
settings-profiles-legend-hide = بار کے چھپائیں ٹوگل یا Ctrl+H تک۔ کبھی رینڈم نہیں۔
settings-profiles-legend-exclude = اس پروفائل سے باہر۔ چھپے دکھائیں اسے چھوڑ دیتا ہے۔
settings-profiles-everything-desc = سب کچھ ہر اسکین شدہ فولڈر اور ہر Workshop آئٹم دکھاتا ہے۔ کچھ چھپانے یا خارج کرنے کے لیے نیا پروفائل بنائیں۔
settings-profiles-delete-hint = حذف کرنے کے لیے دوبارہ کلک کریں۔ وال پیپر اور فولڈر اپنی جگہ رہتے ہیں۔
