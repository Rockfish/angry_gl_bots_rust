#version 330 core
in vec2 TexCoord;

out vec4 FragColor;

uniform sampler2D tex;

void main() {
  FragColor = texture(tex, TexCoord);

//  FragColor = vec4(0.5, 0.9, 0.2, 1.0);
}
