#version 330 core

in vec3 vNormal;    // Viene del vertex shader
in vec3 vWorldPos;  // no lo usamos mucho ahora, pero podría servir

out vec4 FragColor;

// Uniforms para una luz direccional sencilla
uniform vec3 lightDir;   // dirección de la luz
uniform vec3 lightColor; // color de la luz
uniform vec3 objectColor; // color base del objeto

void main()
{
    // 1) Normalizar la normal
    vec3 N = normalize(vNormal);
    // 2) Direccion de la luz
    //    Si 'lightDir' apunta DESDE el objeto hacia la luz, pon L = -lightDir, o viceversa.
    vec3 L = normalize(lightDir);

    // 3) Difuso (Lambert)
    float diff = max(dot(N, L), 0.0);

    // 4) Color difuso
    vec3 diffuse = diff * lightColor * objectColor;

    // 5) Pequeña componente ambiental
    vec3 ambient = 0.1 * objectColor;

    // 6) Sumar y escribir
    vec3 finalColor = ambient + diffuse;
    FragColor = vec4(finalColor, 1.0);
}