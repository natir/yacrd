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
#include <algorithm>
#include <queue>

/* project include */
#include "parser.hpp"
#include "analysis.hpp"

using coverage_t = std::uint_fast8_t;

namespace {

void add_gap(yacrd::utils::interval_vector& middle, yacrd::utils::interval_vector& extremity, const yacrd::utils::interval& gap, const std::uint64_t readlen)
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

}  // namespace

std::unordered_set<std::string> yacrd::analysis::find_chimera(const std::string& paf_filename, std::uint64_t coverage_min)
{
    yacrd::utils::read2mapping_type read2mapping;
    std::unordered_set<std::string> remove_reads;

    // parse paf file
    yacrd::parser::file(std::string(paf_filename), read2mapping);


    yacrd::utils::interval_vector middle_gaps;
    yacrd::utils::interval_vector extremity_gaps;
    std::priority_queue<size_t, std::vector<size_t>, std::greater<size_t>> stack; // interval ends

    // for each read
    for(auto read_name_len : read2mapping)
    {
        middle_gaps.clear();
        extremity_gaps.clear();
        stack = {};

        auto name_len = read_name_len.first;
        std::string& name = name_len.first;
        size_t len = name_len.second;
        yacrd::utils::interval_vector& intervals = read_name_len.second;

        std::sort(intervals.begin(), intervals.end());

        size_t last_covered = 0; // end of the last sufficiently covered interval
        for(auto interval : intervals) {
            // Unstack intervals ending before the beginning of this one
            while(!stack.empty() && stack.top() < interval.first) {
                if(stack.size() > coverage_min) {
                    last_covered = stack.top();
                }
                stack.pop();
            }

            if(stack.size() == coverage_min && last_covered < interval.first) {
                add_gap(middle_gaps, extremity_gaps, {last_covered, interval.first}, len);
            }

            stack.push(interval.second);
        }

        // Unstack until we reach low coverage region or the end of the read
        while(stack.size() > coverage_min && stack.top() < len) {
            last_covered = stack.top();
            stack.pop();
        }

        if(stack.size() <= coverage_min) {
            add_gap(middle_gaps, extremity_gaps, {last_covered, len}, len);
        }


        // if read have 1 or more gap it's a chimeric read
        if(!middle_gaps.empty())
        {
            remove_reads.insert(name);
            std::cout<<"Chimeric:"<<name<<","<<len<<";";
            for(auto gap : middle_gaps)
            {
                std::cout<<yacrd::utils::absdiff(gap.first, gap.second)<<","<<gap.first<<","<<gap.second<<";";
            }
            std::cout<<"\n";
            continue;
        }

        if(!extremity_gaps.empty())
        {
            for(auto gap : extremity_gaps)
            {
                if(yacrd::utils::absdiff(gap.first, gap.second) > 0.8 * len)
                {
                    std::cout<<"Not_covered:"<<name<<","<<len<<";";
                    std::cout<<yacrd::utils::absdiff(gap.first, gap.second)<<","<<gap.first<<","<<gap.second<<";";
                    std::cout<<"\n";
                    remove_reads.insert(read_name_len.first.first);
                    break;
                }
            }
            continue;
        }
    }

    return remove_reads;
}
