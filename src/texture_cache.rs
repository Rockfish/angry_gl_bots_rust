use small_gl_core::error::Error;
use small_gl_core::texture::{Texture, TextureConfig};
use std::collections::HashMap;
use std::ffi::OsString;
use std::rc::Rc;

pub struct TextureCache {
    texture_cache: HashMap<OsString, Rc<Texture>>,
}

impl TextureCache {
    pub fn new() -> Self {
        TextureCache {
            texture_cache: HashMap::new(),
        }
    }

    pub fn get_or_load_texture(&mut self, texture_path: impl Into<OsString>, texture_config: &TextureConfig) -> Result<Rc<Texture>, Error> {
        let os_string: OsString = texture_path.into();
        let cached = self.texture_cache.get(&os_string);
        match cached {
            None => {
                let texture = Rc::new(Texture::new(os_string.clone(), texture_config)?);
                // println!("-loaded texture: {:?}", &texture);
                self.texture_cache.insert(os_string, texture.clone());
                Ok(texture)
            }
            Some(texture) => Ok(texture.clone()),
        }
    }
}
