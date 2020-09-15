#version 430 core
layout (location = 0) in vec3 position;

uniform mat4 camera;
// TODO: generate array size while creating geometric_shape
uniform mat4 transform[1];
void main()
{
    gl_Position = camera * transform[gl_InstanceID] * vec4(position, 1.0);
}