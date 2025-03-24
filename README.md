# Kuantum Network

Kuantum Network, Tor ağına benzer şekilde çalışan, anonim ve güvenli bir P2P (Eşten-Eşe) iletişim altyapısı sağlayan bir projedir. Modern kriptografik teknikler ve dağıtık ağ özellikleri kullanarak, kullanıcıların gizliliklerini korurken iletişim kurmalarını sağlar.

## Özellikler

- **Anonim İletişim**: Geçici kimlikler ve dijital imzalar kullanarak gerçek kimliğinizi gizleyin
- **Çok Katmanlı Şifreleme**: Soğan yönlendirme (onion routing) prensibiyle çoklu şifreleme katmanları
- **Kaotik Yönlendirme**: Mesajların öngörülemeyen yollarla iletilmesi
- **Sahte Trafik Üretimi**: Gerçek trafiği gizlemek için arka planda otomatik sahte istek oluşturma
- **P2P Ağ Yapısı**: libp2p kütüphanesi ile eşler arası dağıtık ağ iletişimi
- **mDNS Keşfi**: Yerel ağda otomatik düğüm keşfi
- **Floodsub Mesajlaşma**: Abonelik tabanlı yayın mesajlaşma protokolü

## Gereksinimler

- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 veya üzeri)
- Bir Linux, macOS veya Windows işletim sistemi
- İnternet bağlantısı

## Kurulum

1. Rust programlama dilini ve Cargo paket yöneticisini yükleyin:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Projeyi klonlayın:
   ```bash
   git clone https://github.com/kullanıcıadı/kuantum-network.git
   cd kuantum-network
   ```

3. Bağımlılıkları yükleyin ve projeyi derleyin:
   ```bash
   cargo build --release
   ```

## Kullanım

1. Programı çalıştırmak için:
   ```bash
   cargo run --release
   ```

2. Program başladığında, yerel bir Peer ID ve dinleme adresleri görüntülenecektir:
   ```
   Yerel peer ID: 12D3KooWFppE8THBovUSWbAhjXkNoWQBLrx6SamGacmRRBHn3SKG
   Dinleme adresi: /ip4/127.0.0.1/tcp/40331
   Dinleme adresi: /ip4/192.168.1.193/tcp/40331
   ```

3. Komut satırında metin yazarak ağdaki diğer düğümlere mesaj gönderebilirsiniz.

4. Programdan çıkmak için `exit` yazın.

## Nasıl Çalışır?

Kuantum Network, aşağıdaki temel prensipler üzerine inşa edilmiştir:

1. **Geçici Kimlikler**: Her kullanıcı belirli aralıklarla değişen geçici kimlikler kullanır. Bu kimlikler gerçek kimlikle bağlantılı değildir ve kısa ömürlüdür.

2. **Soğan Yönlendirme**: Mesajlar birden fazla şifreleme katmanıyla sarılır. Her düğüm sadece kendi katmanını çözebilir, böylece mesajın tamamını hiçbir düğüm göremez.

3. **Kaotik Yönlendirme**: Mesajlar rastgele yollar izleyerek hedeflerine ulaşır. Bu, trafik analizi saldırılarını zorlaştırır.

4. **Sahte Trafik**: Arka planda üretilen sahte trafik, gerçek mesajları gizlemeye yardımcı olur ve ağdaki iletişim modellerini bulanıklaştırır.

## Teknik Mimari

Kuantum Network aşağıdaki bileşenlerden oluşur:

- **AnonymousProtocol**: Anonim mesajlaşma için protokol tanımlamaları
- **MultiLayerEncryption**: Çok katmanlı şifreleme altyapısı (ChaCha20-Poly1305 algoritması)
- **ChaoticRouter**: Kaotik yönlendirme algoritması
- **FakeTrafficGenerator**: Sahte HTTP istekleri oluşturan arka plan servisi
- **KuantumBehaviour**: libp2p ağ davranışlarını yöneten ana modül

## Gelecek Planları

- Web arayüzü entegrasyonu
- Tam Tor benzeri devre oluşturma mekanizması
- Giriş, röle ve çıkış düğümü rolleri
- Dizin hizmetleri ve otomatik keşif
- Performans ve güvenlik iyileştirmeleri
- Daha fazla platform desteği

## Katkıda Bulunma

Projeye katkıda bulunmak isterseniz:

1. Bir issue oluşturun veya mevcut bir issue üzerinde çalışın
2. Projeyi forklayın
3. Yeni bir branch oluşturun (`git checkout -b feature/amazing-feature`)
4. Değişikliklerinizi commit edin (`git commit -m 'Add some amazing feature'`)
5. Branch'inizi push edin (`git push origin feature/amazing-feature`)
6. Bir Pull Request açın


Bu yazılım açık kaynak olarak sunulmuş olup, herhangi bir garanti verilmemektedir. Kullanım ve dağıtım tamamen kullanıcı sorumluluğundadır.

## İletişim

Sorularınız veya önerileriniz için [GitHub Issues](https://github.com/kullanıcıadı/kuantum-network/issues) kullanabilirsiniz. # KuantumNET Projesi
