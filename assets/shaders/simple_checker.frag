#version 430 core

// uniform float elapsed;
uniform float tileSize = 20;

out vec4 color;

void main()
{
    bool isEvenX = mod(gl_FragCoord.x / tileSize, 2) < 1;
    float checker = float(isEvenX) * 1.0;

    bool isEvenY = mod(gl_FragCoord.y / tileSize, 2) < 1;
    // hiding our sinful shader 'if' with syntax :)
    checker = isEvenY ? 1 - checker : checker; 

    color = vec4(checker, checker, checker, 1.0f);
}