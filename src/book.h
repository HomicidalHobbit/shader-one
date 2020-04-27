#pragma once
#include <cstdint>
#include <vector>

class Book {
 public:
  Book(const std::vector<uint32_t> &spirv);
  ~Book();

  bool AddChapter(const std::vector<uint32_t> &chapter);

 private:
  Book();
  std::vector<uint32_t> base;
  std::vector<std::vector<uint32_t>> sections;
  std::vector<std::size_t> section_index;
  std::vector<uint32_t> chapter;
};
