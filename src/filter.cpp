/*
Copyright (c) 2018 Pierre Marijon <pierre.marijon@inria.fr>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/* standard include */
#include <vector>
#include <fstream>
#include <sstream>

/* project include */
#include "filter.hpp"
#include "parser.hpp"


namespace { // Local definitions


inline void filter_alignment(yacrd::parser::parser_t parser, const std::string& filter_path, const std::string& output_path, const std::unordered_set<std::string>& remove_reads)
{
    std::ifstream in(filter_path);
    std::ofstream out(output_path);
    std::string line;
    std::istringstream line_stream;
    yacrd::parser::alignment align;
    while(std::getline(in, line))
    {
        line_stream.str(line);
        line_stream.clear();

        if(!line.empty()) {
            parser(line_stream, align, true);
            if(remove_reads.count(align.first.name) == 0 && remove_reads.count(align.second.name) == 0)
            {
                out<<line<<"\n";
            }
        }
    }
}

inline void filter_fasta(const std::string& filter_path, const std::string& output_path, const std::unordered_set<std::string>& remove_reads, char header_char)
{
    bool keep = true;
    std::string line;
    std::ifstream in(filter_path);
    std::ofstream out(output_path);
    while(std::getline(in, line))
    {
        if(line[0] == header_char)
        {
            keep = !remove_reads.count(line.substr(1, line.find_first_of(' ') - 1));
        }
        if(keep)
        {
            out<<line<<"\n";
        }
    }
}


}  // namespace

void yacrd::filter::read_write(const std::string& filter_path, const std::string& output_path, const std::unordered_set<std::string>& remove_reads)
{
    if(filter_path.substr(filter_path.find_last_of('.') + 1) == "fasta")
    {
        filter_fasta(filter_path, output_path, remove_reads, '>');
    }
    else if(filter_path.substr(filter_path.find_last_of('.') + 1) == "fastq")

    {
        filter_fasta(filter_path, output_path, remove_reads, '@');
    }
    else if(filter_path.substr(filter_path.find_last_of('.') + 1) == "mhap")
    {
        filter_alignment(yacrd::parser::mhap_line, filter_path, output_path, remove_reads);
    }
    else
    {
        filter_alignment(yacrd::parser::paf_line, filter_path, output_path, remove_reads);
    }
}
