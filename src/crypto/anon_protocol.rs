use anyhow::{anyhow, Result};
use std::fmt;
use prost::Message as ProstMessage;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ring::{aead, rand as ringrand, signature::Ed25519KeyPair};
use ring::rand::SecureRandom;

// Kimliksiz mesaj türleri
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    Text = 0,
    Binary = 1,
    Command = 2,
    Handshake = 3,
}

impl MessageType {
    // i32'den MessageType'a dönüştür (Protobuf için)
    pub fn from_i32(value: i32) -> Option<MessageType> {
        match value {
            0 => Some(MessageType::Text),
            1 => Some(MessageType::Binary),
            2 => Some(MessageType::Command),
            3 => Some(MessageType::Handshake),
            _ => None,
        }
    }
    
    // Varsayılan tür
    pub fn default() -> Self {
        MessageType::Text
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Text => write!(f, "Text"),
            MessageType::Binary => write!(f, "Binary"),
            MessageType::Command => write!(f, "Command"),
            MessageType::Handshake => write!(f, "Handshake"),
        }
    }
}

// Protobuf ile tanımlanmış bir mesaj formatı
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AnonMessage {
    pub msg_type: i32,
    pub timestamp: u64,
    pub temp_id: String,
    pub payload: Vec<u8>,
    pub signature: Vec<u8>,
    pub hop_count: u32,
}

impl ProstMessage for AnonMessage {
    fn encode_raw<B>(&self, buf: &mut B) where B: prost::bytes::BufMut, Self: Sized {
        prost::encoding::message::encode(1, &self.msg_type, buf);
        prost::encoding::message::encode(2, &self.timestamp, buf);
        prost::encoding::message::encode(3, &self.temp_id, buf);
        prost::encoding::message::encode(4, &self.payload, buf);
        prost::encoding::message::encode(5, &self.signature, buf);
        prost::encoding::message::encode(6, &self.hop_count, buf);
    }
    
    fn merge_field<B>(&mut self, tag: u32, wire_type: prost::encoding::WireType, buf: &mut B, ctx: prost::encoding::DecodeContext) -> Result<(), prost::DecodeError>
    where B: prost::bytes::Buf, Self: Sized {
        match tag {
            1 => prost::encoding::message::merge(wire_type, &mut self.msg_type, buf, ctx),
            2 => prost::encoding::message::merge(wire_type, &mut self.timestamp, buf, ctx),
            3 => prost::encoding::message::merge(wire_type, &mut self.temp_id, buf, ctx),
            4 => prost::encoding::message::merge(wire_type, &mut self.payload, buf, ctx),
            5 => prost::encoding::message::merge(wire_type, &mut self.signature, buf, ctx),
            6 => prost::encoding::message::merge(wire_type, &mut self.hop_count, buf, ctx),
            _ => prost::encoding::skip_field(wire_type, tag, buf, ctx),
        }
    }
    
    fn encoded_len(&self) -> usize {
        prost::encoding::message::encoded_len(1, &self.msg_type) +
        prost::encoding::message::encoded_len(2, &self.timestamp) +
        prost::encoding::message::encoded_len(3, &self.temp_id) +
        prost::encoding::message::encoded_len(4, &self.payload) +
        prost::encoding::message::encoded_len(5, &self.signature) +
        prost::encoding::message::encoded_len(6, &self.hop_count)
    }
    
    fn clear(&mut self) {
        self.msg_type = 0;
        self.timestamp = 0;
        self.temp_id = String::new();
        self.payload.clear();
        self.signature.clear();
        self.hop_count = 0;
    }
}

impl AnonMessage {
    pub fn new(msg_type: MessageType, temp_id: &str, payload: Vec<u8>, signature: Vec<u8>, hop_count: u32) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            msg_type: msg_type as i32,
            timestamp,
            temp_id: temp_id.to_string(),
            payload,
            signature,
            hop_count,
        }
    }
    
    pub fn get_message_type(&self) -> Option<MessageType> {
        MessageType::from_i32(self.msg_type)
    }
}

// Geçici kimlik
pub struct TemporaryIdentity {
    pub id: String,
    pub keypair: Ed25519KeyPair,
    pub created_at: SystemTime,
    pub valid_until: SystemTime,
}

impl TemporaryIdentity {
    // Yeni bir geçici kimlik oluştur
    pub fn new(valid_duration: Duration) -> Result<Self> {
        // Rastgele veri oluştur
        let rng = ringrand::SystemRandom::new();
        
        // Ed25519 anahtar çifti oluştur
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;
        let keypair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
        
        // Geçici ID oluştur
        let mut id_bytes = [0u8; 16];
        rng.fill(&mut id_bytes)?;
        
        let id = hex::encode(&id_bytes);
        let now = SystemTime::now();
        let valid_until = now + valid_duration;
        
        Ok(Self {
            id,
            keypair,
            created_at: now,
            valid_until,
        })
    }
    
    // Geçerli mi kontrol et
    pub fn is_valid(&self) -> bool {
        SystemTime::now() <= self.valid_until
    }
    
