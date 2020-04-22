#include "keywords.h"

#include <iostream>
#include <sstream>

#include "spirv.h"

// FNV-1a hash function:
// https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
uint32_t Hash32(const char* name) {
  uint32_t value = 2166136261U;
  uint32_t mult = 16777619U;
  std::size_t length = strlen(name);
  for (std::size_t i = 0; i < length; ++i) {
    value = (value ^ name[i]) * mult;
  }
  return value;
}

uint32_t Hash32(std::string& name) {
  return Hash32(name.c_str());
}

// FNV-1a hash function:
// https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
uint64_t Hash(const char* name) {
  uint64_t value = 14695981039346656037U;
  uint64_t mult = 1099511628211U;
  std::size_t length = strlen(name);
  for (std::size_t i = 0; i < length; ++i) {
    value = (value ^ name[i]) * mult;
  }
  return value;
}

uint64_t Hash(std::string& name) {
  return Hash(name.c_str());
}

void AddKeyword(uint64_t hash, const char* keyword) {
  // Indicate that we're still adding keywords
  shaderHashes.hash = 0;

  // Make sure don't duplicate hashes (probably harmless in the preamble though)
  for (auto& kw : shaderHashes.hashes) {
    if (kw == hash) {
      return;
    }
  }
  shaderHashes.hashes.push_back(hash);
  preamble.append("#define ");
  preamble.append(keyword);
  preamble.append("\n");
}

void AddKeyword(uint64_t hash, const std::string& keyword) {
  AddKeyword(hash, keyword.c_str());
}

bool DoReserveKeyword(const std::string& newkw,
                      uint64_t hash,
                      bool updateLocal = false) {
  std::lock_guard<std::mutex> guard(kw_mutex);
  bool found = false;
  for (auto& kw : keywords) {
    // Check the hash
    if (kw.hash == hash) {
      // Now check the string itself, if they are different show a collision
      // error
      if (kw.keyword != newkw) {
        std::cout << "COLLISION! "
                  << "'" << newkw << "' collides with hash 0x" << std::hex
                  << hash << std::dec << " for '" << kw.keyword << "'"
                  << std::endl;
        return false;
      }
      found = true;
    }
    if (found)
      break;
  }

  if (!found) {
    keywords.push_back(Keyword{newkw, hash});
    if (updateLocal)
      threadKeywords.push_back(Keyword{newkw, hash});
  } else {
    if (updateLocal) {
      found = false;
      for (auto& kw : threadKeywords) {
        if (kw.hash == hash) {
          found = true;
          break;
        }
      }
      if (!found)
        threadKeywords.push_back(Keyword{newkw, hash});
    }
  }
  return true;
}

uint64_t FindKeyword(const char* keyword) {
  uint64_t hash = Hash(keyword);
  bool found = false;
  for (auto& kw : threadKeywords) {
    if (kw.hash == hash) {
      found = true;
      break;
    }
  }
  if (!found) {
    std::lock_guard<std::mutex> guard(kw_mutex);
    for (auto& kw : keywords) {
      if (kw.hash == hash) {
        found = true;
        threadKeywords.push_back(Keyword{keyword, hash});
        break;
      }
    }
  }
  if (!found)
    return 0;
  return hash;
}

uint64_t ReserveKeyword(const char* keyword) {
  keywordsID = 0;
  std::string newkw(keyword);
  uint64_t hash = Hash(keyword);
  if (DoReserveKeyword(newkw, hash)) {
    keywordsID = hash;
    return hash;
  }
  return 0;
}

bool EnableKeyword(const char* keyword) {
  uint64_t hash = FindKeyword(keyword);
  if (hash) {
    for (auto& kw : threadEnabledKeywords) {
      if (kw.hash == hash)
        return true;
    }
    threadEnabledKeywords.push_back(Keyword{keyword, hash});
    return true;
  }
  return false;
}

bool EnableGlobalKeyword(const char* keyword) {
  uint64_t hash = FindKeyword(keyword);
  if (hash) {
    std::lock_guard<std::mutex> guard(enabled_mutex);
    for (auto& kw : enabledKeywords) {
      if (kw.hash == hash) {
        return true;
      }
    }
    enabledKeywords.push_back(Keyword{keyword, hash});
    keywordsEnabled = true;
    return true;
  }
  return false;
}

