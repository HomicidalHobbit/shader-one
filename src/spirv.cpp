/*
Author: Richard Underhill 2020
*/

#include "spirv.h"

#include <algorithm>
#include <cstdio>
#include <sstream>

#include "DirStackFileIncluder.h"
#include "book.h"
#include "keywords.h"

thread_local std::list<Shader> shaders;
thread_local std::vector<unsigned int> spirv;
thread_local KeywordCombo shaderHashes;
thread_local std::string shaderComboResult;
thread_local std::vector<KeywordCombo> threadKeywordCombos;
thread_local std::vector<Keyword> threadKeywords;
thread_local std::vector<Keyword> threadEnabledKeywords;
thread_local uint64_t keywordsID;
thread_local std::string source{};
thread_local std::string preamble{};
thread_local std::string warningsErrors;
thread_local std::string queryIDResult;
thread_local std::unique_ptr<spvtools::SpirvTools> tools;
thread_local std::unique_ptr<spvtools::Optimizer> optimizer;
thread_local bool keywordAddEnable;

std::vector<Keyword> keywords;
std::vector<Keyword> enabledKeywords;
std::atomic<bool> keywordsEnabled(false);
std::mutex kw_mutex;
std::mutex enabled_mutex;

static TBuiltInResource Resources;

const TBuiltInResource DefaultTBuiltInResource = {
    /* .MaxLights = */ 32,
    /* .MaxClipPlanes = */ 6,
    /* .MaxTextureUnits = */ 32,
    /* .MaxTextureCoords = */ 32,
    /* .MaxVertexAttribs = */ 64,
    /* .MaxVertexUniformComponents = */ 4096,
    /* .MaxVaryingFloats = */ 64,
    /* .MaxVertexTextureImageUnits = */ 32,
    /* .MaxCombinedTextureImageUnits = */ 80,
    /* .MaxTextureImageUnits = */ 32,
    /* .MaxFragmentUniformComponents = */ 4096,
    /* .MaxDrawBuffers = */ 32,
    /* .MaxVertexUniformVectors = */ 128,
    /* .MaxVaryingVectors = */ 8,
    /* .MaxFragmentUniformVectors = */ 16,
    /* .MaxVertexOutputVectors = */ 16,
    /* .MaxFragmentInputVectors = */ 15,
    /* .MinProgramTexelOffset = */ -8,
    /* .MaxProgramTexelOffset = */ 7,
    /* .MaxClipDistances = */ 8,
    /* .MaxComputeWorkGroupCountX = */ 65535,
    /* .MaxComputeWorkGroupCountY = */ 65535,
    /* .MaxComputeWorkGroupCountZ = */ 65535,
    /* .MaxComputeWorkGroupSizeX = */ 1024,
    /* .MaxComputeWorkGroupSizeY = */ 1024,
    /* .MaxComputeWorkGroupSizeZ = */ 64,
    /* .MaxComputeUniformComponents = */ 1024,
    /* .MaxComputeTextureImageUnits = */ 16,
    /* .MaxComputeImageUniforms = */ 8,
    /* .MaxComputeAtomicCounters = */ 8,
    /* .MaxComputeAtomicCounterBuffers = */ 1,
    /* .MaxVaryingComponents = */ 60,
    /* .MaxVertexOutputComponents = */ 64,
    /* .MaxGeometryInputComponents = */ 64,
    /* .MaxGeometryOutputComponents = */ 128,
    /* .MaxFragmentInputComponents = */ 128,
    /* .MaxImageUnits = */ 8,
    /* .MaxCombinedImageUnitsAndFragmentOutputs = */ 8,
    /* .MaxCombinedShaderOutputResources = */ 8,
    /* .MaxImageSamples = */ 0,
    /* .MaxVertexImageUniforms = */ 0,
    /* .MaxTessControlImageUniforms = */ 0,
    /* .MaxTessEvaluationImageUniforms = */ 0,
    /* .MaxGeometryImageUniforms = */ 0,
    /* .MaxFragmentImageUniforms = */ 8,
    /* .MaxCombinedImageUniforms = */ 8,
    /* .MaxGeometryTextureImageUnits = */ 16,
    /* .MaxGeometryOutputVertices = */ 256,
    /* .MaxGeometryTotalOutputComponents = */ 1024,
    /* .MaxGeometryUniformComponents = */ 1024,
    /* .MaxGeometryVaryingComponents = */ 64,
    /* .MaxTessControlInputComponents = */ 128,
    /* .MaxTessControlOutputComponents = */ 128,
    /* .MaxTessControlTextureImageUnits = */ 16,
    /* .MaxTessControlUniformComponents = */ 1024,
    /* .MaxTessControlTotalOutputComponents = */ 4096,
    /* .MaxTessEvaluationInputComponents = */ 128,
    /* .MaxTessEvaluationOutputComponents = */ 128,
    /* .MaxTessEvaluationTextureImageUnits = */ 16,
    /* .MaxTessEvaluationUniformComponents = */ 1024,
    /* .MaxTessPatchComponents = */ 120,
    /* .MaxPatchVertices = */ 32,
    /* .MaxTessGenLevel = */ 64,
    /* .MaxViewports = */ 16,
    /* .MaxVertexAtomicCounters = */ 0,
    /* .MaxTessControlAtomicCounters = */ 0,
    /* .MaxTessEvaluationAtomicCounters = */ 0,
    /* .MaxGeometryAtomicCounters = */ 0,
    /* .MaxFragmentAtomicCounters = */ 8,
    /* .MaxCombinedAtomicCounters = */ 8,
    /* .MaxAtomicCounterBindings = */ 1,
    /* .MaxVertexAtomicCounterBuffers = */ 0,
    /* .MaxTessControlAtomicCounterBuffers = */ 0,
    /* .MaxTessEvaluationAtomicCounterBuffers = */ 0,
    /* .MaxGeometryAtomicCounterBuffers = */ 0,
    /* .MaxFragmentAtomicCounterBuffers = */ 1,
    /* .MaxCombinedAtomicCounterBuffers = */ 1,
    /* .MaxAtomicCounterBufferSize = */ 16384,
    /* .MaxTransformFeedbackBuffers = */ 4,
    /* .MaxTransformFeedbackInterleavedComponents = */ 64,
    /* .MaxCullDistances = */ 8,
    /* .MaxCombinedClipAndCullDistances = */ 8,
    /* .MaxSamples = */ 4,
    /* .maxMeshOutputVerticesNV = */ 256,
    /* .maxMeshOutputPrimitivesNV = */ 512,
    /* .maxMeshWorkGroupSizeX_NV = */ 32,
    /* .maxMeshWorkGroupSizeY_NV = */ 1,
    /* .maxMeshWorkGroupSizeZ_NV = */ 1,
    /* .maxTaskWorkGroupSizeX_NV = */ 32,
    /* .maxTaskWorkGroupSizeY_NV = */ 1,
    /* .maxTaskWorkGroupSizeZ_NV = */ 1,
    /* .maxMeshViewCountNV = */ 4,

    /* .limits = */
    //{
        /* .nonInductiveForLoops = */ 1,
        /* .whileLoops = */ 1,
        /* .doWhileLoops = */ 1,
        /* .generalUniformIndexing = */ 1,
        /* .generalAttributeMatrixVectorIndexing = */ 1,
        /* .generalVaryingIndexing = */ 1,
        /* .generalSamplerIndexing = */ 1,
        /* .generalVariableIndexing = */ 1,
        /* .generalConstantMatrixVectorIndexing = */ 1,
  //  }
  };

