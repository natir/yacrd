#!/bin/bash

NC='\033[0m'
RED='\033[0;31m'
GREEN='\033[0;32m'

function run_test {
    diff=$(./build/yacrd -i test/${1}.${2} | diff test/${1}.out -)
    if [ "${diff}" == "" ]
    then
	echo -e "${1}.${2} : ${GREEN}PASSED${NC}"
    else
	echo -e "${1}.${2} : ${RED}FAILLED${NC}"
	echo ${diff}
    fi
}

run_test "no_coverage" "paf"
run_test "2_extremity_1_middle" "paf"
run_test "2_extremity_1_middle" "mhap"
run_test "2_extremity_1_middle_position_switch" "paf"
