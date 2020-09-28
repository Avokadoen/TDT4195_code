#version 430 core

out vec4 color;

// SOURCE: https://learnopengl.com/Advanced-OpenGL/Depth-testing
float near = 0.01;
float far = 40;

void main()
{
    float ndc = gl_FragCoord.z * 2.0 - 1.0; 
    float linearDepth = (2.0 * near * far) / (far + near - ndc * (far - near));	
    color = vec4(vec3(1 - linearDepth), 1);
}