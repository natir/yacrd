#ifndef ANALYSIS_HPP
#define ANALYSIS_HPP

/* standard include */
#include <memory>
#include <string>

/* project include */
#include "utils.hpp"

void do_work(const std::string& paf_filename, std::uint64_t coverage_min);
void add_gap(std::vector<yacrd::utils::interval>& middle, std::vector<yacrd::utils::interval>& extremity, yacrd::utils::interval& gap, std::uint64_t readlen);

#endif // ANALYSIS_HPP
