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
#include <map>
#include <cmath>
#include <memory>
#include <string>
#include <utility>
#include <iostream>

/* project include */
#include "parser.hpp"
#include "analysis.hpp"

void add_gap(std::vector<std::unique_ptr<yacrd::utils::interval> >& gaps, std::unique_ptr<yacrd::utils::interval> gap, std::uint64_t readlen)
{
    if(gap->first != gap->second && gap->first != 0 && gap->second != readlen)
    {
        gaps.push_back(std::move(gap));
    }
}

void do_work(const std::string& paf_filename, std::uint64_t coverage_min)
{
    std::map<yacrd::utils::name_len, std::vector<yacrd::utils::interval>> read2mapping;

    // parse paf file
    yacrd::parser::paf(std::string(paf_filename), &read2mapping);

    // for each read
    for(auto read_name_len = read2mapping.begin(); read_name_len != read2mapping.end(); read_name_len++)
    {
        // compute coverage
        std::vector<std::uint64_t> coverage(read_name_len->first.second, 0);
        for(auto mapping : read_name_len->second)
        {
            if(mapping.second > read_name_len->first.second)
            {
                mapping.second = read_name_len->first.second;
            }

            for(auto i = mapping.first; i != mapping.second; i++)
            {
                coverage[i] += 1;
            }
        }

        // find gap in coverage
        bool in_gap = true;
        std::vector<std::unique_ptr<yacrd::utils::interval> > gaps;
        std::unique_ptr<yacrd::utils::interval> gap = std::make_unique<yacrd::utils::interval>();
        auto it = coverage.begin();
        for(; it != coverage.end(); it++)
        {
            if(*it <= coverage_min && in_gap == false)
            {
                gap = std::make_unique<yacrd::utils::interval>();
                gap->first = it - coverage.begin();
                in_gap = true;
            }

            if(*it > coverage_min && in_gap == true)
            {
                gap->second = it - coverage.begin();
                in_gap = false;
                add_gap(gaps, std::move(gap), read_name_len->first.second);

            }
        }

        if(in_gap == true)
        {
            gap->second = it - coverage.begin();
            add_gap(gaps, std::move(gap), read_name_len->first.second);
        }

        // if read have 1 or more gap it's a chimeric read
        if(gaps.size() > 0)
        {
            std::cout<<read_name_len->first.first<<":";
            for(std::uint64_t i = 0; i != gaps.size(); i++)
            {
                std::cout<<abs(gaps[i]->first - gaps[i]->second)<<","<<gaps[i]->first<<","<<gaps[i]->second<<";";
            }
            std::cout<<std::endl;
        }
    }
}