enum TOptions {
  EOptionNone = 0,
  EOptionIntermediate = (1 << 0),
  EOptionSuppressInfolog = (1 << 1),
  EOptionMemoryLeakMode = (1 << 2),
  EOptionRelaxedErrors = (1 << 3),
  EOptionGiveWarnings = (1 << 4),
  EOptionLinkProgram = (1 << 5),
  EOptionMultiThreaded = (1 << 6),
  EOptionDumpConfig = (1 << 7),
  EOptionDumpReflection = (1 << 8),
  EOptionSuppressWarnings = (1 << 9),
  EOptionDumpVersions = (1 << 10),
  EOptionSpv = (1 << 11),
  EOptionHumanReadableSpv = (1 << 12),
  EOptionVulkanRules = (1 << 13),
  EOptionDefaultDesktop = (1 << 14),
  EOptionOutputPreprocessed = (1 << 15),
  EOptionOutputHexadecimal = (1 << 16),
  EOptionReadHlsl = (1 << 17),
  EOptionCascadingErrors = (1 << 18),
  EOptionAutoMapBindings = (1 << 19),
  EOptionFlattenUniformArrays = (1 << 20),
  EOptionNoStorageFormat = (1 << 21),
  EOptionKeepUncalled = (1 << 22),
  EOptionHlslOffsets = (1 << 23),
  EOptionHlslIoMapping = (1 << 24),
  EOptionAutoMapLocations = (1 << 25),
  EOptionDebug = (1 << 26),
  EOptionStdin = (1 << 27),
  EOptionOptimizeDisable = (1 << 28),
  EOptionOptimizeSize = (1 << 29),
  EOptionInvertY = (1 << 30),
  EOptionDumpBareVersion = (1 << 31),
};

