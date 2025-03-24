use anyhow::Result;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use libp2p::PeerId;

// Kaotik yönlendirme sistemi
// Mesajların rastgele yönlendirilmesi için kullanılır
pub struct ChaoticRouter {
    forward_probability: f32,  // Mesajı yönlendirme olasılığı
    max_hops: u32,            // Maksimum atlama sayısı
    current_routes: HashMap<String, Vec<PeerId>>, // Mevcut rotalar
}

impl ChaoticRouter {
    // Yeni bir kaotik yönlendirici oluştur
    pub fn new(forward_probability: f32, max_hops: u32) -> Self {
        Self {
            forward_probability,
            max_hops,
            current_routes: HashMap::new(),
        }
    }
    
    // Mesajın yönlendirilip yönlendirilmeyeceğine karar ver
    pub fn should_forward(&self) -> bool {
        let mut rng = thread_rng();
        rng.gen::<f32>() < self.forward_probability
    }
    
    // Rastgele bir rota oluştur
    pub fn generate_random_route(&self, available_peers: &[PeerId], hop_count: u32) -> Vec<PeerId> {
        if available_peers.is_empty() || hop_count == 0 {
            return Vec::new();
        }
        
        let mut rng = thread_rng();
        let actual_hops = std::cmp::min(hop_count, self.max_hops);
        let mut route = Vec::with_capacity(actual_hops as usize);
        
        for _ in 0..actual_hops {
            if let Some(peer) = available_peers.get(rng.gen_range(0..available_peers.len())) {
                route.push(*peer);
            }
        }
        
        route
    }
    
    // Mesaj için yeni bir rota oluştur ve kaydet
    pub fn create_route(&mut self, message_id: &str, available_peers: &[PeerId]) -> Result<Vec<PeerId>> {
        let hop_count = thread_rng().gen_range(1..=self.max_hops);
        let route = self.generate_random_route(available_peers, hop_count);
        
        self.current_routes.insert(message_id.to_string(), route.clone());
        
        Ok(route)
    }
    
    // Belirli bir mesaj ID'si için rotayı al
    pub fn get_route(&self, message_id: &str) -> Option<&Vec<PeerId>> {
        self.current_routes.get(message_id)
    }
    
    // Rota tamamlandığında temizle
    pub fn clear_route(&mut self, message_id: &str) {
        self.current_routes.remove(message_id);
    }
} 