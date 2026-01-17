precision mediump float;

varying vec2 vPosition;
uniform vec2 uMiddle;
uniform vec3 uAbove_uArea_uBorder;
uniform vec2 uRestrict_uVisual;
uniform vec2 uSmokePosition;
uniform float uSmokeRadius;

float preciseLength(vec2 vec) {
    #define LENGTH_SCALE 64.0
    return length(vec * (1.0 / LENGTH_SCALE)) * LENGTH_SCALE;
}

void main() {
    float area = (vPosition.y - uAbove_uArea_uBorder.y) * uAbove_uArea_uBorder.x;
    float border = preciseLength(vPosition) - uAbove_uArea_uBorder.z;
    gl_FragColor = vec4(0.01, 0.01, 0.01, 1.0) * clamp(max(border, area) * 0.1, 0.0, 0.4);
    vec2 frPos = fract(0.01 * vPosition);
    gl_FragColor.x += (smoothstep(1., frPos.x,.98) + smoothstep(1., frPos.y,.98) + smoothstep(.0, frPos.x,.02) + smoothstep(.0, frPos.y,.02)) * clamp(max(border, area) * 0.06, 0.0, 0.9);
    gl_FragColor = mix(gl_FragColor, vec4(0.0, 0.0174, 0.0835, 1.0), clamp((preciseLength(vPosition - uMiddle) - uRestrict_uVisual.y) * 0.1, 0.0, uRestrict_uVisual.x));
    
    // Smoke screen effect: fog with grid pattern (matches reference image)
    if (uSmokeRadius > 0.0) {
        float smokeDist = preciseLength(vPosition - uSmokePosition);
        
        // Soft fog effect that fades from center to edge
        float fogIntensity = 1.0 - smoothstep(0.0, uSmokeRadius, smokeDist);
        
        // Grid/dotted pattern
        vec2 gridPos = vPosition * 0.15; // Scale grid
        float gridX = abs(fract(gridPos.x) - 0.5);
        float gridY = abs(fract(gridPos.y) - 0.5);
        float grid = smoothstep(0.48, 0.5, gridX) + smoothstep(0.48, 0.5, gridY);
        grid = clamp(grid, 0.0, 0.4);
        
        // Combine fog and grid
        vec4 fogColor = vec4(0.15, 0.25, 0.4, 1.0); // Blue-gray fog
        float alpha = fogIntensity * 0.35 + grid * fogIntensity * 0.15;
        gl_FragColor = mix(gl_FragColor, fogColor, alpha);
        
        // White circle outline at smoke edge
        float ringDist = abs(smokeDist - uSmokeRadius);
        float ring = 1.0 - smoothstep(0.0, 3.0, ringDist);
        gl_FragColor = mix(gl_FragColor, vec4(1.0, 1.0, 1.0, 0.6), ring * 0.5);
    }
}