extern "C" void Initialise() {
  printf("Initialising!\n");
  glslang::InitializeProcess();
  tools.reset(new spvtools::SpirvTools(SPV_ENV_UNIVERSAL_1_3));
  optimizer.reset(new spvtools::Optimizer(SPV_ENV_UNIVERSAL_1_3));

  /*
  auto print_msg_to_stderr = [](spv_message_level_t, const char*, const
  spv_position&, const char* m) { std::cerr << "error: " << m << std::endl;
  };
  tools->SetMessageConsumer(print_msg_to_stderr);
  optimizer->SetMessageConsumer(print_msg_to_stderr);
  */
  Resources = DefaultTBuiltInResource;
  keywordAddEnable = true;
}

extern "C" std::size_t GetSpirvSize() {
  return spirv.size();
}

extern "C" const char* DecompileToGLSL() {
  spirv_cross::CompilerGLSL glsl(spirv);
  source = glsl.compile();
  return source.c_str();
}

extern "C" const char* DecompileToHLSL() {
  spirv_cross::CompilerHLSL hlsl(spirv);
  source = hlsl.compile();
  return source.c_str();
}

extern "C" const char* DecompileToMetal() {
  spirv_cross::CompilerMSL msl(spirv);
  source = msl.compile();
  return source.c_str();
}

extern "C" void* GetCompiledSource() {
  return (void*)source.data();
}

extern "C" void* CreateProgram() {
  return new glslang::TProgram;
}

extern "C" void DeleteProgram(void* program) {
  delete (glslang::TProgram*)program;
}

extern "C" std::size_t CompileShader(EShLanguage stage, const char* sourcecode) {
  bool compile_failed = false;
  printf("Compiling Shader Stage: %i\n", stage);
  if (!shaderHashes.hash) {
    if (threadEnabledKeywords.size()) {
      for (auto& kw : threadEnabledKeywords) {
        AddKeyword(kw.hash, kw.keyword);
      }
    }

    if (keywordsEnabled) {
      std::lock_guard<std::mutex> guard(enabled_mutex);
      for (auto& kw : enabledKeywords) {
        AddKeyword(kw.hash, kw.keyword);
      }
    }
    if (shaderHashes.hashes.size()) {
      // Make sure the keywords are in a deteministic order for hash generation
      std::sort(shaderHashes.hashes.begin(), shaderHashes.hashes.end(),
                [](uint64_t a, uint64_t b) { return a < b; });

      // Remove any duplicates
      shaderHashes.hashes.erase(
          std::unique(shaderHashes.hashes.begin(), shaderHashes.hashes.end()),
          shaderHashes.hashes.end());

      shaderComboResult.clear();
      for (auto& kwh : shaderHashes.hashes) {
        //           std::cout << "0x" << std::hex << kwh << " ";
        shaderComboResult += std::to_string(kwh);
      }
      shaderHashes.hash = Hash(shaderComboResult);

      // Authenticate this combination
      if (!AuthenticateKeywords()) {
        printf(
            "Panic! Compile Failed due to Keyword Combination Collision!!!!\n");
        return 0;
      }
      // std::cout << std::endl;
    } else {
      keywordsID = 0;
    }
  }

  keywordsID = shaderHashes.hash;
  shaders.emplace_back(sourcecode, stage);
  glslang::TShader& shader = shaders.back().shader;
  shader.setStrings(&sourcecode, 1);
  shader.setEnvInput(glslang::EShSourceGlsl, stage, glslang::EShClientVulkan,
                     100);
  shader.setEnvClient(glslang::EShClientVulkan, glslang::EShTargetVulkan_1_0);
  shader.setEnvTarget(glslang::EShTargetSpv, glslang::EShTargetSpv_1_0);
  shader.setNanMinMaxClamp(false);
  shader.setInvertY(true);
  shader.setAutoMapBindings(true);
  shader.setAutoMapLocations(true);
  // printf("Preamble:\n%s", preamble.c_str());
  if (!preamble.empty()) {
    shader.setPreamble(preamble.c_str());
  }

  DirStackFileIncluder includer;

  if (!shader.parse(&Resources, 100, false, EShMsgDefault, includer))
    compile_failed = true;

  if (compile_failed) {
    printf("Compile Failed!\n%s\n%s\n", sourcecode, shader.getInfoLog());
    shaders.pop_back();
    return 0;
  } else {
    printf("Compile OK! ");
  }
  keywordAddEnable = false;
  return shaders.size();
}

