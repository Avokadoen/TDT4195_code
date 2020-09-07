#version 430 core

uniform mat4 camera;

uniform mat4 transform;

layout (location = 0) in vec3 position;
layout (location = 1) in vec4 color;

out vec4 vertColor;

void main()
{
    vertColor = color;
    gl_Position = camera * transform * vec4(position, 1.0);
}