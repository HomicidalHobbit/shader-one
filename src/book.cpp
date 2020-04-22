#include "book.h"

#include <cstdio>

#include "spirv.h"

static std::unique_ptr<Book> book;

struct Chapter {
  uint64_t keywords;
  std::size_t length;
};

struct Section {
  std::size_t start;
  std::size_t end;
  bool match;
};

Book::Book(const std::vector<uint32_t>& spirv) {
  printf("Book Created from spirv of size: %lu\n", spirv.size());
  base = spirv;
}

Book::~Book() {
  printf("Book Destroyed!!\n");
}

bool Book::AddChapter(const std::vector<uint32_t>& c) {
  // Find the longest string in the dictionary that matches this chapter
  chapter = c;
  std::size_t index = 0;
  std::size_t count = 0;
  std::size_t start = 0;
  std::size_t bs = base.size();
  std::size_t cs = chapter.size();
  bool matching = true;
  std::vector<Section> sections;

  while (index < bs && index < cs) {
    if (matching) {
      if (base[index] != chapter[index]) {
        sections.push_back(Section{start, index, true});
        matching = false;
        start = index++;
        continue;
      }
      ++index;
    } else {
      if (base[index] == chapter[index]) {
        sections.push_back(Section{start, index, false});
        matching = true;
        start = index++;
      }
    }
    ++count;
  }
  sections.push_back(Section{start, index, matching});

  for (auto& m : sections) {
    if (m.match) {
      printf("Matches ");
    } else {
      printf("Not Matching ");
    }
    printf("%lu to %lu\n", m.start, m.end);
  }

  return true;
}

extern "C" void CreateBook() {
  book.reset(new Book(spirv));
}

extern "C" bool AddChapter() {
  book->AddChapter(spirv);
  return true;
}