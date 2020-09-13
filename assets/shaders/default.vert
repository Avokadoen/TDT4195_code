#version 430 core
uniform mat4 camera;
// TODO: generate array size while creating geometric_shape
// TODO: instancing
uniform mat4 transform[1];
layout (location = 0) in vec3 position;
void main()
{
    gl_Position = camera * transform[gl_InstanceID] * vec4(position, 1.0);
}