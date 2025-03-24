use anyhow::{anyhow, Result};
use ring::{aead, rand::SecureRandom};
use ring::rand as ringrand;
use std::fmt;

// Şifreleme katmanı
pub struct EncryptionLayer {
    key: [u8; 32],
    nonce: [u8; 12],
}

impl fmt::Debug for EncryptionLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptionLayer")
            .field("key_len", &self.key.len())
            .field("nonce_len", &self.nonce.len())
            .finish()
    }
}

impl EncryptionLayer {
    // Yeni bir şifreleme katmanı oluştur
    pub fn new() -> Self {
        let rng = ringrand::SystemRandom::new();
        
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        
        // Rastgele anahtar ve nonce oluştur
        rng.fill(&mut key).expect("Anahtar oluşturma hatası");
        rng.fill(&mut nonce).expect("Nonce oluşturma hatası");
        
        Self { key, nonce }
    }
    
    // Veriyi şifrele
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &self.key)
            .map_err(|_| anyhow!("Anahtar oluşturma hatası"))?;
        let key = aead::LessSafeKey::new(unbound_key);
        
        let nonce = aead::Nonce::assume_unique_for_key(self.nonce);
        
        // Şifreleme için giriş/çıkış verisi
        let mut in_out = data.to_vec();
        
        // Veriyi şifrele
        key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| anyhow!("Şifreleme hatası"))?;
        
        // Şifrelenmiş verilere nonce'u ekle
        let mut result = Vec::with_capacity(in_out.len() + self.nonce.len());
        result.extend_from_slice(&self.nonce);
        result.extend_from_slice(&in_out);
        
        Ok(result)
    }
    
    // Veriyi çöz
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < self.nonce.len() {
            return Err(anyhow!("Geçersiz şifrelenmiş veri"));
        }
        
        // Nonce ve şifrelenmiş veriyi ayır
        let ciphertext = &data[self.nonce.len()..];
        
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &self.key)
            .map_err(|_| anyhow!("Anahtar oluşturma hatası"))?;
        let key = aead::LessSafeKey::new(unbound_key);
        
        let nonce = aead::Nonce::assume_unique_for_key(self.nonce);
        
        // Şifrelenmiş veriyi çöz
        let mut in_out = ciphertext.to_vec();
        key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| anyhow!("Şifre çözme hatası"))?;
        
        // Tag boyutunu çıkar
        let tag_len = aead::CHACHA20_POLY1305.tag_len();
        in_out.truncate(in_out.len() - tag_len);
        
        Ok(in_out)
    }
}

// Çok katmanlı şifreleme sistemi
// Verilerin birden fazla katman ile şifrelenmesini sağlar
#[derive(Debug)]
pub struct MultiLayerEncryption {
    layer_count: usize,
    layers: Vec<EncryptionLayer>,
}

impl MultiLayerEncryption {
    // Yeni bir çok katmanlı şifreleme oluştur
    pub fn new(layer_count: usize) -> Self {
        let mut layers = Vec::with_capacity(layer_count);
        
        // Belirtilen sayıda şifreleme katmanı oluştur
        for _ in 0..layer_count {
            layers.push(EncryptionLayer::new());
        }
        
        Self {
            layer_count,
            layers,
        }
    }
    
    // Veriyi çok katmanlı şifrele
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut current_data = data.to_vec();
        
        // Her katman için şifreleme yap
        for layer in &self.layers {
            current_data = layer.encrypt(&current_data)?;
        }
        
        Ok(current_data)
    }
    
    // Çok katmanlı şifrelenmiş veriyi çöz
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut current_data = data.to_vec();
        
        // Her katmanı ters sırayla çöz
        for layer in self.layers.iter().rev() {
            current_data = layer.decrypt(&current_data)?;
        }
        
        Ok(current_data)
    }
    
    // Yeni bir şifreleme katmanı ekle
    pub fn add_layer(&mut self) {
        self.layers.push(EncryptionLayer::new());
        self.layer_count += 1;
    }
    
    // Katman sayısını döndür
    pub fn layer_count(&self) -> usize {
        self.layer_count
    }
} 