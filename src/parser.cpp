#include <fstream>

#include "parser.hpp"

void chimdect::parser::paf(const std::string& filename, std::map<chimdect::utils::name_len, std::vector<chimdect::utils::interval> >* read2mapping)
{
    std::uint64_t switch_val;

    std::ifstream infile(filename);
    std::string line;
    while(std::getline(infile, line))
    {
        std::string name_a, name_b;
        std::uint64_t len_a, beg_a, end_a, len_b, beg_b, end_b;

        paf_line(line, &name_a, &len_a, &beg_a, &end_a, &name_b, &len_b, &beg_b, &end_b);

        if(beg_a > end_a)
        {
            switch_val = beg_a;
            beg_a = end_a;
            end_a = switch_val;
        }

        if(beg_b > end_b)
        {
            switch_val = beg_b;
            beg_b = end_b;
            end_b = switch_val;
        }

        if(read2mapping->count(std::make_pair(name_a, len_a)) == 0)
        {
            read2mapping->emplace(std::make_pair(name_a, len_a), std::vector<chimdect::utils::interval>());
        }
        read2mapping->at(std::make_pair(name_a, len_a)).push_back(std::make_pair(beg_a, end_a));

        if(read2mapping->count(std::make_pair(name_b, len_b)) == 0)
        {
            read2mapping->emplace(std::make_pair(name_b, len_b), std::vector<chimdect::utils::interval>());
        }
        read2mapping->at(std::make_pair(name_b, len_b)).push_back(std::make_pair(beg_b, end_b));
    }

}

void chimdect::parser::paf_line(const std::string& line, std::string* name_a, std::uint64_t* len_a, std::uint64_t* beg_a, std::uint64_t* end_a, std::string* name_b, std::uint64_t* len_b, std::uint64_t* beg_b, std::uint64_t* end_b)
{
    std::vector<std::string> split_line = chimdect::utils::split(line, '\t');

    *name_a = split_line[0];
    *len_a = std::stoi(split_line[1]);
    *beg_a = std::stoi(split_line[2]);
    *end_a = std::stoi(split_line[3]);

    *name_b = split_line[5];
    *len_b = std::stoi(split_line[6]);
    *beg_b = std::stoi(split_line[7]);
    *end_b = std::stoi(split_line[8]);
}
