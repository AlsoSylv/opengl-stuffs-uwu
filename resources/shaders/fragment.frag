#version 330 core
out vec4 FragColor;

in vec3 ourColor;
in vec4 posColor;
in vec2 TexCoord;

layout (binding = 0) uniform sampler2D ourTexture;

void main()
{
    FragColor = texture(ourTexture, TexCoord);
}