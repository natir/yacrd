cmake_minimum_required(VERSION 2.8)

project(chimdetc)

include_directories(inc)

file(
    GLOB_RECURSE
    src_files
    src/*
    )

add_executable(${PROJECT_NAME} ${src_files})
