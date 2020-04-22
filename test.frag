#version 450

	uniform sampler2D samplerColor;

	in vec2 inUV;
#ifdef LOD_BIAS
	in float inLodBias;
#endif
	in vec3 inNormal;
#ifdef LIT
	in vec3 inViewVec;
	in vec3 inLightVec;
#endif

	out vec4 outFragColor;

	void main()
	{
	#ifdef LOD_BIAS
		vec4 color = texture(samplerColor, inUV, inLodBias);
	#else
		vec4 color = texture(samplerColor, inUV);
	#endif

	#ifdef LIT
		vec3 N = normalize(inNormal);
		vec3 L = normalize(inLightVec);
		vec3 V = normalize(inViewVec);
		vec3 R = reflect(-L, N);
		vec3 diffuse = max(dot(N, L), 0.0) * vec3(1.0);
		float specular = pow(max(dot(R, V), 0.0), 16.0) * color.a;
		outFragColor = vec4(diffuse * color.rgb + specular, 1.0);
	#else
		outFragColor = vec4(color.xyz, 1.0);
	#endif
	}