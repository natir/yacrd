#ifndef ANALYSIS_HPP
#define ANALYSIS_HPP

/* standard include */
#include <memory>
#include <string>

/* project include */
#include "utils.hpp"

void do_work(const std::string& paf_filename, std::uint64_t coverage_min);
void add_gap(std::vector<std::unique_ptr<yacrd::utils::interval> >& middle, std::vector<std::unique_ptr<yacrd::utils::interval> >& extremity, std::unique_ptr<yacrd::utils::interval> gap, std::uint64_t readlen);

#endif // ANALYSIS_HPP