    // Mesajı imzala
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.keypair.sign(message).as_ref().to_vec()
    }
}

// Anonim mesaj oluşturucu
pub struct AnonymousProtocol {
    current_identity: Option<TemporaryIdentity>,
    identity_duration: Duration,
}

impl AnonymousProtocol {
    pub fn new(identity_duration: Duration) -> Self {
        Self {
            current_identity: None,
            identity_duration,
        }
    }
    
    // Geçerli bir kimlik al veya yeni oluştur
    pub fn get_identity(&mut self) -> Result<&TemporaryIdentity> {
        // Önce geçerli kimliğin durumunu kontrol et
        let should_create_new = match &self.current_identity {
            Some(identity) if identity.is_valid() => false,
            _ => true,
        };
        
        // Geçerli değilse yeni oluştur
        if should_create_new {
            self.current_identity = Some(TemporaryIdentity::new(self.identity_duration)?);
        }
        
        // Şimdi güvenle döndür
        match &self.current_identity {
            Some(identity) => Ok(identity),
            None => Err(anyhow!("Kimlik oluşturulamadı")),
        }
    }
    
    // Yeni bir anonim mesaj oluştur
    pub fn create_message(&mut self, msg_type: MessageType, payload: &[u8], hop_count: u32) -> Result<AnonMessage> {
        // Geçerli bir kimlik al
        let identity = self.get_identity()?;
        
        // Timestamp oluştur
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Zaman hesaplama hatası: {}", e))?
            .as_secs();
        
        // Mesaj içeriğini hazırla
        let mut message = AnonMessage {
            msg_type: msg_type as i32,
            timestamp,
            temp_id: identity.id.clone(),
            payload: payload.to_vec(),
            signature: Vec::new(), // İmza ilk başta boş
            hop_count,
        };
        
        // Mesaj verilerinden bir hash oluştur
        let mut message_data = Vec::new();
        message_data.extend_from_slice(&(message.msg_type as u32).to_be_bytes());
        message_data.extend_from_slice(&message.timestamp.to_be_bytes());
        message_data.extend_from_slice(message.temp_id.as_bytes());
        message_data.extend_from_slice(&message.payload);
        message_data.extend_from_slice(&message.hop_count.to_be_bytes());
        
        // İmzala
        let signature = identity.sign(&message_data);
        message.signature = signature;
        
        Ok(message)
    }
    
    // Mesajı şifreli bir paket içine koy (ChaCha20-Poly1305 ile)
    pub fn encrypt_message(&self, message: &AnonMessage) -> Result<Vec<u8>> {
        // Önce mesajı binary formata dönüştür
        let mut encoded = Vec::new();
        message.encode(&mut encoded);
        
        // Şifreleme için anahtar ve nonce oluştur
        let rng = ringrand::SystemRandom::new();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes)?;
        
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)?;
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
        
        // ChaCha20-Poly1305 ile şifrele
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &key_bytes)
            .map_err(|_| anyhow!("Anahtar oluşturma hatası"))?;
        let key = aead::LessSafeKey::new(unbound_key);
        
        // Veriyi şifrele
        let mut in_out = encoded;
        key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| anyhow!("Şifreleme hatası"))?;
        
        // Şifrelenmiş verilere nonce'u ekle
        let mut result = Vec::with_capacity(in_out.len() + nonce_bytes.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&in_out);
        
        Ok(result)
    }
    
    // Şifreli paketi çöz
    pub fn decrypt_message(&self, encrypted: &[u8]) -> Result<AnonMessage> {
        if encrypted.len() < 12 {
            return Err(anyhow!("Geçersiz şifrelenmiş mesaj"));
        }
        
        // Nonce'u ve şifrelenmiş veriyi ayır
        let nonce_bytes = &encrypted[..12];
        let ciphertext = &encrypted[12..];
        
        // Şifre çözme için anahtar oluştur (gerçekte bu doğru değil, 
        // alıcının anahtarı bilmesi gerekir, ama bu örnek için basitleştirilmiştir)
        let rng = ringrand::SystemRandom::new();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes)?;
        
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(nonce_bytes);
        let nonce = aead::Nonce::assume_unique_for_key(nonce_arr);
        
        // ChaCha20-Poly1305 anahtarı oluştur
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &key_bytes)
            .map_err(|_| anyhow!("Anahtar oluşturma hatası"))?;
        let key = aead::LessSafeKey::new(unbound_key);
        
        // Veriyi çöz
        let mut in_out = ciphertext.to_vec();
        key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| anyhow!("Şifre çözme hatası"))?;
        
        // Tag boyutunu çıkar
        let tag_len = aead::CHACHA20_POLY1305.tag_len();
        in_out.truncate(in_out.len() - tag_len);
        
        // Çözülmüş veriyi AnonMessage'a dönüştür
        let message = AnonMessage::decode(&*in_out)
            .map_err(|e| anyhow!("Mesaj çözme hatası: {}", e))?;
        
        Ok(message)
    }
} 