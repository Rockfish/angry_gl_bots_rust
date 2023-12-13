#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::assign_op_pattern)]

mod capsule;
mod enemy;
mod geom;
mod sprite_sheet;
mod bullet_store;
mod floor;
mod texture_cache;
mod quads;
mod depth_map;
mod wiggly_bois;
mod enemies;
mod player;

extern crate glfw;

use crate::enemy::{chase_player, Enemy, ENEMY_COLLIDER, EnemySpawner};
use crate::geom::distanceBetweenPointAndLineSegment;
use glam::{vec2, vec3, Mat4, Vec3, Vec2};
use glfw::{Action, Context, Key};
use log::error;
use small_gl_core::camera::{Camera, CameraMovement};
use small_gl_core::gl;
use small_gl_core::model::ModelBuilder;
use small_gl_core::shader::Shader;
use small_gl_core::texture::{TextureConfig, TextureType, TextureWrap};
use std::rc::Rc;
use std::time::Duration;
use small_gl_core::animator::{AnimationClip, AnimationRepeat};
use crate::bullet_store::BulletStore;
use crate::floor::Floor;
use crate::player::Player;
use crate::sprite_sheet::SpriteSheet;
use crate::texture_cache::TextureCache;
use crate::wiggly_bois::draw_wiggly_bois;

const SCR_WIDTH: f32 = 1000.0;
const SCR_HEIGHT: f32 = 800.0;

const parallelism: i32 = 4;

// Viewport
const viewportWidth: i32 = 1500;
const viewportHeight: i32 = 1000;

// Texture units
const texUnit_playerDiffuse: i32 = 0;
const texUnit_gunDiffuse: i32 = 1;
const texUnit_floorDiffuse: i32 = 2;
const texUnit_wigglyBoi: i32 = 3;
const texUnit_bullet: i32 = 4;
const texUnit_floorNormal: i32 = 5;
const texUnit_playerNormal: i32 = 6;
const texUnit_gunNormal: i32 = 7;
const texUnit_shadowMap: i32 = 8;
const texUnit_emissionFBO: i32 = 9;
const texUnit_playerEmission: i32 = 10;
const texUnit_gunEmission: i32 = 11;
const texUnit_scene: i32 = 12;
const texUnit_horzBlur: i32 = 13;
const texUnit_vertBlur: i32 = 14;
const texUnit_impactSpriteSheet: i32 = 15;
const texUnit_muzzleFlashSpriteSheet: i32 = 16;
const texUnit_floorSpec: i32 = 18;
const texUnit_playerSpec: i32 = 19;
const texUnit_gunSpec: i32 = 20;

// Player
const fireInterval: f32 = 0.1; // seconds
const spreadAmount: i32 = 20;
const playerSpeed: f32 = 1.5;
const playerCollisionRadius: f32 = 0.35;

// Models
const playerModelScale: f32 = 0.0044;
const playerModelGunHeight: f32 = 120.0; // un-scaled
const playerModelGunMuzzleOffset: f32 = 100.0; // un-scaled
const monsterY: f32 = playerModelScale * playerModelGunHeight;

// Lighting
const lightFactor: f32 = 0.8;
const nonBlue: f32 = 0.9;

const floorLightFactor: f32 = 0.35;
const floorNonBlue: f32 = 0.7;

// Enemies
const monsterSpeed: f32 = 0.6;




struct State {
    camera: Camera,
    delta_time: f32,
    last_frame: f32,
    first_mouse: bool,
    last_x: f32,
    last_y: f32,
    player: Player,
    enemies: Vec<Enemy>,
}

fn error_callback(err: glfw::Error, description: String) {
    error!("GLFW error {:?}: {:?}", err, description);
}

