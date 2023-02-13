#version 330 core
out vec4 FragColor;

in vec3 ourColor;
in vec4 posColor;
in vec2 TexCoord;

layout (binding = 0) uniform sampler2D ourTexture;
layout (binding = 1) uniform sampler2D ourFace;

void main()
{
    FragColor = mix(texture(ourTexture, TexCoord), texture(ourFace, vec2(TexCoord.x, TexCoord.y * -1.0)), 0.2);
}