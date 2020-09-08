#version 430 core

uniform mat4 camera;

// TODO: generate array size while creating geometric_shape
// TODO: do instancing
uniform mat4 transform[10];

layout (location = 0) in vec3 position;
layout (location = 1) in vec4 color;

out vec4 vertColor;

void main()
{
    vertColor = color;
    gl_Position = camera * transform[gl_InstanceID] * vec4(position, 1.0);
}