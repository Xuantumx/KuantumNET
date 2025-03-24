use anyhow::{anyhow, Result};
use rand::{rngs::OsRng, RngCore, Rng, seq::SliceRandom};
use ring::{aead, rand as ringrand};
use ring::rand::SecureRandom;
use std::vec::Vec;

pub mod fake_traffic;
pub mod anon_protocol;
pub mod chaotic_routing;
pub mod multi_layer;

// Şifreleme katmanlarını tanımla
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionLayer {
    ChaCha20Poly1305,
    AesGcm,
}

// Şifrelenmiş bir paket
#[derive(Debug, Clone)]
pub struct EncryptedPacket {
    // Paket içeriği
    pub data: Vec<u8>,
    // Kullanılan nonce'lar (katman sırasına göre)
    pub nonces: Vec<Vec<u8>>,
    // Hangi katmanların kullanıldığı (sırayla)
    pub layers: Vec<EncryptionLayer>,
    // Gidiş yolu (düğüm kimlikleri)
    pub route: Vec<String>,
}

// Rastgele bir yönlendirme yolu oluştur
pub fn generate_random_route(peer_ids: &[String], length: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();
    
    // En az bir düğüm varsa
    if !peer_ids.is_empty() {
        let mut route = Vec::with_capacity(length);
        
        for _ in 0..length {
            // Rastgele bir peer ID seç
            let peer = peer_ids.choose(&mut rng)
                .expect("Peer ID listesi boş olamaz")
                .clone();
            
            route.push(peer);
        }
        
        route
    } else {
        // Test için varsayılan yol (gerçek uygulamada kullanılmamalı)
        (0..length).map(|i| format!("test-peer-{}", i)).collect()
    }
}

// Çok katmanlı şifreleme
pub fn multi_layer_encrypt(data: &[u8], layers: &[EncryptionLayer]) -> Result<(Vec<u8>, Vec<Vec<u8>>)> {
    let mut current_data = data.to_vec();
    let mut nonces = Vec::with_capacity(layers.len());
    
    for &layer in layers {
        match layer {
            EncryptionLayer::ChaCha20Poly1305 => {
                let (encrypted, nonce) = encrypt_chacha20_poly1305(&current_data)?;
                current_data = encrypted;
                nonces.push(nonce);
            }
            EncryptionLayer::AesGcm => {
                // Gerçek uygulamada AES-GCM eklenebilir, şimdilik ChaCha20 kullan
                let (encrypted, nonce) = encrypt_chacha20_poly1305(&current_data)?;
                current_data = encrypted;
                nonces.push(nonce);
            }
        }
    }
    
    Ok((current_data, nonces))
}

// Bir katman şifresini çöz
pub fn decrypt_layer(data: &[u8], nonce: &[u8], layer: EncryptionLayer) -> Result<Vec<u8>> {
    match layer {
        EncryptionLayer::ChaCha20Poly1305 => {
            decrypt_chacha20_poly1305(data, nonce)
        }
        EncryptionLayer::AesGcm => {
            // Gerçek uygulamada AES-GCM çözme eklenir
            decrypt_chacha20_poly1305(data, nonce)
        }
    }
}

// ChaCha20-Poly1305 ile şifrele
fn encrypt_chacha20_poly1305(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    // Rastgele bir nonce oluştur
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
    
    // ChaCha20-Poly1305 anahtarı oluştur
    let rng = ringrand::SystemRandom::new();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes).expect("RNG hatası");
    let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &key_bytes)
        .expect("Anahtar oluşturma hatası");
    let key = aead::LessSafeKey::new(unbound_key);
    
    // Veriyi şifrele
    let mut in_out = data.to_vec();
    key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| anyhow!("Şifreleme hatası"))?;
    
    Ok((in_out, nonce_bytes.to_vec()))
}

// ChaCha20-Poly1305 ile şifresi çöz
fn decrypt_chacha20_poly1305(encrypted_data: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>> {
    if nonce_bytes.len() != 12 {
        return Err(anyhow!("Nonce 12 byte olmalıdır"));
    }
    
    let mut nonce_arr = [0u8; 12];
    nonce_arr.copy_from_slice(nonce_bytes);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_arr);
    
    // ChaCha20-Poly1305 anahtarı oluştur
    let rng = ringrand::SystemRandom::new();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes).expect("RNG hatası");
    let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &key_bytes)
        .expect("Anahtar oluşturma hatası");
    let key = aead::LessSafeKey::new(unbound_key);
    
    // Veriyi çöz
    let mut in_out = encrypted_data.to_vec();
    key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| anyhow!("Şifre çözme hatası"))?;
    
    // Tag boyutunu çıkar
    let tag_len = aead::CHACHA20_POLY1305.tag_len();
    in_out.truncate(in_out.len() - tag_len);
    
    Ok(in_out)
}

// Bir paketi birden fazla katmanda şifrele ve rota ekle
pub fn create_onion_packet(data: &[u8], peer_ids: &[String], layer_count: usize) -> Result<EncryptedPacket> {
    let mut rng = rand::thread_rng();
    
    // Kullanılacak şifreleme katmanları
    let layers: Vec<EncryptionLayer> = (0..layer_count)
        .map(|_| {
            if rng.gen_bool(0.5) {
                EncryptionLayer::ChaCha20Poly1305
            } else {
                EncryptionLayer::AesGcm
            }
        })
        .collect();
    
    // Kaotik bir rota oluştur
    let route_length = 3.max(rng.gen_range(3..7)); // En az 3, en fazla 6 düğüm
    let route = generate_random_route(peer_ids, route_length);
    
    // Veriyi şifrele
    let (encrypted_data, nonces) = multi_layer_encrypt(data, &layers)?;
    
    Ok(EncryptedPacket {
        data: encrypted_data,
        nonces,
        layers,
        route,
    })
} 