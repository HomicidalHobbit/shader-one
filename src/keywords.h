#pragma once

#include <iomanip>
#include <string>

uint32_t Hash32(const char *name);
uint32_t Hash32(std::string &name);
uint64_t Hash(const char *name);
uint64_t Hash(std::string &name);

void AddKeyword(uint64_t hash, const char *keyword);
void AddKeyword(uint64_t hash, const std::string &keyword);
uint64_t FindKeyword(const char *keyword);
bool AuthenticateKeywords();

extern "C" {
uint64_t ReserveKeyword(const char *keyword);
bool EnableKeyword(const char *keyword);
bool EnableGlobalKeyword(const char *keyword);
bool DisableKeyword(const char *keyword);
bool DisableGlobalKeyword(const char *keyword);
uint64_t AddKeyword(const char *keyword);
const char *GetKeywordsFromID(uint64_t id);
const char *PrintKeywords();
}