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

std::unordered_set<std::string> yacrd::analysis::find_chimera(std::istream* input, yacrd::parser::parser_t parser, std::uint64_t coverage_min, float coverage_ratio_min)
{
    yacrd::utils::read2mapping_type read2mapping;
    std::unordered_set<std::string> remove_reads;

    // parse paf file
    yacrd::parser::file(input, parser, read2mapping);

    yacrd::utils::interval_vector middle_gaps;
    std::priority_queue<size_t, std::vector<size_t>, std::greater<size_t>> stack; // interval ends

    // for each read
    for(auto read_name_len : read2mapping)
    {
        middle_gaps.clear();
        stack = std::priority_queue<size_t, std::vector<size_t>, std::greater<size_t>>();

        auto name_len = read_name_len.first;
        std::string& name = name_len.first;
        size_t len = name_len.second;
        yacrd::utils::interval_vector& intervals = read_name_len.second;

        std::sort(intervals.begin(), intervals.end());

        size_t first_covered = 0;
        size_t last_covered = 0; // end of the last sufficiently covered interval
        for(auto interval : intervals) {
            // Unstack intervals ending before the beginning of this one
            while(!stack.empty() && stack.top() < interval.first) {
                if(stack.size() > coverage_min) {
                    last_covered = stack.top();
                }
                stack.pop();
            }

            // If the new interval will cross the coverage treshold
            if(stack.size() == coverage_min) {
                if(last_covered != 0) { // Closing a gap
                    middle_gaps.emplace_back(last_covered, interval.first);
                } else { // First covered region
                    first_covered = interval.first;
                }
            }

            stack.push(interval.second);
        }

        // Unstack until we reach low coverage region or the end of the read
        while(stack.size() > coverage_min) {
            last_covered = stack.top();
            if(last_covered >= len) {
                break;
            }
            stack.pop();
        }

        // Sum first and last gap, check if the covered region is above a treshold
        size_t uncovered_extremities = first_covered + (len - last_covered);

        const char* label = nullptr; // nullptr is "pass"

        if(!middle_gaps.empty())
        {   // if read have 1 or more gap it's a chimeric read
            label = "Chimeric\t";
        } else if(uncovered_extremities > coverage_ratio_min * len) {
            label = "Not_covered\t";
        }

        if(label != nullptr)
        {
            remove_reads.insert(name);

            size_t ngaps = size_t(first_covered != 0) + size_t(last_covered != len) + middle_gaps.size();
            auto print_gap = [ngaps](std::pair<size_t, size_t> gap) mutable {
                std::cout << gap.second - gap.first << "," << gap.first<< ","<< gap.second;
                if(--ngaps > 0) {
                    std::cout << ";";
                } else {
                    std::cout << "\n";
                }
            };

            std::cout << label << name << "\t"<<len<<"\t";
            if(first_covered != 0) {
                print_gap({0, first_covered});
            }
            for(auto gap : middle_gaps)
            {
                print_gap(gap);
            }
            if(last_covered != len) {
                print_gap({last_covered, len});
            }
        }
    }

    return remove_reads;
}
