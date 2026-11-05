settings-library-folders-desc = Bu klasörleri ve alt klasörlerini görsel ve video için tara.
settings-library-folders-say = Taranıyor: { $val }{ $folders }{ $end }.{ $state ->
        [polling] { " " }Yoklama, yerel izlemenin kaçırdığını her { $every } yakalar.{ $unread }
        [native] { " " }Yerel dosya izleme etkin; yoklama boşta.
        [down] { " " }Yerel izleme başarısız oldu ve yoklama kapalı.
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }Bir klasör okunamıyor.
       *[other] { " " }{ $count } klasör okunamıyor.
    }
settings-library-watching = İzleme
settings-library-poll-hint = Ağ veya FUSE klasörleri için
settings-library-interval-hint = Değiştirince skwd-walld'u yeniden başlatın
settings-library-excluded-desc = Bu klasörleri ve alt klasörlerini atla. Mevcut meta veriler saklanır.
settings-library-excluded-say = Asla taranmıyor: { $val }{ $folders }{ $end }.{ $outside }
settings-library-outside = { $count ->
        [one] { " " }Biri tüm duvar kâğıdı klasörlerinin dışında ve hiçbir şeyi atlamıyor.
       *[other] { " " }{ $count } tanesi tüm duvar kâğıdı klasörlerinin dışında ve hiçbir şeyi atlamıyor.
    }
settings-library-excluded-hint = Hariç klasörler asla taranmaz, bu yüzden hiçbir profil gösteremez. Bir klasörü tutup herkesten gizlemek için { $link } kullanın.
settings-library-profiles-desc = Seçicinin, klasör filtresinin, rastgele duvar kâğıtlarının ve zamanlamaların hangi klasörleri kullanacağını belirleyin.
settings-library-profiles-say = Etkin profil { $state ->
        [none] { $menu }, her klasörü gösterir.
       *[some] { $menu }: { $summary }.
    } Ctrl+H veya çubuktaki gizli düğmesi, kartından gizlenenler dahil gizli duvar kâğıtlarını sonraki açılışa kadar gösterir.
settings-library-active = Etkin profil
settings-library-terminal = Terminalden değiştir
settings-library-tree-note = Gizle, Ctrl+H'ye kadar rastgelenin dışında kalır. Hariç tut, Ctrl+H ile bile dışarıda kalır. Klasör alt klasörlerini de ayarlar.
settings-library-show-tip = Seçicide ve rastgelede
settings-library-hide-tip = Ctrl+H'ye kadar, asla rastgele değil
settings-library-exclude-tip = Bu profilin dışında, Ctrl+H atlar
settings-library-from = Kaynak: { $link }
settings-library-excluded-inside = İçinde { $count } hariç
settings-library-both-inside = İçinde { $hidden } gizli, { $excluded } hariç
settings-library-storage = Depolama ve çöp
settings-library-storage-desc = Küçük WebP kopyaları, kurtarma deposu ve yeniden renklendirilmiş kopyalar.
settings-library-storage-say = { $optimise ->
        [on] Yeni görseller { $quality } kalitede WebP olur, üst sınır { $size }.
       *[off] Yeni görseller olduğu gibi kalır. Tümünü iyileştir { $quality } kalite kullanır, üst sınır { $size }.
    } { $trash ->
        [on] Çöpteki özgün dosyalar { $days } kalır, sonra silinir.
       *[off] Çöpteki özgün dosyalar siz silene kadar kalır.
    } { $recolour ->
        [on] Yeni duvar kâğıtları yeniden renklendirilmiş bir kopya da alır.
       *[off] Yeni duvar kâğıtları yeniden renklendirilmez.
    }
settings-library-retention-hint = Çöpteki görsellerin saklanacağı gün
settings-library-optimise-hint = Kütüphanedeki tüm görselleri iyileştir
settings-library-recolour = Yeniden renklendirme
settings-library-thumbnails = Küçük resimler ve onarım
settings-library-thumbnails-desc = Bir şey yanlış görünürse önizlemeleri ve renk verilerini yeniden oluşturun.
settings-library-thumbnails-say = Yeniden oluştururken aynı anda en çok { $jobs } küçük resim yapılır.
settings-library-rebuild = Yeniden oluştur
settings-library-start-over = Baştan başla
settings-library-recompute-hint = Mevcut küçük resimlerden
settings-library-clear-hint = Küçük resimler sonraki taramada yenilenir
