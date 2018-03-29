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

#ifndef PARSER_HPP
#define PARSER_HPP

/* standard include */
#include <string>
#include <vector>
#include <utility>
#include <unordered_map>

/* project include */
#include "utils.hpp"

namespace yacrd {
namespace parser {

void paf(const std::string& filename, yacrd::utils::read2mapping_type* read2mapping);

void paf_line(const std::string& line, std::string* name_a, std::uint64_t* len_a, std::uint64_t* beg_a, std::uint64_t* end_a, std::string* name_b, std::uint64_t* len_b, std::uint64_t* beg_b, std::uint64_t* end_b, std::vector<std::string>& tokens);


} // namespace yacrd
} // namespace parser


#endif // PARSER_HPP
