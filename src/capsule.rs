pub struct Capsule {
    pub(crate) height: f32,
    pub(crate) radius: f32,
}

impl Capsule {
    pub fn new(height: f32, radius: f32) -> Self {
        Capsule { height, radius }
    }
}
