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
#include <fstream>
#include <utility>
#include <iostream>
#include <unordered_set>

/* getopt include */
#include <getopt.h>

/* project include */
#include "utils.hpp"
#include "parser.hpp"
#include "filter.hpp"
#include "analysis.hpp"

void print_help(void);
void print_version(void);

int main(int argc, char** argv)
{
    std::string in_filename, filter, output, format;
    std::uint64_t coverage_min = 0;

    int c;

    const struct option longopts[] = 
    {
	{"in", required_argument, 0, 'i'},
	{"min_coverage", optional_argument, 0, 'c'},
	{"filter", optional_argument, 0, 'f'},
	{"output", optional_argument, 0, 'o'},
	{0, 0, 0, 0}
    };

    int option_index = 0;
    while((c = getopt_long(argc, argv, "hvi:c:f:o:F:", longopts, &option_index)) != -1)
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
                in_filename = optarg;
                break;

	    case 'f':
		filter = optarg;
		break;

	    case 'o':
		output = optarg;
		break;

            case 'c':
                coverage_min = atol(optarg);
                break;

	    case 'F':
                format = optarg;
                break;

	    case 'v':
		print_version();
		return -1;
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

    if((!filter.empty() && output.empty()) || (filter.empty() && !output.empty()))
    {
	std::cerr<<"You need set -f,--filter and -o,--output !\n"<<std::endl;
	print_help();
	return -1;
    }

    std::istream* input = nullptr;
    if(in_filename == "-")
    {
	input = &std::cin;
    }
    else
    {
	input = new std::ifstream(in_filename);
    }

    yacrd::parser::parser_t parser = yacrd::parser::paf_line;
    if(in_filename.substr(in_filename.find_last_of(".") + 1) == "mhap")
    {
	parser = yacrd::parser::mhap_line;
    }

    if(!format.empty() && format == "mhap")
    {
	parser = yacrd::parser::mhap_line;
    }

    std::unordered_set<std::string> remove_reads = yacrd::analysis::find_chimera(input, parser, coverage_min);

    if(!filter.empty() && !output.empty())
    {
	yacrd::filter::read_write(filter, output, remove_reads);
    }

    return 0;
}

void print_help(void)
{
    std::cerr<<"usage: yacrd [-h] [-c coverage_min] [-f file_to_filter.(fasta|fastq|paf|mhap)] [-F (paf|mhap)] -i (mapping.(paf|mhap)|-) -o output.(fasta|fastq|paf|mhap)]\n";
    std::cerr<<"\n";
    std::cerr<<"options:\n";
    std::cerr<<"\t-h                   Print help message\n";
    std::cerr<<"\t-v                   Print version number\n";
    std::cerr<<"\t-c,--min_coverage    Overlap depth threshold below which a gap should be created [default: coverage 0]\n";
    std::cerr<<"\t-i,--in              Mapping input file in PAF or MHAP format (with .paf or .mhap extension), use - for read standard input\n";
    std::cerr<<"\t-f,--filter          File containing reads that will be filtered (fasta|fastq|paf), requires -o\n";
    std::cerr<<"\t-o,--output          File where filtered data are write (fasta|fastq|paf), requires -f\n";
    std::cerr<<"\t-F,--format          Force the input format paf or mhap [default: paf]";
    std::cerr<<std::endl;
}

void print_version(void)
{
    std::cerr<<"yacrd 0.2.1 Kabuto"<<std::endl;
}
