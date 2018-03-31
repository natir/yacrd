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

/* project include */
#include "filter.hpp"
#include "parser.hpp"

void yacrd::filter::read_write(const std::string& filter_path, const std::string& output_path, const std::unordered_set<std::string>& remove_reads)
{
    if(filter_path.substr(filter_path.find_last_of(".") + 1) == "fasta")
    {
        yacrd::filter::read_write_fasta(filter_path, output_path, remove_reads);
    }
    else if(filter_path.substr(filter_path.find_last_of(".") + 1) == "fastq")
    {
        yacrd::filter::read_write_fastq(filter_path, output_path, remove_reads);
    }
    else if(filter_path.substr(filter_path.find_last_of(".") + 1) == "mhap")
    {
        yacrd::filter::read_write_mhap(filter_path, output_path, remove_reads);
    }
    else
    {
        yacrd::filter::read_write_paf(filter_path, output_path, remove_reads);
    }
}

void yacrd::filter::read_write_fasta(const std::string& filter_path, const std::string& output_path, const std::unordered_set<std::string>& remove_reads)
{
    bool keep = true;
    std::string line;
    std::ifstream in(filter_path);
    std::ofstream out(output_path);
    while(std::getline(in, line))
    {
        if(line[0] == '>')
        {
            if(remove_reads.count(line.substr(1, line.find_first_of(' ') - 1)) == 0)
            {
                keep = true;
            }
            else
            {
                keep = false;
            }
        }
        if(keep)
        {
            out<<line<<"\n";
        }
    }
}

void yacrd::filter::read_write_fastq(const std::string& filter_path, const std::string& output_path, const std::unordered_set<std::string>& remove_reads)
{
    bool keep = true;
    std::string line;
    std::ifstream in(filter_path);
    std::ofstream out(output_path);
    while(std::getline(in, line))
    {
        if(line[0] == '@')
        {
            if(remove_reads.count(line.substr(1, line.find_first_of(' ') - 1)) == 0)
            {
                keep = true;
            }
            else
            {
                keep = false;
            }
        }
        if(keep)
        {
            out<<line<<"\n";
        }
    }
}

void yacrd::filter::read_write_mhap(const std::string& filter_path, const std::string& output_path, const std::unordered_set<std::string>& remove_reads)
{
    std::uint64_t hack;
    std::ifstream in(filter_path);
    std::ofstream out(output_path);
    std::vector<std::string> tokens;
    std::string line, name_a, name_b;
    while(std::getline(in, line))
    {
        yacrd::parser::mhap_line(line, &name_a, &hack, &hack, &hack, &name_b, &hack, &hack, &hack, tokens);
        if(remove_reads.count(name_a) == 0 && remove_reads.count(name_b) == 0)
        {
            out<<line<<"\n";
        }
    }
}

void yacrd::filter::read_write_paf(const std::string& filter_path, const std::string& output_path, const std::unordered_set<std::string>& remove_reads)
{
    std::uint64_t hack;
    std::ifstream in(filter_path);
    std::ofstream out(output_path);
    std::vector<std::string> tokens;
    std::string line, name_a, name_b;
    while(std::getline(in, line))
    {
        yacrd::parser::paf_line(line, &name_a, &hack, &hack, &hack, &name_b, &hack, &hack, &hack, tokens);
        if(remove_reads.count(name_a) == 0 && remove_reads.count(name_b) == 0)
        {
            out<<line<<"\n";
        }
    }
}