bool DisableKeyword(const char* keyword) {
  uint64_t hash = FindKeyword(keyword);
  if (hash) {
    bool found = false;
    if (!threadEnabledKeywords.empty()) {
      for (auto& kw : threadEnabledKeywords) {
        if (kw.hash == hash) {
          found = true;
          break;
        }
      }

      if (found) {
        std::vector<Keyword>::iterator it = threadEnabledKeywords.begin();
        while (it != threadEnabledKeywords.end()) {
          if (it->hash == hash)
            it = threadEnabledKeywords.erase(it);
          else
            it++;
        }
        return true;
      }
    }
    return false;
  }
  return false;
}

bool DisableGlobalKeyword(const char* keyword) {
  uint64_t hash = FindKeyword(keyword);
  if (hash) {
    bool found = false;
    if (keywordsEnabled) {
      std::lock_guard<std::mutex> guard(enabled_mutex);
      for (auto& kw : enabledKeywords) {
        if (kw.hash == hash) {
          found = true;
          break;
        }
      }

      if (found) {
        std::vector<Keyword>::iterator it = enabledKeywords.begin();
        while (it != enabledKeywords.end()) {
          if (it->hash == hash)
            it = enabledKeywords.erase(it);
          else
            it++;
        }
        if (enabledKeywords.empty())
          keywordsEnabled = false;
        return true;
      }
    }
    return false;
  }
  return false;
}

uint64_t AddKeyword(const char* keyword) {
  keywordsID = 0;
  if (!keywordAddEnable) {
    printf("Cannot Add More Keywords until the stages have been linked!!!");
    return 0;
  }

  std::string newkw(keyword);
  uint64_t hash = Hash(keyword);

  // Check to see if this thread has this keyword
  for (auto& kw : threadKeywords) {
    if (kw.hash == hash) {
      // Now check the string itself, if they are different show an collision
      // error
      if (kw.keyword != newkw) {
        std::cout << "COLLISION! "
                  << "'" << newkw << "' collides with hash 0x" << std::hex
                  << hash << std::dec << " for '" << kw.keyword << "'"
                  << std::endl;
        return 0;
      } else {
        AddKeyword(hash, keyword);
        return hash;
      }
    }
  }

  DoReserveKeyword(newkw, hash, true);
  AddKeyword(hash, keyword);
  return hash;
}

const char* PrintKeywords() {
  queryIDResult.clear();
  std::lock_guard<std::mutex> guard(kw_mutex);

  std::size_t count = 0;
  for (auto& kw : keywords) {
    std::stringstream ss;
    ss << "0x" << std::setfill('0') << std::setw(sizeof(uint64_t) << 1)
       << std::hex << (kw.hash | 0) << " ";
    queryIDResult += ss.str();
    queryIDResult += kw.keyword;
    queryIDResult += "\n";
    ++count;
  }
  queryIDResult += "count = " + std::to_string(count) + "\n";
  printf("Keywords Registered\n");
  printf("-------------------\n");
  printf("%s", queryIDResult.c_str());
  return queryIDResult.c_str();
}

bool AuthenticateKeywords() {
  for (auto& kwc : threadKeywordCombos) {
    if (kwc.hash == shaderHashes.hash) {
      if (kwc.hashes.size() == shaderHashes.hashes.size()) {
        for (std::size_t i = 0; i < shaderHashes.hashes.size(); ++i) {
          if (kwc.hashes[i] != shaderHashes.hashes[i]) {
            return false;
          }
        }
        return true;
      }
    }
  }

  threadKeywordCombos.push_back(shaderHashes);
  return true;
}

const char* GetKeywordsFromID(uint64_t id) {
  queryIDResult.clear();
  for (auto& s : threadKeywordCombos) {
    if (s.hash == id) {
      for (auto& h : s.hashes) {
        for (auto& kw : threadKeywords) {
          if (h == kw.hash) {
            queryIDResult += kw.keyword;
            queryIDResult += '\n';
            break;
          }
        }
      }
    }
  }
  if (!queryIDResult.empty())
    queryIDResult.pop_back();

  return queryIDResult.c_str();
}