#include <fstream>
#include <iostream>
#include <string>
#include <vector>

using namespace std;
enum WordLevel {
    CET4, CET6
};

class Word {
public:
    enum WordLevel level;
    string word;
    string explanation;
    string phonetic_symbol;

    Word(enum WordLevel level, const char *word, const char *explanation,
         const char *phonetic_symbol);
};

class CETFileParser {
public:
    string filepath; // 单词文件
    vector <Word> get_words();

    CETFileParser(const char *filepath);
};

vector <Word> CETFileParser::get_words() {
    ifstream fs(filepath);
    char line[512];
    vector <Word> words;
    char word[128];
    char explanation[256];
    char phonetic_symbol[128];
    while (fs.getline(line, 512)) {
        // TODO: 从一行中读取
        /*
        喵喵喵
        */
    }
}

class Reciter {
public:
    vector <Word> cet4_words;
    vector <Word> cet6_words;
};