fn main() {
    let mut glfw = glfw::init(error_callback).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(
            SCR_WIDTH as u32,
            SCR_HEIGHT as u32,
            "LearnOpenGL",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_all_polling(true);
    window.make_current();

    gl::load(|e| glfw.get_proc_address_raw(e) as *const std::os::raw::c_void);

    // Mouse input
    let mut mouseClipX = 0.0f32;
    let mut mouseClipY = 0.0f32;

    // Lighting
    let lightDir: Vec3 = vec3(-0.8, 0.0, -1.0).normalize_or_zero();
    let playerLightDir: Vec3 = vec3(-1.0, -1.0, -1.0).normalize_or_zero();
    let lightColor: Vec3 = lightFactor * 1.0 * vec3(nonBlue * 0.406, nonBlue * 0.723, 1.0);
    // const lightColor: Vec3 = lightFactor * 1.0 * vec3(0.406, 0.723, 1.0);
    let floorLightColor: Vec3 =
        floorLightFactor * 1.0 * vec3(floorNonBlue * 0.406, floorNonBlue * 0.723, 1.0);
    let floorAmbientColor: Vec3 =
        floorLightFactor * 0.50 * vec3(floorNonBlue * 0.7, floorNonBlue * 0.7, 0.7);
    let ambientColor: Vec3 = lightFactor * 0.10 * vec3(nonBlue * 0.7, nonBlue * 0.7, 0.7);

    let muzzlePointLightColor = vec3(1.0, 0.2, 0.0);

    // Camera
    let cameraFollowVec = vec3(-4.0, 4.3, 0.0);
    let cameraUp = vec3(0.0, 1.0, 0.0);

    // perspective setting
    let camera = Camera::camera_vec3_up_yaw_pitch(
        // vec3(400.0, 400.0, 700.0), for current x,y world
        vec3(0.0, 20.0, 50.0), // for xz world
        vec3(0.0, 1.0, 0.0),
        -90.0, // seems camera starts by looking down the x-axis, so needs to turn left to see the plane
        -20.0,
    );

    // Player
    let player = Player {
        lastFireTime: 0.0f32,
        isTryingToFire: false,
        isAlive: true,
        aimTheta: 0.0f32,
        position: vec3(0.0, 0.0, 0.0),
        movementDir: vec2(0.0, 0.0),
        animation_name: "".to_string(),
    };

    let mut state = State {
        camera,
        delta_time: 0.0,
        last_frame: 0.0,
        first_mouse: true,
        last_x: SCR_WIDTH / 2.0,
        last_y: SCR_HEIGHT / 2.0,
        player,
        enemies: vec![],
    };

    println!("Loading assets");

    let playerShader = Rc::new(Shader::new("shaders/player_shader.vert", "shaders/player_shader.frag").unwrap());
    let wigglyShader = Rc::new(Shader::new("shaders/wiggly_shader.vert", "shaders/player_shader.frag").unwrap());

    let basicerShader = Shader::new("shaders/basicer_shader.vert", "shaders/basicer_shader.frag").unwrap();
    let basicTextureShader = Rc::new(Shader::new("shaders/basic_texture_shader.vert", "shaders/basic_texture_shader.frag").unwrap());
    let blurShader = Shader::new("shaders/basicer_shader.vert", "shaders/blur_shader.frag").unwrap();
    let sceneDrawShader = Shader::new("shaders/basicer_shader.vert", "shaders/texture_merge_shader.frag").unwrap();
    let simpleDepthShader = Shader::new("shaders/depth_shader.vert", "shaders/depth_shader.frag").unwrap();
    let textureShader = Rc::new(Shader::new("shaders/geom_shader.vert", "shaders/texture_shader.frag").unwrap());

    simpleDepthShader.use_shader();
    let lsml = simpleDepthShader.get_uniform_location("lightSpaceMatrix");

    playerShader.use_shader();

    let playerLightSpaceMatrixLocation = playerShader.get_uniform_location( "lightSpaceMatrix");
    playerShader.set_vec3("directionLight.dir", &playerLightDir);
    playerShader.set_vec3("directionLight.color", &lightColor);
    playerShader.set_vec3("ambient", &ambientColor);

    // player model textures handled externally
    let player_model = ModelBuilder::new("player", "assets/Models/Player/Player.fbx")
        .add_texture("Player", TextureType::Diffuse, "Textures/Player_D.tga")
        .add_texture( "Player", TextureType::Specular, "Textures/Player_M.tga")
        .add_texture( "Player", TextureType::Emissive, "Textures/Player_E.tga", )
        .add_texture( "Player", TextureType::Normals, "Textures/Player_NRM.tga", )
        .add_texture( "Gun", TextureType::Diffuse, "Textures/Gun_D.tga", )
        .add_texture( "Gun", TextureType::Specular, "Textures/Gun_M.tga", )
        .add_texture( "Gun", TextureType::Emissive, "Textures/Gun_E.tga", )
        .add_texture( "Gun", TextureType::Normals, "Textures/Gun_NRM.tga", )
        .build().unwrap();

    let idle = Rc::new(AnimationClip::new("idle", 55.0, 130.0, AnimationRepeat::Forever));
    let forward = Rc::new(AnimationClip::new("forward", 134.0, 154.0, AnimationRepeat::Forever));
    let backwards = Rc::new(AnimationClip::new("backwards", 159.0, 179.0, AnimationRepeat::Forever));
    let right = Rc::new(AnimationClip::new("right", 184.0, 204.0, AnimationRepeat::Forever));
    let left = Rc::new(AnimationClip::new("left", 209.0, 229.0, AnimationRepeat::Forever));
    let dying = Rc::new(AnimationClip::new("dying", 234.0, 293.0, AnimationRepeat::Once));

    player_model.play_clip(&idle);
    player_model.play_clip_with_transition(&forward, Duration::from_secs(6));
    state.player.animation_name = String::from(&forward.name);


    let wigglyBoi = ModelBuilder::new("dog", "assets/Models/Eeldog/EelDog.FBX")
        .build().unwrap();

    let mut enemy_spawner = EnemySpawner::new(monsterY);

    // logTimeSince("shaders loaded ", appStart);

    let bulletImpactSpritesheet = SpriteSheet::new(texUnit_impactSpriteSheet, 11, 0.05);
    let muzzleFlashImpactSpritesheet = SpriteSheet::new(texUnit_muzzleFlashSpriteSheet, 6, 0.05);
    let bulletStore = BulletStore::initialize_buffer_and_create(); // &threadPool);

    let textures_to_load = [
        // from angrygl
        (texUnit_impactSpriteSheet, TextureType::Diffuse, "angrygl_assets/bullet/impact_spritesheet_with_00.png"),
        (texUnit_muzzleFlashSpriteSheet, TextureType::Diffuse, "angrygl_assets/Player/muzzle_spritesheet.png"),
        //(texUnit_bullet, TextureType::Diffuse, "angrygl_assets/bullet/BulletTexture2.png"), todo: need to create this one. view video for details
        // from Unity
        (texUnit_bullet, TextureType::Diffuse, "assets/Models/Bullet/Textures/BulletTexture.png"), // using this temporarily to get code right
        // (texUnit_wigglyBoi, TextureType::Diffuse, "assets/Models/Eeldog/Eeldog_Green_Albedo.png"),
        (texUnit_floorNormal, TextureType::Normals, "assets/Models/Floor N.png"),
        (texUnit_floorDiffuse, TextureType::Diffuse, "assets/Models/Floor D.png"),
        (texUnit_floorSpec, TextureType::Specular, "assets/Models/Floor M.png"),
    ];

    let mut texture_cache = TextureCache::new();
    let mut texture_config = TextureConfig::new().set_wrap(TextureWrap::Repeat);

    for (_unit, texture_type, path) in textures_to_load {
        texture_config.texture_type = texture_type;
        let texture = texture_cache.get_or_load_texture(path, &texture_config);
        match texture {
            Ok(_) => println!("Loaded: {}", path),
            Err(e) => println!("Error loading: {} {}", path, e),
        }
    }

    let floor = Floor::new(&mut texture_cache, &basicTextureShader);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    println!("Assets loaded. Starting loop.");

    // render loop
    while !window.should_close() {
        let current_time = glfw.get_time() as f32;
        state.delta_time = current_time - state.last_frame;
        state.last_frame = current_time;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut state);
        }

        unsafe {
            gl::ClearColor(0.0, 0.02, 0.45, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let view = state.camera.get_view_matrix();
        let projection = Mat4::perspective_rh_gl(
            state.camera.zoom.to_radians(),
            SCR_WIDTH / SCR_HEIGHT,
            0.1,
            2000.0,
        );

        let camera_position = state.player.position + cameraFollowVec.clone();

        // let view = Mat4::look_at_rh(camera_position, state.player.playerPosition, state.camera.up);

        let projection_view = projection * view;

        floor.draw(&projection_view, &ambientColor);


        // let dx = worldX - state.player.playerPosition.x;
        // let dz = worldZ - state.player.playerPosition.z;
        // let aimTheta = (dx / dz).atan() + if dz < 0.0 { PI } else { 0.0 };
        let aimTheta = 0.0;


        let mut player_model_transform = Mat4::from_translation(state.player.position);
        player_model_transform *= Mat4::from_scale(Vec3::splat(playerModelScale));
        // player_model_transform *= Mat4::from_scale(Vec3::splat(0.05));
        player_model_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), aimTheta);

        playerShader.use_shader();
        playerShader.set_vec3("viewPos", &camera_position);
        playerShader.set_mat4("model", &player_model_transform);
        playerShader.set_mat4("aimRot", &Mat4::IDENTITY);
        playerShader.set_mat4("lightSpaceMatrix", &Mat4::IDENTITY);
        playerShader.set_mat4("projectionView", &projection_view);
        playerShader.set_bool("useLight", true);
        playerShader.set_vec3("ambient", &ambientColor);

        player_model.update_animation(state.delta_time);
        player_model.render(&playerShader);

        if !state.player.isAlive {
            if state.player.animation_name != String::from(&dying.name) {
                player_model.play_clip_with_transition(&dying, Duration::from_secs(1));
                state.player.animation_name = String::from(&dying.name);
            }
        }

        wigglyShader.use_shader();
        wigglyShader.set_mat4("projectionView", &projection_view);

        let mut wiggly_transform = Mat4::from_translation(vec3(0.0, 0.5, 0.0));
        // wiggly_transform *= Mat4::from_scale(Vec3::splat(playerModelScale));
        wiggly_transform *= Mat4::from_scale(Vec3::splat(0.1));
        // wiggly_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), -90.0f32.to_radians());
        wiggly_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), -90.0f32.to_radians());
        // wiggly_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), aimTheta);
        wigglyShader.set_mat4("model", &wiggly_transform);

        wigglyShader.set_vec3("viewPos", &vec3(0.0, 0.0, 1.0)); //camera_position);
        wigglyShader.set_mat4("aimRot", &Mat4::IDENTITY);
        wigglyShader.set_mat4("lightSpaceMatrix", &Mat4::IDENTITY);
        wigglyShader.set_bool("useLight", false);
        wigglyShader.set_vec3("ambient", &ambientColor);

        wigglyShader.set_float("time", current_time);
        wigglyShader.set_vec3("nosePos", &vec3(1.0, 1.0, -2.0));
        //
        // wigglyBoi.render(&wigglyShader);

        draw_wiggly_bois(&wigglyBoi, &wigglyShader, &mut state);

        enemy_spawner.update(&mut state);
        chase_player(&mut state);

        window.swap_buffers();
    }
}

