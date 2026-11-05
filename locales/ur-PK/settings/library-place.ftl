settings-library-folders-desc = تصاویر اور ویڈیوز کے لیے ان فولڈرز اور ذیلی فولڈرز کو اسکین کریں۔
settings-library-folders-say = { $val }{ $folders }{ $end } اسکین ہو رہے ہیں۔{ $state ->
        [polling] { " " }پولنگ ہر { $every } وہ پکڑتی ہے جو مقامی نگرانی سے چھوٹ جائے۔{ $unread }
        [native] { " " }مقامی فائل نگرانی فعال ہے؛ پولنگ رکی ہوئی ہے۔
        [down] { " " }مقامی نگرانی ناکام ہوئی اور پولنگ بند ہے۔
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }ایک فولڈر پڑھا نہیں جا سکتا۔
       *[other] { " " }{ $count } فولڈر پڑھے نہیں جا سکتے۔
    }
settings-library-watching = نگرانی
settings-library-poll-hint = نیٹ ورک یا FUSE فولڈرز کے لیے
settings-library-interval-hint = بدلنے کے بعد skwd-walld دوبارہ چلائیں
settings-library-excluded-desc = ان فولڈرز اور ذیلی فولڈرز کو چھوڑیں۔ موجودہ میٹاڈیٹا محفوظ رہتا ہے۔
settings-library-excluded-say = { $val }{ $folders }{ $end } کبھی اسکین نہیں ہوتے۔{ $outside }
settings-library-outside = { $count ->
        [one] { " " }ایک کسی وال پیپر فولڈر کے اندر نہیں اور کچھ نہیں چھوڑتا۔
       *[other] { " " }{ $count } کسی وال پیپر فولڈر کے اندر نہیں اور کچھ نہیں چھوڑتے۔
    }
settings-library-excluded-hint = خارج فولڈر کبھی اسکین نہیں ہوتے، اس لیے کوئی پروفائل انہیں نہیں دکھا سکتی۔ فولڈر رکھ کر عوام میں چھپانے کے لیے { $link } استعمال کریں۔
settings-library-profiles-desc = طے کریں کہ پکر، فولڈر فلٹر، بے ترتیب وال پیپر اور شیڈول کون سے فولڈر لیں۔
settings-library-profiles-say = فعال پروفائل { $state ->
        [none] { $menu }، جو ہر فولڈر دکھاتی ہے۔
       *[some] { $menu }:‏ { $summary }۔
    } Ctrl+H یا بار کا چھپا بٹن چھپے وال پیپر دکھاتا ہے، کارڈ سے چھپائے گئے بھی، اگلی بار چلانے تک۔
settings-library-active = فعال پروفائل
settings-library-terminal = ٹرمینل سے بدلیں
settings-library-tree-note = چھپائیں: Ctrl+H تک بے ترتیب سے باہر۔ خارج: Ctrl+H پر بھی باہر۔ فولڈر اپنے ذیلی فولڈر بھی طے کرتا ہے۔
settings-library-show-tip = پکر اور بے ترتیب میں
settings-library-hide-tip = Ctrl+H تک، کبھی بے ترتیب نہیں
settings-library-exclude-tip = اس پروفائل سے باہر، Ctrl+H چھوڑ دیتا ہے
settings-library-from = { $link } سے
settings-library-excluded-inside = اندر { $count } خارج
settings-library-both-inside = اندر { $hidden } چھپے، { $excluded } خارج
settings-library-storage = اسٹوریج اور ردی
settings-library-storage-desc = چھوٹی WebP نقول، بحالی اسٹوریج اور نئے رنگ کی نقول۔
settings-library-storage-say = { $optimise ->
        [on] نئی تصاویر { $quality } معیار پر WebP بنتی ہیں، حد { $size }۔
       *[off] نئی تصاویر جیسی ہیں ویسی رہتی ہیں۔ سب بہتر کریں { $quality } معیار لیتا ہے، حد { $size }۔
    } { $trash ->
        [on] ردی کے اصل { $days } رہتے ہیں، پھر حذف ہوتے ہیں۔
       *[off] ردی کے اصل آپ کے حذف کرنے تک رہتے ہیں۔
    } { $recolour ->
        [on] نئے وال پیپر کو نئے رنگ کی نقل بھی ملتی ہے۔
       *[off] نئے وال پیپر کا رنگ نہیں بدلتا۔
    }
settings-library-retention-hint = ردی کی تصاویر رکھنے کے دن
settings-library-optimise-hint = لائبریری کی سب تصاویر بہتر کریں
settings-library-recolour = نیا رنگ
settings-library-thumbnails = تھمب نیل اور مرمت
settings-library-thumbnails-desc = کچھ غلط لگے تو پیش منظر اور رنگ کا ڈیٹا دوبارہ بنائیں۔
settings-library-thumbnails-say = دوبارہ بناتے وقت ایک ساتھ { $jobs } تک تھمب نیل بنتے ہیں۔
settings-library-rebuild = دوبارہ بنائیں
settings-library-start-over = نئے سرے سے
settings-library-recompute-hint = موجودہ تھمب نیل سے
settings-library-clear-hint = اگلے اسکین پر تھمب نیل دوبارہ بنیں گے
