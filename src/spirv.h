#pragma once
#include <glslang/SPIRV/GlslangToSpv.h>

#include <atomic>
#include <mutex>
#include <spirv-tools/libspirv.hpp>
#include <spirv-tools/optimizer.hpp>
#include <spirv_cross/spirv_glsl.hpp>
#include <spirv_cross/spirv_hlsl.hpp>
#include <spirv_cross/spirv_msl.hpp>
#include <vector>

struct Shader {
  Shader(const char *src, EShLanguage stage, Shader *pshader = nullptr)
      : source(std::string(src)), shader(glslang::TShader(stage)),
        parent(pshader) {}

  std::string source;
  glslang::TShader shader;
  Shader *parent;
};

struct Keyword {
  std::string keyword;
  uint64_t hash;
};

struct KeywordCombo {
  uint64_t hash;
  std::vector<uint64_t> hashes;
};

extern thread_local std::vector<unsigned int> spirv;
extern thread_local KeywordCombo shaderHashes;
extern thread_local std::string preamble;
extern thread_local std::vector<KeywordCombo> threadKeywordCombos;
extern thread_local std::vector<Keyword> threadKeywords;
extern thread_local std::vector<Keyword> threadEnabledKeywords;
extern thread_local std::string queryIDResult;
extern thread_local uint64_t keywordsID;
extern thread_local bool keywordAddEnable;

extern std::vector<Keyword> keywords;
extern std::vector<Keyword> enabledKeywords;
extern std::atomic<bool> keywordsEnabled;
extern std::mutex kw_mutex;
extern std::mutex enabled_mutex;
