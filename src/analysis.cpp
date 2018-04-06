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
#include <memory>
#include <string>
#include <utility>
#include <iostream>
#include <unordered_map>
#include <unordered_set>

/* project include */
#include "parser.hpp"
#include "analysis.hpp"

void yacrd::analysis::find_chimera(const std::string& paf_filename, std::uint64_t coverage_min, std::unordered_set<std::string>* remove_reads)
{
    yacrd::utils::read2mapping_type read2mapping;

    // parse paf file
    yacrd::parser::file(std::string(paf_filename), &read2mapping);

    std::vector<std::uint64_t> coverage;
    // for each read
    for(auto read_name_len : read2mapping)
    {
        // compute coverage
	coverage.assign(read_name_len.first.second, 0);
        for(auto mapping : read_name_len.second)
        {
            if(mapping.second > read_name_len.first.second)
            {
                mapping.second = read_name_len.first.second;
            }

            for(auto i = mapping.first; i != mapping.second; i++)
            {
                coverage[i] += 1;
            }
        }

        // find gap in coverage
        bool in_gap = true;
        std::vector<yacrd::utils::interval> middle_gaps;
        std::vector<yacrd::utils::interval> extremity_gaps;
        yacrd::utils::interval gap = std::make_pair<std::uint64_t, std::uint64_t>(0, 0);
        auto it = coverage.begin();
        for(; it != coverage.end(); it++)
        {
            if(*it <= coverage_min && in_gap == false)
            {
                gap = std::make_pair<std::uint64_t, std::uint64_t>(0, 0);
                gap.first = it - coverage.begin();
                in_gap = true;
            }

            if(*it > coverage_min && in_gap == true)
            {
                gap.second = it - coverage.begin();
                in_gap = false;
                add_gap(middle_gaps, extremity_gaps, gap, read_name_len.first.second);

            }
        }

        if(in_gap == true)
        {
            gap.second = it - coverage.begin();
            add_gap(middle_gaps, extremity_gaps, gap, read_name_len.first.second);
        }

        // if read have 1 or more gap it's a chimeric read
        if(middle_gaps.size() > 0)
        {
	    remove_reads->insert(read_name_len.first.first);
            std::cout<<"Chimeric:"<<read_name_len.first.first<<","<<read_name_len.first.second<<";";
            for(auto gap : middle_gaps)
            {
                std::cout<<yacrd::utils::absdiff(gap.first, gap.second)<<","<<gap.first<<","<<gap.second<<";";
            }
            std::cout<<std::endl;
            continue;
        }

        if(extremity_gaps.size() > 0)
        {
            for(auto gap : extremity_gaps)
            {
                if(yacrd::utils::absdiff(gap.first, gap.second) > 0.8 * read_name_len.first.second)
                {
                    std::cout<<"Not_covered:"<<read_name_len.first.first<<","<<read_name_len.first.second<<";";
                    std::cout<<yacrd::utils::absdiff(gap.first, gap.second)<<","<<gap.first<<","<<gap.second<<";";
                    std::cout<<std::endl;
		    remove_reads->insert(read_name_len.first.first);
                    break;
                }
            }
            continue;
        }
    }
}

void yacrd::analysis::add_gap(std::vector<yacrd::utils::interval>& middle, std::vector<yacrd::utils::interval>& extremity, yacrd::utils::interval& gap, std::uint64_t readlen)
{
    if(gap.first == gap.second)
    {
        return ;
    }

    if(gap.first == 0 || gap.second == readlen)
    {
        extremity.push_back(gap);
        return ;
    }
    middle.push_back(gap);
}