extern "C" uint64_t GetKeywordsID() {
  return keywordsID;
}

Shader* GetShader(std::size_t handle) {
  if (handle) {
    // Get the address of this shader by walking the list
    std::size_t count = 0;
    for (auto& shader : shaders) {
      if (++count == handle) {
        return &shader;
      }
    }
  }
  return nullptr;
}

extern "C" std::size_t Recompile(std::size_t handle) {
  Shader* parent = GetShader(handle);
  std::size_t new_handle = 0;
  if (parent && parent->parent == nullptr) {
    new_handle =
        CompileShader(parent->shader.getStage(), parent->source.c_str());
    shaders.back().parent = parent;
  }
  return new_handle;
}

extern "C" void Add(glslang::TProgram* program, std::size_t handle) {
  Shader* shader = GetShader(handle);
  if (shader) {
    program->addShader(&shader->shader);
  }
}

extern "C" void SetPreamble(const char* sourcecode) {
  preamble = std::string(sourcecode);
}

extern "C" void ClearPreamble() {
  preamble.clear();
}

extern "C" void ClearShaderCache() {
  shaders.clear();
}

extern "C" bool Link(glslang::TProgram* program) {
  bool result = false;
  if (!program->link(EShMsgDefault)) {
    printf("Link Failed!\n%s\n", program->getInfoLog());
  } else {
    printf("Link OK\n");
    result = true;
  }
  shaderHashes.hash = 0;
  shaderHashes.hashes.clear();
  preamble.clear();
  keywordAddEnable = true;
  return result;
}

extern "C" void PrintSpirv() {
  printf("SPIRV size: %zu\n", spirv.size());
  std::size_t count = 0;
  for (auto& entry : spirv) {
    printf("%.8x ", entry);
    if (!(++count & 7)) {
      printf("\n");
    }
  }
  printf("\n");
}

extern "C" void CopyToSpirv(unsigned int* ptr, std::size_t length) {
  spirv.resize(length);
  memcpy(spirv.data(), ptr, sizeof(unsigned int) * length);
}

extern "C" void* GetSpirvForStage(glslang::TProgram* program,
                                  EShLanguage stage) {
  unsigned int* sp = nullptr;
  if (program->getIntermediate(stage)) {
    spirv.clear();
    spv::SpvBuildLogger logger;
    glslang::SpvOptions spvOptions;
    spvOptions.generateDebugInfo = true;
    spvOptions.validate = true;
    spvOptions.disassemble = true;
    glslang::GlslangToSpv(*program->getIntermediate(stage), spirv, &logger,
                          &spvOptions);
    printf("Stage %i SPIRV\n", stage);
    printf("%s", logger.getAllMessages().c_str());
    printf("%s", preamble.c_str());
    // PrintSpirv();
    sp = spirv.data();
  }
  return sp;
}

extern "C" bool Assemble(const char* code) {
  source = std::string(code);
  return tools->Assemble(source, &spirv);
}

extern "C" bool Validate(unsigned int* ptr, std::size_t length) {
  CopyToSpirv(ptr, length);
  return tools->Validate(spirv);
}

extern "C" void Optimize() {}

extern "C" const char* Disassemble(unsigned int* ptr, std::size_t length) {
  CopyToSpirv(ptr, length);
  source.clear();
  tools->Disassemble(spirv, &source);
  return source.c_str();
}

extern "C" void Shutdown() {
  ClearShaderCache();
  glslang::FinalizeProcess();
}
