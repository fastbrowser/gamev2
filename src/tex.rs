use macroquad::texture::Texture2D;

#[derive(Debug)]
pub struct Textures {
    pub block: Option<Texture2D>,
    pub spike: Option<Texture2D>,
    pub player: Option<Texture2D>,
}
impl Clone for Textures {
    fn clone(&self) -> Self {
        Textures {
            player: self.player.as_ref().map(|t| t.weak_clone()),
            block: self.block.as_ref().map(|t| t.weak_clone()),
            spike: self.spike.as_ref().map(|t| t.weak_clone()),
        }
    }
}