//
// GLFW maps callbacks to events.
//
fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, state: &mut State) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::FramebufferSize(width, height) => {
            framebuffer_size_event(window, width, height);
        }
        glfw::WindowEvent::Key(Key::W, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Forward, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::S, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Backward, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::A, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Left, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::D, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Right, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Q, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Up, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Z, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Down, state.delta_time);
        }
        glfw::WindowEvent::CursorPos(xpos, ypos) => mouse_handler(state, xpos, ypos),
        glfw::WindowEvent::Scroll(xoffset, ysoffset) => scroll_handler(state, xoffset, ysoffset),
        _evt => {
            // println!("WindowEvent: {:?}", _evt);
        }
    }
}

// glfw: whenever the window size changed (by OS or user resize) this event fires.
// ---------------------------------------------------------------------------------------------
fn framebuffer_size_event(_window: &mut glfw::Window, width: i32, height: i32) {
    // make sure the viewport matches the new window dimensions; note that width and
    // height will be significantly larger than specified on retina displays.
    // println!("Framebuffer size: {}, {}", width, height);
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}

fn mouse_handler(state: &mut State, xposIn: f64, yposIn: f64) {
    let xpos = xposIn as f32;
    let ypos = yposIn as f32;

    if state.first_mouse {
        state.last_x = xpos;
        state.last_y = ypos;
        state.first_mouse = false;
    }

    let xoffset = xpos - state.last_x;
    let yoffset = state.last_y - ypos; // reversed since y-coordinates go from bottom to top

    state.last_x = xpos;
    state.last_y = ypos;

    state.camera.process_mouse_movement(xoffset, yoffset, true);
}

fn scroll_handler(state: &mut State, _xoffset: f64, yoffset: f64) {
    state.camera.process_mouse_scroll(yoffset as f32);
}




