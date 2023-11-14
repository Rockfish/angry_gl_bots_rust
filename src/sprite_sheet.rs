use glam::Vec3;

pub struct SpriteSheet {
    pub texture_unit: i32,
    pub num_columns: i32,
    pub time_per_sprite: f32,
}

impl SpriteSheet {
    pub fn new(texture_unit: i32, num_columns: i32, time_per_sprite: f32) -> Self {
        SpriteSheet {
            texture_unit,
            num_columns,
            time_per_sprite,
        }
    }
}

pub struct SpriteSheetSprite {
    pub world_position: Vec3,
    pub age: f32,
}

impl SpriteSheetSprite {
    pub fn new(world_position: Vec3) -> Self {
        SpriteSheetSprite {
            world_position,
            age: 0.0,
        }
    }
}
