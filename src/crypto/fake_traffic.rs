use anyhow::Result;
use rand::{Rng, thread_rng};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

// Sahte HTTP yöntemleri
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
        }
    }
    
    // Rastgele bir HTTP yöntemi döndür
    pub fn random() -> Self {
        let mut rng = thread_rng();
        match rng.gen_range(0..4) {
            0 => HttpMethod::GET,
            1 => HttpMethod::POST,
            2 => HttpMethod::PUT,
            _ => HttpMethod::DELETE,
        }
    }
}

// Sahte bir HTTP isteği
#[derive(Debug, Clone)]
pub struct FakeHttpRequest {
    pub id: String,
    pub method: String,
    pub url: String,
    pub data: Vec<u8>,
}

impl FakeHttpRequest {
    // Rastgele bir HTTP isteği oluştur
    pub fn random() -> Self {
        let mut rng = thread_rng();
        
        // Rastgele URL'ler
        let urls = [
            "https://example.com",
            "https://example.org/api/v1/users",
            "https://api.service.io/data",
            "https://cdn.content.net/assets",
            "https://search.services.org/query",
        ];
        
        // Rastgele veri boyutu (10-100 byte)
        let data_size = rng.gen_range(10..100);
        let mut data = Vec::with_capacity(data_size);
        for _ in 0..data_size {
            data.push(rng.gen::<u8>());
        }
        
        Self {
            id: Uuid::new_v4().to_string(),
            method: HttpMethod::random().as_str().to_string(),
            url: urls[rng.gen_range(0..urls.len())].to_string(),
            data,
        }
    }
}

// Sahte trafik üreteci
pub struct FakeTrafficGenerator {
    // Saniyede ortalama oluşturulacak sahte istek sayısı
    pub rate_per_second: f64,
    // Çalışıp çalışmadığı
    pub active: bool,
}

impl FakeTrafficGenerator {
    pub fn new(rate_per_second: f64) -> Self {
        Self {
            rate_per_second,
            active: false,
        }
    }
    
    // Sahte trafik üretmeye başla
    pub async fn start<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(FakeHttpRequest) + Send + 'static
    {
        self.active = true;
        
        let rate = self.rate_per_second;
        
        // Ayrı bir tokio görevinde sahte istekleri oluştur
        tokio::spawn(async move {
            loop {
                // Rastgele bir bekleme süresi (ortalama hıza göre)
                let wait_time = if rate > 0.0 {
                    // Üretilme hızı saniyede kaç istek
                    let mean_delay_secs = 1.0 / rate;
                    // Rastgele bir bekleme süresi hesapla
                    let delay_secs = {
                        let mut rng = thread_rng();
                        rng.gen_range(0.0..(mean_delay_secs * 2.0))
                    };
                    Duration::from_secs_f64(delay_secs)
                } else {
                    Duration::from_secs(10) // Eğer hız 0 ise, 10 saniye bekle
                };
                
                // Bekleme süresini uygula
                sleep(wait_time).await;
                
                // Sahte istek oluştur ve callback ile gönder
                let request = FakeHttpRequest::random();
                callback(request);
            }
        });
        
        Ok(())
    }
    
    // Sahte trafik üretmeyi durdur
    pub fn stop(&mut self) {
        self.active = false;
    }
} 