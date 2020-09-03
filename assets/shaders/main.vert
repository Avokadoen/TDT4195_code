#version 430 core

uniform mat4 camera;

uniform mat4 transform;

in vec3 position;

void main()
{
    gl_Position = camera * transform * vec4(position, 1.0f);
}