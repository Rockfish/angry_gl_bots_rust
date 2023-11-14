
# Angry GL Rust

Port of the cpp project [AngryGL](https://github.com/ntcaston/AngryGL) to Rust. Which is an OpenGL clone of the [Unity Angry Bots ECS sample project](https://github.com/UnityTechnologies/AngryBots_ECS)

## Dependencies

* small_gl_core - https://github.com/Rockfish/small_gl_core

Which is in turn dependent on:

* glfw - For window and OpenGL context. https://docs.rs/glfw/0.52.0/glfw/


* glad - OpenGL API bindings generated from https://gen.glad.sh/

    * See https://github.com/Dav1dde/glad/tree/glad2


* glam - Math library. https://docs.rs/glam/latest/glam/


* image - Image library. https://docs.rs/image/0.24.7/image/


* assimp - For modeling loading using the Assimp library, https://github.com/assimp/assimp


* russimp - For assimp rust bindings, https://github.com/jkvargas/russimp

## Assets

The original Unity Angry Bot assets must be added to the project like this:

    assets/Models/Player/Player.fbx
    assets/Models/Player/Textures/Gun_D.tga
    assets/Models/Player/Textures/Gun_E.tga
    assets/Models/Player/Textures/Gun_M.tga
    assets/Models/Player/Textures/Gun_NRM.tga
    assets/Models/Player/Textures/Player_D.tga
    assets/Models/Player/Textures/Player_E.tga
    assets/Models/Player/Textures/Player_M.tga
    assets/Models/Player/Textures/Player_NRM.tga
    assets/Models/Bullet/BulletTexture2.png
    assets/Models/Bullet_hit_metal_enemy_4.wav
    assets/Models/Floor_D.psd
    assets/Models/Floor_M.psd
    assets/Models/Floor_N.psd
    assets/Audio/Player_SFX/player_shooting.wav
    assets/Audio/Player_SFX/player_shooting_one.wav
    assets/Models/Eeldog/EelDog.FBX
    assets/Models/Eeldog/Eeldog_Albedo.png
    assets/Models/Eeldog/Eeldog_Normal.tif