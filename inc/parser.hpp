#ifndef PARSER_HPP
#define PARSER_HPP

#include <map>
#include <string>
#include <vector>
#include <utility>

#include "utils.hpp"

namespace chimdect {
namespace parser {

void paf(const std::string& filename, std::map<chimdect::utils::name_len, std::vector<chimdect::utils::interval> >* read2mapping);
void paf_line(const std::string& line, std::string* name_a, std::uint64_t* len_a, std::uint64_t* beg_a, std::uint64_t* end_a, std::string* name_b, std::uint64_t* len_b, std::uint64_t* beg_b, std::uint64_t* end_b);


} // namespace chimdect
} // namespace parser


#endif // PARSER_HPP
