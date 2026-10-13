settings-library-watch-section-desc = skwd-wall dışında eklenen dosyaları algılayın ve kitaplığı güncel tutun.
settings-library-watch-fallback-label = Düzenli tarama yedeği
settings-library-watch-fallback-desc = Yalnızca yerel dosya izleyicisinin izleyemediği kitaplık klasörlerini kontrol edin. Değişiklikleri kaçıran ağ veya FUSE bağlamaları için etkinleştirin, ardından skwd-walld'yi yeniden başlatın.
settings-library-watch-interval-label = Tarama aralığı
settings-library-watch-interval-desc = Sınırlı kontroller arasında beklenecek saniye sayısı. Düşük değerler değişiklikleri daha erken bulur ancak dosya sistemini daha sık okur. Değiştirdikten sonra skwd-walld'yi yeniden başlatın.
settings-library-watch-unknown-label = İzleyici durumu alınamıyor
settings-library-watch-unknown-desc = Bu arka plan hizmeti kitaplık izleme durumunu bildirmiyor. skwd-walld'yi güncelleyin veya yeniden başlatın.
settings-library-watch-poll-failed-label = Tarama bir kitaplık klasörünü okuyamıyor
settings-library-watch-poll-failed-desc = Yapılandırılan tüm kitaplık klasörlerinin bağlı ve okunabilir olduğunu kontrol edin. Tarama { $interval } saniye sonra yeniden denenecek.
settings-library-watch-polling-label = Düzenli tarama yedeği etkin
settings-library-watch-polling-desc = Yerel izleme { $count ->
    [one] bir kitaplık klasörü
   *[other] { $count } kitaplık klasörü
    } için başarısız oldu. Her { $interval } saniyede en fazla { $budget } öğe kontrol edilir. Son başarılı eşitleme: { $convergence }.
settings-library-watch-recovering-label = Yerel izleme yeniden çalışıyor
settings-library-watch-recovering-desc = Yerel izleyici yeniden etkin. Kitaplığın güncel olduğu doğrulanmadan önce tam devir taraması sürüyor.
settings-library-watch-unavailable-label = Kitaplık izleme kullanılamıyor
settings-library-watch-unavailable-desc = Yerel dosya izleme başarısız oldu ve düzenli tarama yedeği kapalı. Düzenli tarama yedeğini etkinleştirin, ardından skwd-walld'yi yeniden başlatın.
settings-library-watch-recovered-label = Yerel izleme geri yüklendi
settings-library-watch-recovered-desc = Yerel izleyici ve devir taraması güncel. Son başarılı eşitleme: { $convergence }.
settings-library-watch-native-label = Yerel dosya izleme
settings-library-watch-native-desc = Tüm kitaplık klasörlerinde dosya sistemi olayları etkin. Düzenli tarama beklemede.
settings-library-watch-convergence-never = Henüz tamamlanmadı
settings-library-watch-convergence-seconds = { $value ->
    [one] 1 saniye önce
   *[other] { $value } saniye önce
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] 1 dakika önce
   *[other] { $value } dakika önce
    }
settings-library-watch-convergence-hours = { $value ->
    [one] 1 saat önce
   *[other] { $value } saat önce
    }
settings-profiles-card = Profiller
settings-profiles-card-desc = Seçicinin, klasör filtresinin, rastgele duvar kâğıtlarının ve zamanlamaların hangi klasörleri kullanacağını belirleyin. Gizli klasörler çubuktaki gizli anahtarı veya onun kısayolu ile geri gelir ve her açılış onları yeniden gizli başlatır.
settings-profiles-everything = Tümü
settings-profiles-name-label = Ad
settings-profiles-name-placeholder = Profil adı
settings-profiles-state-show = Göster
settings-profiles-state-hide = Gizle
settings-profiles-state-exclude = Hariç tut
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = Workshop öğelerinin klasörü yoktur, bu yüzden bu ayar hepsine uygulanır. Tek bir duvar kâğıdını kartının arkasından gizleyin.
settings-profiles-summary = { $hidden } gizli, { $excluded } hariç
settings-profiles-summary-none = Tüm klasörleri gösterir
settings-profiles-delete-action = Profili sil
settings-profiles-add-label = Yeni profil
settings-profiles-new-name = Yeni profil
settings-profiles-tree-label = Bu profildeki klasörler
settings-profiles-tree-desc = Göster klasörü seçicide ve rastgelede tutar. Gizle, çubuktaki gizli anahtarına veya kısayoluna basana kadar onu gözden uzak tutar. Hariç tut onu profilin dışında bırakır, bu yüzden anahtar onu atlar. Bir klasör alt klasörlerini de ayarlar; istisna için sonradan bir alt klasörü değiştirin.
settings-profiles-never-scanned = Hiç taranmaz
settings-profiles-mixed = İçi karışık
settings-profiles-hidden-inside = { $count ->
    [one] İçinde { $count } gizli
   *[other] İçinde { $count } gizli
    }
settings-profiles-copy-action = Kopyala
settings-profiles-helm-desc = Bunu etkin profil yapar; döndürme ve zamanlamalar onu izler.
settings-profiles-flag-desc = Bu profilde tek bir seçici açar ve etkin olanı olduğu gibi bırakır.
settings-profiles-make-active = Etkin yap
settings-profiles-active-tag = Etkin
settings-profiles-builtin-tag = Yerleşik
settings-profiles-filter-placeholder = Klasörleri filtrele
settings-profiles-no-match = Eşleşen klasör yok.
settings-profiles-switch-label = Bu profile geç
settings-profiles-switch-desc = skwd-helm kayıtlı profili değiştirir, böylece döngü ve zamanlamalar onu izler. Başlatma bayrağı bu profilde bir seçici açar ve kayıtlı olana dokunmaz.
settings-profiles-hidden-note-label = Gizli duvar kağıtları
settings-profiles-hidden-note-desc = Çubuktaki gizli düğmesi (Ctrl+H) etkin profilin gizli klasörlerini ve kartının arkasından gizlenen her duvar kağıdını gösterir. Her başlatma onları yeniden gizler.
settings-profiles-tree-note = Hiç taranmadı, Kitaplık › Klasörler içindeki hariç tutulan klasörlerden gelir, bu yüzden hiçbir profil onu gösteremez. Buradaki Hariç tut yalnızca bir klasörü bu profilin dışında bırakır. Kartından gizlenen duvar kağıtları, Her şey dahil her profilde Gizlileri göster'e kadar gizli kalır. .private gibi nokta klasörler burada sıradan klasörlerdir.
settings-profiles-legend-show = seçicide ve rastgelede.
settings-profiles-legend-hide = çubuktaki gizli düğmesine veya Ctrl+H'ye kadar. Asla rastgele.
settings-profiles-legend-exclude = bu profilden. Gizlileri göster onu atlar.
settings-profiles-everything-desc = Her şey, taranan her klasörü ve her Workshop öğesini gösterir. Bazılarını gizlemek veya hariç tutmak için yeni bir profil oluşturun.
settings-profiles-delete-hint = Silmek için yeniden tıklayın. Duvar kağıtları ve klasörler yerinde kalır.
