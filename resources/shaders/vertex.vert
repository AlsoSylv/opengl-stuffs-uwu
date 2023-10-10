#version 450 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

layout(std140, binding = 0) uniform MatrixBlock {
    mat4 projection;
    mat4 view;
    mat4 model;
};

layout(std140, binding = 1) uniform Tile {
    uint texture_id_and_overlauy;
}

void main()
{
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    TexCoord = aTexCoord;
}