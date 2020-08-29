#version 430 core

uniform mat4 camera;

in vec3 position;

void main()
{
    gl_Position = camera * vec4(position, 1.0f);
}