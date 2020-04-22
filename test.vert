#version 450

	in vec3 inPos;
	in vec2 inUV;
	in vec3 inNormal;

	out vec2 outUV;
#ifdef LOD_BIAS
	out float outLodBias;
#endif
	out vec3 outNormal;

#ifdef LIT
	out vec3 outViewVec;
	out vec3 outLightVec;
#endif

	out gl_PerVertex
	{
		vec4 gl_Position;
	};

	uniform UBO 
	{
		mat4 projection;
		mat4 model;
		vec4 viewPos;
#ifdef LOD_BIAS
		float lodBias;
#endif
	} ubo;

	void main()
	{
		outUV = inUV;
#ifdef LOD_BIAS
		outLodBias = ubo.lodBias;
#endif
		vec3 worldPos = vec3(ubo.model * vec4(inPos, 1.0));
		gl_Position = ubo.projection * ubo.model * vec4(inPos.xyz, 1.0);

		vec4 pos = ubo.model * vec4(inPos, 1.0);
		outNormal = mat3(inverse(transpose(ubo.model))) * inNormal;
#ifdef LIT
		vec3 lightPos = vec3(0.0);
		vec3 lPos = mat3(ubo.model) * lightPos.xyz;
		outLightVec = lPos - pos.xyz;
		outViewVec = ubo.viewPos.xyz - pos.xyz;
#endif
	}


/*

1 A
2 B

1 2
1 B
A 2
A B



1 A
2 B
3 C


1 2 3
1 2 C
1 B 3
1 B C
A 2 3
A 2 C
A B 3
A B C


1 A
2 B O

1 2
1 B
1 O
A 2
A B
A O


1 A O
2 B
3 C P


1 2 3
1 2 C
1 2 P
1 B 3
1 B C
1 B P

A 2 3
A 2 C
A 2 P
A B 3
A B C
A B P

O 2 3
O 2 C
O 2 P
O B 3
O B C
O B P






*/