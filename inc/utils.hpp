#ifndef UTILS_HPP
#define UTILS_HPP

#include <vector>
#include <string>

namespace chimdect {
namespace utils {

using name_len = std::pair<std::string, std::uint64_t>;
using interval = std::pair<std::uint64_t, std::uint64_t>;

std::vector<std::string> split(const std::string& s, char delimiter);

} // namespace utils
} // namespace chimdect

#endif // UTILS_HPP
