#!/bin/bash

NC='\033[0m'
RED='\033[0;31m'
GREEN='\033[0;32m'

function test_output {
    diff=$(./build/yacrd -i test/${1}.${2} | diff test/${1}.out -)
    if [ "${diff}" == "" ]
    then
	echo -e "${1}.${2} : ${GREEN}PASSED${NC}"
    else
	echo -e "${1}.${2} : ${RED}FAILLED${NC}"
	echo ${diff}
    fi
}

function test_filter {
    ./build/yacrd -i test/${1}.${2} -f test/${1}.${3} -o test/${1}.filter.${3} > /dev/null
    diff=$(diff test/${1}.filter.${3} test/${1}.filter.${3}.out)
    if [ "${diff}" == "" ]
    then
	echo -e "${1}.${2} ${3}  : ${GREEN}PASSED${NC}"
    else
	echo -e "${1}.${2} ${3} : ${RED}FAILLED${NC}"
	echo ${diff}
    fi
}

test_output "no_coverage" "paf"
test_output "2_extremity_1_middle" "paf"
test_output "2_extremity_1_middle" "mhap"
test_output "2_extremity_1_middle_position_switch" "paf"

test_filter "2_extremity_1_middle" "paf" "paf"
test_filter "2_extremity_1_middle" "paf" "mhap"
test_filter "2_extremity_1_middle" "paf" "fasta"
test_filter "2_extremity_1_middle" "paf" "fastq"

