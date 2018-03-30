cmake_minimum_required(VERSION 2.8)

project(yacrd)

# warning flags
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wredundant-decls -Wcast-align -Wmissing-declarations -Wmissing-include-dirs -Wswitch-enum -Wswitch-default -Wextra -Wall -Werror -Winvalid-pch -Wredundant-decls -Wformat=2 -Wmissing-format-attribute -Wformat-nonliteral -flto")

# optimisation flags
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -O2")

include_directories(inc)

file(
    GLOB_RECURSE
    src_files
    src/*
    )

add_executable(${PROJECT_NAME} ${src_files})
