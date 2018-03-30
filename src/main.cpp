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

/* standard include */
#include <memory>
#include <string>
#include <utility>
#include <iostream>

/* getopt include */
#include <getopt.h>

/* project include */
#include "utils.hpp"
#include "parser.hpp"
#include "analysis.hpp"

void print_help(void);

int main(int argc, char** argv)
{
    std::string paf_filename;
    std::uint64_t coverage_min = 0;

    if(argc < 3)
    {
	print_help();
	return -1;
    }

    int c;

    const struct option longopts[] = 
    {
	{"in", required_argument, 0, 'i'},
	{"min_coverage", optional_argument, 0, 'c'},
	{0, 0, 0, 0}
    };

    int option_index = 0;
    while((c = getopt_long(argc, argv, "hi:c:", longopts, &option_index)) != -1)
    {
        switch(c)
        {
            case 0:
                if(longopts[option_index].flag != 0)
                    break;
                printf ("option %s", longopts[option_index].name);
                if(optarg)
                    printf(" with arg %s", optarg);
                printf ("\n");
                break;

            case 'i':
                paf_filename = optarg;
                break;

            case 'c':
                coverage_min = atol(optarg);
                break;

            case 'h':
                print_help();
                return -1;
            case '?':
		print_help();
		return -1;

            default:
		print_help();
                return -1;
        }
    }

    yacrd::analysis::find_chimera(paf_filename, coverage_min);

    return 0;
}

void print_help()
{
    std::cerr<<"usage: yacrd [-h] [-c coverage_min] -i mapping.paf\n";
    std::cerr<<"\n";
    std::cerr<<"options:\n";
    std::cerr<<"\t-h                   Print help message\n";
    std::cerr<<"\t-c,--min_coverage    If coverage are minus or equal to this create a gap [0]\n";
    std::cerr<<"\t-i,--in              Maping input file in PAF or MHAP format (with .paf or .mhap extension)\n";
    std::cerr<<std::endl;
}
