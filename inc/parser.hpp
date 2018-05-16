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
#include <sstream>

/* project include */
#include "utils.hpp"

namespace yacrd {
namespace parser {

struct alignment_span {
  std::string name;
  size_t beg, end, len;
};

using alignment = std::pair<alignment_span, alignment_span>;

using parser_t = void (*)(std::istringstream&, alignment&, bool);

// PAF
void file(std::istream* filename, parser_t, yacrd::utils::read2mapping_type& read2mapping);

void paf_line(std::istringstream& line, alignment& out, bool only_names=false);

void mhap_line(std::istringstream& line, alignment& out, bool only_names=false);

} // namespace parser
} // namespace yacrd

#endif // PARSER_HPP
